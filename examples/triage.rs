//! Run `cargo run --example triage`: batch questions, then branch with a flow trace.
mod support;
use semantic_operators::flow::{FlowContext, FlowExit};
use semantic_operators::{Boolean, Choice, IndexMap, Operator, Score, State};
use support::SyntheticProvider;

async fn triage(ctx: &mut FlowContext<'_>, state: &State) -> Result<String, FlowExit> {
    let department = Operator::new(
        "department",
        Choice::new(
            "Which team should handle the request?",
            IndexMap::from([
                ("technical".into(), Some("Device or software issue".into())),
                ("billing".into(), Some("Invoice or payment issue".into())),
            ]),
        )?,
    )?;
    let urgency = Operator::new(
        "urgency",
        Score::new(
            "How urgent is the request?",
            vec!["routine".into(), "soon".into(), "urgent".into()],
        )?,
    )?;
    let answers = ctx
        .ask(state, &[department.erased(), urgency.erased()])
        .await?;
    let team = answers.require(&department)?;
    let score = answers.require(&urgency)?;
    if team == "technical" && score >= 1.5 {
        let outage = Operator::new(
            "outage",
            Boolean::new("Is service unavailable?")?.with_min_confidence(0.8)?,
        )?;
        let answer = ctx.ask(state, &[outage.erased()]).await?;
        if answer.require(&outage)? {
            return Ok("technical: incident review".into());
        }
    }
    Ok(format!("{team}: ordinary queue"))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures_executor::block_on(async {
        let provider = SyntheticProvider::default();
        let mut ctx = FlowContext::new(&provider);
        let result = triage(&mut ctx, &State::text("Synthetic request fixture")).await;
        let outcome = ctx.finish(result)?;
        println!("OFFLINE FIXTURE — no model was called.");
        if let Some(value) = &outcome.value {
            println!("Route: {value}");
        }
        if let Some(name) = &outcome.stopped_at {
            println!("Stopped on undecided operator: {name}");
        }
        println!(
            "Trace steps: {}; synthetic calls: {}",
            outcome.calls(),
            provider.calls()
        );
        Ok(())
    })
}
