//! Standalone JSON Lines consumer of the semantic-operators public API.
use semantic_operators::providers::typesafe::{Config, DEFAULT_ENDPOINT, TypeSafe};
use semantic_operators::{
    Answer, Choice, IndexMap, Operator, Request, Score, State, ask, operator,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    io::{self, Read, Write},
    process::ExitCode,
    time::Duration,
};

const MAX_INPUT: usize = 1024 * 1024;
const MAX_RECORDS: usize = 100;
const HELP: &str = "semops-triage --model MODEL --max-calls N [--endpoint URL] [--timeout-ms N] [--live]\n\nReads JSON Lines {\"id\":\"ticket-1\",\"text\":\"message\"} from stdin.\nTYPESAFE_API_KEY supplies credentials. Remote HTTPS requires --live.\nOne call per record, no retries; all input validates before dispatch.\nLimits: 1 MiB input, 100 records, 16 KiB text, 1 KiB ID.\nExit: 0 completed (including review), 1 execution failure, 2 input/configuration.\n";

struct Failure {
    code: u8,
    message: String,
}
impl Failure {
    fn input(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: message.into(),
        }
    }
    fn io() -> Self {
        Self {
            code: 1,
            message: "input/output failure".into(),
        }
    }
    fn provider(error: semantic_operators::Error) -> Self {
        Self {
            code: 1,
            message: format!(
                "provider failure ({:?}); remaining records not processed",
                error.kind()
            ),
        }
    }
}
struct Options {
    endpoint: String,
    model: String,
    max_calls: usize,
    timeout_ms: u64,
}
fn options() -> Result<Option<Options>, Failure> {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|value| {
            value
                .into_string()
                .map_err(|_| Failure::input("arguments must be UTF-8"))
        })
        .collect::<Result<_, _>>()?;
    if args == ["--help"] || args == ["-h"] {
        return Ok(None);
    }
    let mut endpoint = None;
    let mut model = None;
    let mut budget = None;
    let mut timeout = None;
    let mut live = false;
    let mut seen = HashSet::new();
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        if !seen.insert(flag.as_str()) {
            return Err(Failure::input("duplicate option"));
        }
        if flag == "--live" {
            live = true;
            continue;
        }
        if !matches!(
            flag.as_str(),
            "--endpoint" | "--model" | "--max-calls" | "--timeout-ms"
        ) {
            return Err(Failure::input("unknown option; use --help"));
        }
        let value = args
            .next()
            .ok_or_else(|| Failure::input("missing option value"))?;
        match flag.as_str() {
            "--endpoint" => endpoint = Some(value.clone()),
            "--model" => model = Some(value.clone()),
            "--max-calls" => {
                budget = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| Failure::input("invalid call limit"))?,
                )
            }
            "--timeout-ms" => {
                timeout = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| Failure::input("invalid timeout"))?,
                )
            }
            _ => unreachable!(),
        }
    }
    let model = model
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| Failure::input("--model is required"))?;
    let max_calls = budget
        .filter(|v| (1..=MAX_RECORDS).contains(v))
        .ok_or_else(|| Failure::input("--max-calls must be from 1 to 100"))?;
    let timeout_ms = timeout.unwrap_or(30_000);
    if !(1..=300_000).contains(&timeout_ms) {
        return Err(Failure::input(
            "timeout must be from 1 to 300000 milliseconds",
        ));
    }
    let endpoint = endpoint.unwrap_or_else(|| DEFAULT_ENDPOINT.into());
    let url = url::Url::parse(&endpoint).map_err(|_| Failure::input("invalid endpoint"))?;
    let local = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        _ => false,
    };
    if !local && (!live || url.scheme() != "https") {
        return Err(Failure::input(
            "remote endpoints require HTTPS and --live; use a loopback IP for local fixtures",
        ));
    }
    Ok(Some(Options {
        endpoint,
        model,
        max_calls,
        timeout_ms,
    }))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ticket {
    id: String,
    text: String,
}
fn tickets(max_calls: usize) -> Result<Vec<Ticket>, Failure> {
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take((MAX_INPUT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| Failure::io())?;
    if bytes.len() > MAX_INPUT {
        return Err(Failure::input("input exceeds 1 MiB"));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| Failure::input("input must be UTF-8"))?;
    let mut result = Vec::new();
    let mut ids = HashSet::new();
    for (line, value) in text.lines().enumerate() {
        if value.trim().is_empty() {
            continue;
        }
        let ticket: Ticket = serde_json::from_str(value)
            .map_err(|_| Failure::input(format!("invalid record at line {}", line + 1)))?;
        if ticket.id.trim().is_empty()
            || ticket.id.len() > 1024
            || ticket.text.trim().is_empty()
            || ticket.text.len() > 16 * 1024
        {
            return Err(Failure::input(format!(
                "invalid id/text at line {}",
                line + 1
            )));
        }
        if !ids.insert(ticket.id.clone()) {
            return Err(Failure::input("duplicate record id"));
        }
        result.push(ticket);
        if result.len() > max_calls {
            return Err(Failure::input("input exceeds call limit"));
        }
    }
    Ok(result)
}
fn evidence<T: Serialize>(answer: &Answer<T>) -> Value {
    let call = answer.call().map(|call| {
        json!({
            "provider": call.provider, "model": call.model,
            "input_tokens": call.input_tokens, "output_tokens": call.output_tokens
        })
    });
    json!({"value": answer.value(), "confidence": answer.confidence(), "probabilities": answer.probabilities(), "call": call})
}
async fn run(options: Options) -> Result<(), Failure> {
    // Validate the complete finite batch before creating a provider or emitting output.
    let tickets = tickets(options.max_calls)?;
    let key = std::env::var("TYPESAFE_API_KEY")
        .map_err(|_| Failure::input("TYPESAFE_API_KEY is required"))?;
    let config = Config::new(key, options.model)
        .and_then(|c| c.with_endpoint(options.endpoint))
        .and_then(|c| c.with_timeout(Duration::from_millis(options.timeout_ms)))
        .map_err(|_| Failure::input("invalid provider configuration"))?;
    let provider = TypeSafe::new(config).map_err(Failure::provider)?;
    let department = Operator::new(
        "department",
        Choice::new(
            "Which team should handle this support request?",
            IndexMap::from([
                ("technical".into(), Some("Device or software issue".into())),
                ("billing".into(), Some("Invoice or payment issue".into())),
            ]),
        )
        .and_then(|q| q.with_min_confidence(0.8))
        .map_err(Failure::provider)?,
    )
    .map_err(Failure::provider)?;
    let urgency = Operator::new(
        "urgency",
        Score::new(
            "How urgent is this support request?",
            vec!["routine".into(), "soon".into(), "urgent".into()],
        )
        .and_then(|q| q.with_min_confidence(0.8))
        .map_err(Failure::provider)?,
    )
    .map_err(Failure::provider)?;
    let questions =
        operator::questions(&[department.erased(), urgency.erased()]).map_err(Failure::provider)?;
    let mut output = io::BufWriter::new(io::stdout().lock());
    for ticket in tickets {
        // IDs remain local metadata; the provider receives only ticket text.
        let request =
            Request::new(State::text(ticket.text), questions.clone()).map_err(Failure::provider)?;
        let answers = ask(&provider, &request).await.map_err(Failure::provider)?;
        let team = department.answer(&answers).map_err(Failure::provider)?;
        let urgency = urgency.answer(&answers).map_err(Failure::provider)?;
        let (status, route, priority) = match (team.value(), urgency.value()) {
            (Some(team), Some(score)) => (
                "routed",
                Some(team.as_str()),
                Some(if *score >= 1.5 { "urgent" } else { "routine" }),
            ),
            _ => ("review", None, None),
        };
        let row = json!({"id":ticket.id,"status":status,"route":route,"priority":priority,
            "answers":{"department":evidence(&team),"urgency":evidence(&urgency)}});
        serde_json::to_writer(&mut output, &row).map_err(|_| Failure::io())?;
        output.write_all(b"\n").map_err(|_| Failure::io())?;
        output.flush().map_err(|_| Failure::io())?;
    }
    Ok(())
}
#[tokio::main]
async fn main() -> ExitCode {
    let result = match options() {
        Ok(None) => io::stdout()
            .write_all(HELP.as_bytes())
            .map_err(|_| Failure::io()),
        Ok(Some(options)) => run(options).await,
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{}", error.message);
            ExitCode::from(error.code)
        }
    }
}
