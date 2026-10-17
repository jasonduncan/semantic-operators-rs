//! Run `cargo run --example rerank`: stable ordering and local reweighting.
mod support;
use semantic_operators::Score;
use semantic_operators::each::BatchOptions;
use semantic_operators::rerank::{Candidate, rerank};
use serde_json::json;
use support::SyntheticProvider;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    futures_executor::block_on(async {
        let provider = SyntheticProvider::default();
        let relevance = Score::new(
            "How useful is the document for answering the query?",
            vec![
                "no useful information".into(),
                "partial answer".into(),
                "complete answer".into(),
            ],
        )?;
        let candidates = vec![
            Candidate::new("doc-a", json!({"demo_score_probabilities":[0.0,0.8,0.2]}))?,
            Candidate::new("doc-b", json!({"demo_score_probabilities":[0.5,0.0,0.5]}))?,
        ];
        let ranking = rerank(
            &provider,
            &relevance,
            "How do I reset the device?",
            &candidates,
            &BatchOptions::default(),
        )
        .await?;
        println!("OFFLINE FIXTURE — no model was called.");
        for item in ranking.top(10, false)? {
            println!("Original: {} = {}", item.id(), item.score());
        }
        let weighted = ranking.reweighted(&[0.0, 1.0, 100.0])?;
        for item in weighted.top(10, false)? {
            println!("Reweighted: {} = {}", item.id(), item.score());
        }
        println!(
            "Synthetic calls: {} (reweighting makes no extra calls)",
            provider.calls()
        );
        Ok(())
    })
}
