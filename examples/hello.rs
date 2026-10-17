//! Run `cargo run --example hello`: a typed answer from an explicit offline fixture.
mod support;
use semantic_operators::{Boolean, Operator, State};
use support::SyntheticProvider;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures_executor::block_on(async {
        let provider = SyntheticProvider::default();
        let complaint = Operator::new(
            "complaint",
            Boolean::new("Is the customer reporting a problem?")?.with_min_confidence(0.8)?,
        )?;
        let state = State::text("Synthetic support ticket: the device will not start.");
        let answer = complaint.ask(&provider, &state).await?;
        println!("OFFLINE FIXTURE — no model was called.");
        match answer.value() {
            Some(true) => println!(
                "Route to support (fixture confidence: {}).",
                answer.confidence()
            ),
            Some(false) => println!("No problem reported by the fixture."),
            None => println!("Undecided; retain this answer for human review."),
        }
        println!("Synthetic calls: {}", provider.calls());
        Ok(())
    })
}
