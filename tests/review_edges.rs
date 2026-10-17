//! Independent privacy and numeric overflow review regressions.
use semantic_operators::*;
use semantic_operators::{
    each::BatchOptions,
    each::Item,
    filter::filter_items,
    rerank::{Candidate, rerank},
};
use serde_json::json;
struct Decide;
impl Provider for Decide {
    fn ask<'a>(&'a self, r: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            let q = &r.questions()["q"];
            let (v, p) = match q {
                Question::Boolean(_) => (
                    AnswerValue::Boolean(true),
                    IndexMap::from([("true".into(), 0.9), ("false".into(), 0.1)]),
                ),
                Question::Score(s) => (
                    AnswerValue::Score(1.0),
                    s.levels()
                        .iter()
                        .cloned()
                        .zip([0.333, 0.334, 0.333])
                        .collect(),
                ),
                _ => unreachable!(),
            };
            Ok(IndexMap::from([("q".into(), make_answer(q, v, p, None)?)]))
        })
    }
}
#[test]
fn question_and_recipe_debug_do_not_disclose_prompt_text_or_rubric() {
    futures_executor::block_on(async {
        let b = Boolean::new("PRIVATE-INSTRUCTIONS")
            .unwrap()
            .with_descriptions(Some("PRIVATE-TRUE".into()), Some("PRIVATE-FALSE".into()));
        let c = Choice::new(
            "PRIVATE-INSTRUCTIONS",
            IndexMap::from([
                (
                    "PRIVATE-OPTION-A".into(),
                    Some("PRIVATE-DESCRIPTION".into()),
                ),
                ("PRIVATE-OPTION-B".into(), None),
            ]),
        )
        .unwrap();
        let s = Score::new(
            "PRIVATE-INSTRUCTIONS",
            vec![
                "PRIVATE-LOW".into(),
                "PRIVATE-MIDDLE".into(),
                "PRIVATE-HIGH".into(),
            ],
        )
        .unwrap();
        let bbox = BoxQuestion::new("PRIVATE-INSTRUCTIONS").unwrap();
        for q in [b.clone().into(), c.into(), s.clone().into(), bbox.into()] {
            let q: Question = q;
            assert!(!format!("{q:?}").contains("PRIVATE"));
        }
        let filtered = filter_items(
            &Decide,
            &b,
            &[Item::new("id", State::text("PRIVATE-STATE")).unwrap()],
            &BatchOptions::default(),
        )
        .await
        .unwrap();
        assert!(!format!("{filtered:?}").contains("PRIVATE"));
        let ranking = rerank(
            &Decide,
            &s,
            "PRIVATE-QUERY",
            &[Candidate::new("id", json!("PRIVATE-DOCUMENT")).unwrap()],
            &BatchOptions::default(),
        )
        .await
        .unwrap();
        assert!(!format!("{ranking:?}").contains("PRIVATE"));
    });
}
#[test]
fn finite_extreme_weights_never_create_nonfinite_ranking_scores() {
    futures_executor::block_on(async {
        let score = Score::new("q", vec!["a".into(), "b".into(), "c".into()]).unwrap();
        let ranking = rerank(
            &Decide,
            &score,
            "query",
            &[Candidate::new("id", json!("doc")).unwrap()],
            &BatchOptions::default(),
        )
        .await
        .unwrap();
        for weights in [
            [-f64::MAX, 0.0, f64::MAX],
            [f64::MAX / 2.0, f64::MAX * 0.75, f64::MAX],
        ] {
            match ranking.reweighted(&weights) {
                Ok(r) => assert!(r.ranked().iter().all(|r| r.score().is_finite())),
                Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidArgument),
            }
        }
    });
}
#[test]
fn rounded_probability_mass_with_extreme_weights_reports_overflow() {
    struct Rounded;
    impl Provider for Rounded {
        fn ask<'a>(&'a self, r: &'a Request) -> ProviderFuture<'a> {
            Box::pin(async move {
                Ok(IndexMap::from([(
                    "q".into(),
                    make_answer(
                        &r.questions()["q"],
                        AnswerValue::Score(1.01),
                        IndexMap::from([
                            ("a".into(), 0.34),
                            ("b".into(), 0.34),
                            ("c".into(), 0.335),
                        ]),
                        None,
                    )?,
                )]))
            })
        }
    }
    futures_executor::block_on(async {
        let score = Score::new("q", vec!["a".into(), "b".into(), "c".into()]).unwrap();
        let ranking = rerank(
            &Rounded,
            &score,
            "query",
            &[Candidate::new("id", json!("doc")).unwrap()],
            &BatchOptions::default(),
        )
        .await
        .unwrap();
        let error = ranking
            .reweighted(&[f64::MAX * 0.99, f64::MAX * 0.995, f64::MAX])
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidArgument);
    });
}
#[test]
fn python_information_separator_whitespace_is_blank() {
    for character in ['\u{1c}', '\u{1d}', '\u{1e}', '\u{1f}'] {
        let blank = character.to_string();
        assert!(Boolean::new(&blank).is_err());
        let error = semantic_operators::wire::decode_request(
            json!({"state":"s","questions":[{"name":"q","type":"boolean","instructions":blank}]}),
        )
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidRequest);
        assert_eq!(error.path(), Some("questions[0]"));
    }
    // These separators remain valid labels: Python does not trim labels.
    assert!(Score::new("q", vec!["\u{1c}".into(), "b".into()]).is_ok());
}
#[test]
fn python_blank_query_is_rejected_before_dispatch() {
    futures_executor::block_on(async {
        let score = Score::new("q", vec!["a".into(), "b".into(), "c".into()]).unwrap();
        assert_eq!(
            rerank(&Decide, &score, "\u{1c}", &[], &BatchOptions::default())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidArgument
        );
    });
}
#[cfg(feature = "images")]
#[test]
fn python_blank_target_is_rejected_before_dispatch() {
    futures_executor::block_on(async {
        let request = Request::new(
            State::text("s"),
            IndexMap::from([("q".into(), Boolean::new("q").unwrap().into())]),
        )
        .unwrap();
        assert_eq!(
            semantic_operators::locate::locate(&Decide, &request, "\u{1c}", Default::default())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::InvalidArgument
        );
    });
}
