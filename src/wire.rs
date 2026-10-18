//! Strict Python-compatible text wire schema 1.
use crate::{Answer, AnswerValue, Boolean, Choice, Error, ErrorKind, Question, Score, State};
use indexmap::IndexMap;
use serde_json::{Map, Value, json};
/// A decoded schema-1 request. Images and Box questions are excluded.
#[derive(Clone, Debug)]
pub struct WireRequest {
    /// Validated text, object, or array state.
    pub state: State,
    /// Unique named questions in request order.
    pub questions: IndexMap<String, Question>,
    /// Optional caller identifier.
    pub request_id: Option<String>,
}
fn invalid(path: &str, message: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidRequest, message).with_path(path)
}
fn no_unknown(object: &Map<String, Value>, allowed: &[&str], path: &str) -> Result<(), Error> {
    if let Some(key) = object
        .keys()
        .filter(|k| !allowed.contains(&k.as_str()))
        .min()
    {
        return Err(invalid(
            &if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            },
            "unknown field",
        ));
    }
    Ok(())
}
/// Decode a parsed JSON request using the frozen schema-1 rules.
///
/// # Errors
/// Returns `InvalidRequest` with a field path for unsupported schema versions,
/// unknown fields, invalid state/question definitions, empty question lists,
/// or duplicate names. Box is not part of schema 1.
pub fn decode_request(request: Value) -> Result<WireRequest, Error> {
    let object = request
        .as_object()
        .ok_or_else(|| invalid("", "the request must be a JSON object"))?;
    no_unknown(
        object,
        &["schema_version", "state", "questions", "request_id"],
        "",
    )?;
    if object
        .get("schema_version")
        .is_some_and(|v| v.as_str() != Some("1"))
    {
        return Err(invalid("schema_version", "only \"1\" is supported"));
    }
    let value = object
        .get("state")
        .ok_or_else(|| invalid("state", "required"))?;
    let state = State::try_from(value.clone())
        .map_err(|_| invalid("state", "must be text, a JSON object, or an array"))?;
    let request_id = optional_string(object.get("request_id"), "request_id")?;
    let specs = object
        .get("questions")
        .and_then(Value::as_array)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| invalid("questions", "must be a non-empty array"))?;
    let mut questions = IndexMap::new();
    for (i, spec) in specs.iter().enumerate() {
        let path = format!("questions[{i}]");
        let (name, question) = decode_question(spec, &path)?;
        if questions.contains_key(&name) {
            return Err(invalid(
                &format!("{path}.name"),
                format!("{} is used twice", python_repr(&name)),
            ));
        }
        questions.insert(name, question);
    }
    Ok(WireRequest {
        state,
        questions,
        request_id,
    })
}
fn optional_string(value: Option<&Value>, path: &str) -> Result<Option<String>, Error> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        _ => Err(invalid(path, "must be a string")),
    }
}
fn decode_question(spec: &Value, path: &str) -> Result<(String, Question), Error> {
    let obj = spec
        .as_object()
        .ok_or_else(|| invalid(path, "must be a JSON object"))?;
    let kind = obj.get("type").and_then(Value::as_str).unwrap_or("");
    let mut allowed = vec!["name", "type", "instructions", "min_confidence"];
    match kind {
        "boolean" => allowed.extend(["true", "false"]),
        "choice" => allowed.push("options"),
        "score" => allowed.push("levels"),
        _ => {
            return Err(invalid(
                &format!("{path}.type"),
                "must be 'boolean', 'choice', or 'score'",
            ));
        }
    }
    no_unknown(obj, &allowed, path)?;
    let name = obj
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid(&format!("{path}.name"), "must be a non-empty string"))?;
    // Python validates side/options/levels before constructing the question.
    let descriptions = if kind == "boolean" {
        (
            optional_string(obj.get("true"), &format!("{path}.true"))?,
            optional_string(obj.get("false"), &format!("{path}.false"))?,
        )
    } else {
        (None, None)
    };
    let options = if kind == "choice" {
        Some(decode_options(obj.get("options"), path)?)
    } else {
        None
    };
    let levels = if kind == "score" {
        let levels = obj
            .get("levels")
            .and_then(Value::as_array)
            .ok_or_else(|| invalid(&format!("{path}.levels"), "must be an array of strings"))?;
        Some(levels)
    } else {
        None
    };
    let instructions = obj
        .get("instructions")
        .and_then(Value::as_str)
        .filter(|s| !crate::question::is_blank(s))
        .ok_or_else(|| invalid(path, "instructions must be a non-empty string"))?;
    let threshold = match obj.get("min_confidence") {
        None => 0.0,
        Some(v) => v
            .as_f64()
            .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
            .ok_or_else(|| invalid(path, "min_confidence must be a number from 0 to 1"))?,
    };
    let question = match kind {
        "boolean" => Boolean::new(instructions)?
            .with_descriptions(descriptions.0, descriptions.1)
            .with_min_confidence(threshold)?
            .into(),
        "choice" => {
            let options = options.unwrap_or_default();
            if options.len() < 2 {
                return Err(invalid(
                    path,
                    format!("need at least 2 options, got {}", options.len()),
                ));
            }
            Choice::new(instructions, options)?
                .with_min_confidence(threshold)?
                .into()
        }
        _ => {
            let levels = levels.map(Vec::as_slice).unwrap_or(&[]);
            if levels.len() < 2 {
                return Err(invalid(
                    path,
                    format!("need at least 2 levels, got {}", levels.len()),
                ));
            }
            let levels: Vec<String> = levels
                .iter()
                .map(|v| {
                    v.as_str()
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .ok_or_else(|| invalid(path, "levels must be non-empty strings"))
                })
                .collect::<Result<_, _>>()?;
            if levels
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != levels.len()
            {
                return Err(invalid(path, "levels must be unique"));
            }
            Score::new(instructions, levels)?
                .with_min_confidence(threshold)?
                .into()
        }
    };
    Ok((name.to_owned(), question))
}
fn decode_options(
    value: Option<&Value>,
    path: &str,
) -> Result<IndexMap<String, Option<String>>, Error> {
    let options = value
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(&format!("{path}.options"), "must be an array"))?;
    let mut result = IndexMap::new();
    for (i, option) in options.iter().enumerate() {
        let where_ = format!("{path}.options[{i}]");
        let (name, description) = match option {
            Value::String(s) => (Some(s.as_str()), None),
            Value::Object(obj) => {
                no_unknown(obj, &["name", "description"], &where_)?;
                let description =
                    optional_string(obj.get("description"), &format!("{where_}.description"))?;
                (obj.get("name").and_then(Value::as_str), description)
            }
            _ => {
                return Err(invalid(
                    &where_,
                    "must be a name or {\"name\", \"description\"}",
                ));
            }
        };
        let name = name
            .filter(|s| !s.is_empty())
            .ok_or_else(|| invalid(&where_, "needs a non-empty name"))?;
        if result.contains_key(name) {
            return Err(invalid(
                &where_,
                format!("option {} is used twice", python_repr(name)),
            ));
        }
        result.insert(name.to_owned(), description);
    }
    Ok(result)
}
fn python_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_control() => {
                let n = c as u32;
                if n < 256 {
                    out.push_str(&format!("\\x{n:02x}"));
                } else {
                    out.push_str(&format!("\\u{n:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}
fn significant(value: f64) -> Result<Value, Error> {
    let rounded = format!("{value:.3e}")
        .parse::<f64>()
        .map_err(|_| Error::new(ErrorKind::Runtime, "decimal conversion failed"))?;
    serde_json::Number::from_f64(rounded)
        .map(Value::Number)
        .ok_or_else(|| Error::new(ErrorKind::InvalidResponse, "wire numbers must be finite"))
}
/// Encode normalized answers, excluding the optional [`Answer::raw`] payload.
/// Schema 1 projects only the first answer's call.
/// The answer set must exactly cover the decoded request. This is a projection
/// of already-normalized answers; call the checked provider boundary first to
/// apply the request thresholds. Existing abstentions are preserved.
///
/// # Errors
/// Returns `InvalidResponse` for missing/extra or incompatible answers,
/// `Unsupported` for Box values/questions, or a numeric conversion error.
pub fn encode_response(
    request: &WireRequest,
    answers: &IndexMap<String, Answer>,
) -> Result<Value, Error> {
    if answers.is_empty()
        || answers.len() != request.questions.len()
        || request
            .questions
            .keys()
            .any(|name| !answers.contains_key(name))
    {
        return Err(Error::new(
            ErrorKind::InvalidResponse,
            "wire answers must cover exactly the requested questions",
        ));
    }
    let mut response = Map::new();
    if let Some(id) = &request.request_id {
        response.insert("request_id".into(), json!(id));
    }
    let mut entries = Map::new();
    for (name, answer) in answers {
        let question = &request.questions[name];
        if matches!(question, Question::Box(_)) {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "wire schema 1 does not support Box",
            ));
        }
        let value = match answer.value() {
            None => Value::Null,
            Some(AnswerValue::Boolean(v)) => json!(v),
            Some(AnswerValue::Choice(v)) => json!(v),
            Some(AnswerValue::Score(v)) => significant(*v)?,
            Some(AnswerValue::Box(_)) => {
                return Err(Error::new(
                    ErrorKind::Unsupported,
                    "wire schema 1 does not support Box",
                ));
            }
        };
        // Verify compatibility, while preserving deliberate caller abstentions.
        answer.revalidate(question)?;
        let probabilities: Map<String, Value> = answer
            .probabilities()
            .iter()
            .map(|(k, v)| Ok((k.clone(), significant(*v)?)))
            .collect::<Result<_, Error>>()?;
        entries.insert(name.clone(),json!({"type":question.kind(),"value":value,"decided":answer.decided(),"confidence":significant(answer.confidence())?,"probabilities":probabilities}));
    }
    response.insert("answers".into(), Value::Object(entries));
    let call=answers.first().and_then(|(_,a)|a.call()).map(|c|json!({"provider":c.provider,"model":c.model,"input_tokens":c.input_tokens,"output_tokens":c.output_tokens})).unwrap_or(Value::Null);
    response.insert("call".into(), call);
    Ok(Value::Object(response))
}
/// Encode a classified failure in schema 1.
pub fn encode_error(error: &Error, request_id: Option<&str>) -> Value {
    let code = match error.kind() {
        ErrorKind::Timeout => "timeout",
        ErrorKind::InvalidRequest
        | ErrorKind::InvalidQuestion
        | ErrorKind::InvalidState
        | ErrorKind::InvalidArgument => "invalid_request",
        _ => "provider_error",
    };
    let mut response = Map::new();
    if let Some(id) = request_id {
        response.insert("request_id".into(), json!(id));
    }
    response.insert(
        "error".into(),
        json!({"code":code,"message":error.to_string()}),
    );
    Value::Object(response)
}
