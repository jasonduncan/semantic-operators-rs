//! Run `cargo run --example filter`: all four outcomes retained in input order.
mod support;
use semantic_operators::each::{BatchOptions, Item};
use semantic_operators::filter::filter_items;
use semantic_operators::{Boolean, State};
use serde_json::json;
use support::SyntheticProvider;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures_executor::block_on(async {
        let provider = SyntheticProvider::default();
        let question = Boolean::new("Does this record describe a failed release?")?
            .with_min_confidence(0.8)?;
        // Fixture controls are explicit synthetic input, not natural-language inference.
        let items = vec![
            Item::new(
                "release-a",
                State::try_from(json!({"demo_yes_probability":0.95}))?,
            )?,
            Item::new(
                "release-b",
                State::try_from(json!({"demo_yes_probability":0.05}))?,
            )?,
            Item::new(
                "release-c",
                State::try_from(json!({"demo_yes_probability":0.6}))?,
            )?,
            Item::new(
                "release-d",
                State::try_from(json!({"demo_provider_error":true}))?,
            )?,
        ];
        let result = filter_items(&provider, &question, &items, &BatchOptions::default()).await?;
        println!("OFFLINE FIXTURE — no model was called.");
        for verdict in result.verdicts() {
            println!("{}: {:?}", verdict.id(), verdict.outcome());
        }
        println!(
            "Matched: {}; unsure: {}; failed: {}; synthetic calls: {}",
            result.matched().len(),
            result.unsure().len(),
            result.failed().len(),
            provider.calls()
        );
        Ok(())
    })
}
