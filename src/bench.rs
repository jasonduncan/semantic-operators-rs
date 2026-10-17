//! Offline scoring and explicit, bounded benchmark execution.
//!
//! A benchmark runner invokes its provider: default warm-up adds one invocation.
//! Provider failures abort; they are never folded into abstention denominators.
use crate::flow::{FlowFailure, FlowOutcome};
use crate::{
    Answer, AnswerValue, Answers, Error, ErrorKind, IndexMap, Provider, Question, Questions,
    Request, Result, State, ask,
};
use futures_util::{StreamExt, TryStreamExt, stream};
use std::{future::Future, num::NonZeroUsize, time::Instant};
/// One discrete non-abstention label. Scores use zero-based rubric indices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Boolean label.
    Boolean(bool),
    /// Choice outcome name.
    Choice(String),
    /// Score rubric index.
    Score(usize),
}
/// One labeled input; omitted question labels do not contribute to metrics.
#[derive(Clone)]
pub struct Case {
    state: State,
    expected: IndexMap<String, Decision>,
}
impl std::fmt::Debug for Case {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Case")
            .field("labels", &self.expected.len())
            .finish_non_exhaustive()
    }
}
impl Case {
    /// Retain state and labels; question-specific validation happens before scoring or dispatch.
    pub fn new(state: State, expected: IndexMap<String, Decision>) -> Self {
        Self { state, expected }
    }
    /// Input state.
    pub fn state(&self) -> &State {
        &self.state
    }
    /// Supplied labels; may be an empty subset of the questions.
    pub fn expected(&self) -> &IndexMap<String, Decision> {
        &self.expected
    }
}
/// Counts and probabilities for the labeled subset of one question.
#[derive(Clone, Debug, Default)]
pub struct QuestionStats {
    correct: usize,
    undecided: usize,
    total: usize,
    p_correct: Vec<f64>,
}
impl QuestionStats {
    /// Correct decided answers.
    pub fn correct(&self) -> usize {
        self.correct
    }
    /// Undecided labeled cases.
    pub fn undecided(&self) -> usize {
        self.undecided
    }
    /// Number of cases carrying this question's label.
    pub fn total(&self) -> usize {
        self.total
    }
    /// Labeled cases with a decision.
    pub fn answered(&self) -> usize {
        self.total - self.undecided
    }
    /// Correct divided by all labeled cases; zero for an empty denominator.
    pub fn accuracy(&self) -> f64 {
        ratio(self.correct, self.total)
    }
    /// Correct divided by answered labeled cases; zero for an empty denominator.
    pub fn accuracy_when_answered(&self) -> f64 {
        ratio(self.correct, self.answered())
    }
    /// Mean probability assigned to the label, including abstentions.
    pub fn mean_p_correct(&self) -> f64 {
        if self.p_correct.is_empty() {
            0.0
        } else {
            accurate_sum(self.p_correct.iter().copied()).unwrap_or(0.0)
                / self.p_correct.len() as f64
        }
    }
    /// Per-labeled-case probabilities in case order.
    pub fn p_correct(&self) -> &[f64] {
        &self.p_correct
    }
}
fn ratio(n: usize, d: usize) -> f64 {
    if d == 0 { 0.0 } else { n as f64 / d as f64 }
}
/// A wrong decided answer; abstentions are not misses.
#[derive(Clone, Debug)]
pub struct Miss {
    /// Case position.
    pub case: usize,
    /// Question name.
    pub question: String,
    /// Correct label.
    pub expected: Decision,
    /// Full observed answer.
    pub got: Answer,
}
/// A scored batch, retaining answers, decisions and observed latency.
#[derive(Clone, Debug)]
pub struct Report {
    questions: IndexMap<String, QuestionStats>,
    latencies_ms: Vec<f64>,
    total_ms: f64,
    misses: Vec<Miss>,
    answers: Vec<Answers>,
    decisions: Vec<IndexMap<String, Option<Decision>>>,
    warmup_calls: usize,
}
impl Report {
    /// Metrics per question, in question order.
    pub fn questions(&self) -> &IndexMap<String, QuestionStats> {
        &self.questions
    }
    /// Per-call latencies; excludes time waiting for a runner concurrency slot.
    pub fn latencies_ms(&self) -> &[f64] {
        &self.latencies_ms
    }
    /// Measured batch wall time, excluding warm-up.
    pub fn total_ms(&self) -> f64 {
        self.total_ms
    }
    /// Wrong decided answers only.
    pub fn misses(&self) -> &[Miss] {
        &self.misses
    }
    /// Full answers for every case, including unlabeled questions.
    pub fn answers(&self) -> &[Answers] {
        &self.answers
    }
    /// Discrete decisions for every case and question.
    pub fn decisions(&self) -> &[IndexMap<String, Option<Decision>>] {
        &self.decisions
    }
    /// Extra provider calls made by runner warm-up; zero for purely scored data.
    pub fn warmup_calls(&self) -> usize {
        self.warmup_calls
    }
    /// Provider invocations represented by this report, including warm-up.
    pub fn calls(&self) -> usize {
        self.answers.len() + self.warmup_calls
    }
}
/// Reduce a normalized answer to a label. Score rounding is half up.
/// Box is unsupported even when its answer is undecided.
pub fn decide(question: &Question, answer: &Answer) -> Result<Option<Decision>> {
    if matches!(question, Question::Box(_)) {
        return Err(invalid("Box questions cannot be benchmarked"));
    }
    let checked = answer.revalidate(question)?;
    Ok(match checked.value() {
        None => None,
        Some(AnswerValue::Boolean(v)) => Some(Decision::Boolean(*v)),
        Some(AnswerValue::Choice(v)) => Some(Decision::Choice(v.clone())),
        Some(AnswerValue::Score(v)) => Some(Decision::Score((v + 0.5).floor() as usize)),
        Some(AnswerValue::Box(_)) => return Err(invalid("Box questions cannot be benchmarked")),
    })
}
fn label_key<'a>(q: &'a Question, label: &'a Decision) -> Result<&'a str> {
    match (q, label) {
        (Question::Boolean(_), Decision::Boolean(v)) => Ok(if *v { "true" } else { "false" }),
        (Question::Choice(q), Decision::Choice(v)) if q.options().contains_key(v) => Ok(v),
        (Question::Score(q), Decision::Score(i)) => q
            .levels()
            .get(*i)
            .map(String::as_str)
            .ok_or_else(|| invalid("Score label is outside the rubric")),
        _ => Err(invalid("label does not match the question outcomes")),
    }
}
fn validate(questions: &Questions, cases: &[Case]) -> Result<()> {
    if questions.keys().any(String::is_empty) {
        return Err(invalid("question names must be nonempty"));
    }
    if questions.values().any(|q| matches!(q, Question::Box(_))) {
        return Err(invalid("Box questions cannot be benchmarked"));
    }
    for case in cases {
        for (name, label) in &case.expected {
            let q = questions
                .get(name)
                .ok_or_else(|| invalid("case label names an unknown question"))?;
            label_key(q, label)?;
        }
    }
    Ok(())
}
fn check_latencies(count: usize, latencies: &[f64], total: f64) -> Result<()> {
    if latencies.len() != count
        || !total.is_finite()
        || total < 0.0
        || latencies.iter().any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err(invalid(
            "latencies must align with cases and be finite nonnegative milliseconds",
        ));
    }
    Ok(())
}
/// Score already-collected answers. Label subsets and empty case lists are valid.
/// Answer and latency lengths must exactly match cases, and answer keys must match questions.
pub fn score(
    questions: &Questions,
    cases: &[Case],
    answers: Vec<Answers>,
    latencies_ms: Vec<f64>,
    total_ms: f64,
) -> Result<Report> {
    validate(questions, cases)?;
    check_latencies(cases.len(), &latencies_ms, total_ms)?;
    if answers.len() != cases.len() {
        return Err(invalid("case and answer counts must match"));
    }
    let answers: Vec<Answers> = answers
        .into_iter()
        .map(|observed| {
            if observed.len() != questions.len()
                || questions.keys().any(|name| !observed.contains_key(name))
            {
                return Err(invalid(
                    "answer keys must exactly match benchmark questions",
                ));
            }
            questions
                .iter()
                .map(|(name, q)| Ok((name.clone(), observed[name].revalidate(q)?)))
                .collect()
        })
        .collect::<Result<_>>()?;
    let mut stats: IndexMap<String, QuestionStats> = questions
        .keys()
        .map(|name| (name.clone(), QuestionStats::default()))
        .collect();
    let mut misses = Vec::new();
    let mut decisions = Vec::with_capacity(cases.len());
    for (i, (case, observed)) in cases.iter().zip(&answers).enumerate() {
        if observed.len() != questions.len()
            || questions.keys().any(|name| !observed.contains_key(name))
        {
            return Err(invalid(
                "answer keys must exactly match benchmark questions",
            ));
        }
        let row: IndexMap<String, Option<Decision>> = questions
            .iter()
            .map(|(name, q)| Ok((name.clone(), decide(q, &observed[name])?)))
            .collect::<Result<_>>()?;
        for (name, label) in &case.expected {
            let answer = &observed[name];
            let q = &questions[name];
            let s = stats
                .get_mut(name)
                .ok_or_else(|| invalid("case label names an unknown question"))?;
            let p = answer
                .probabilities()
                .get(label_key(q, label)?)
                .ok_or_else(|| invalid("answer has no probability for label"))?;
            s.total += 1;
            s.p_correct.push(*p);
            match &row[name] {
                None => s.undecided += 1,
                Some(value) if value == label => s.correct += 1,
                Some(_) => misses.push(Miss {
                    case: i,
                    question: name.clone(),
                    expected: label.clone(),
                    got: answer.clone(),
                }),
            }
        }
        decisions.push(row);
    }
    Ok(Report {
        questions: stats,
        latencies_ms,
        total_ms,
        misses,
        answers,
        decisions,
        warmup_calls: 0,
    })
}
/// Apply a confidence threshold without provider calls or restoring old abstentions.
pub fn at_min_confidence(
    report: &Report,
    questions: &Questions,
    cases: &[Case],
    threshold: f64,
) -> Result<Report> {
    if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
        return Err(invalid("threshold must be finite and from zero to one"));
    }
    let answers = report
        .answers
        .iter()
        .map(|row| {
            row.iter()
                .map(|(name, answer)| {
                    Ok((name.clone(), answer.clone().with_min_confidence(threshold)?))
                })
                .collect::<Result<Answers>>()
        })
        .collect::<Result<Vec<_>>>()?;
    let mut scored = score(
        questions,
        cases,
        answers,
        report.latencies_ms.clone(),
        report.total_ms,
    )?;
    scored.warmup_calls = report.warmup_calls;
    Ok(scored)
}
/// Compare decisions across aligned nonempty reports, including identical abstentions.
/// One report is perfectly stable; stability does not measure correctness.
pub fn stability(reports: &[Report]) -> Result<IndexMap<String, f64>> {
    let first = reports
        .first()
        .ok_or_else(|| invalid("stability needs at least one report"))?;
    let count = first.decisions.len();
    if count == 0 {
        return Err(invalid("stability needs at least one case"));
    }
    let names = &first.questions;
    for report in reports {
        if report.decisions.len() != count
            || report.questions.len() != names.len()
            || names.keys().any(|n| !report.questions.contains_key(n))
        {
            return Err(invalid(
                "stability reports have mismatched cases or question names",
            ));
        }
    }
    Ok(names
        .keys()
        .map(|name| {
            let stable = (0..count)
                .filter(|i| {
                    reports
                        .iter()
                        .all(|r| r.decisions[*i][name] == first.decisions[*i][name])
                })
                .count();
            (name.clone(), stable as f64 / count as f64)
        })
        .collect())
}
/// Discounted cumulative gain at `k`; missing IDs have zero grade.
/// Returns None when ideal gain is zero. Grades and intermediate gains must be finite.
pub fn ndcg(order: &[String], grades: &IndexMap<String, u32>, k: usize) -> Result<Option<f64>> {
    let gains: IndexMap<String, f64> = grades
        .iter()
        .map(|(id, g)| {
            let exponent =
                i32::try_from(*g).map_err(|_| invalid("relevance grade overflows gain"))?;
            let gain = 2.0f64.powi(exponent) - 1.0;
            if !gain.is_finite() {
                return Err(invalid("relevance grade overflows gain"));
            }
            Ok((id.clone(), gain))
        })
        .collect::<Result<_>>()?;
    let dcg = |ids: &[&String]| -> Result<f64> {
        accurate_sum(
            ids.iter()
                .take(k)
                .enumerate()
                .map(|(i, id)| gains.get(*id).copied().unwrap_or(0.0) / ((i + 2) as f64).log2()),
        )
    };
    let mut ideal: Vec<_> = grades.keys().collect();
    ideal.sort_by_key(|id| std::cmp::Reverse(grades[*id]));
    let ideal = dcg(&ideal)?;
    if ideal == 0.0 {
        return Ok(None);
    }
    let actual: Vec<_> = order.iter().collect();
    let result = dcg(&actual)? / ideal;
    if !result.is_finite() {
        return Err(invalid("NDCG is not finite"));
    }
    Ok(Some(result))
}
/// NDCG with the Python default cutoff of ten.
pub fn ndcg_default(order: &[String], grades: &IndexMap<String, u32>) -> Result<Option<f64>> {
    ndcg(order, grades, 10)
}
/// Explicit benchmark work budget settings. Warm-up defaults to one extra call.
#[derive(Clone, Debug)]
pub struct RunOptions {
    /// First cases additionally invoked before timing, capped at the case count.
    pub warmup: usize,
    /// Maximum concurrently polled measured requests; defaults to four.
    pub concurrency: NonZeroUsize,
    /// Request-local images for each independent benchmark case.
    #[cfg(feature = "images")]
    pub assets: crate::image::Assets,
}
impl Default for RunOptions {
    fn default() -> Self {
        Self {
            warmup: 1,
            concurrency: NonZeroUsize::new(4).expect("four is nonzero"),
            #[cfg(feature = "images")]
            assets: Default::default(),
        }
    }
}
/// Invoke a provider for all cases, aborting on failure and dropping pending work.
/// Validate labels and request assets before any warm-up or measured call.
pub async fn run(
    provider: &dyn Provider,
    questions: &Questions,
    cases: &[Case],
    options: &RunOptions,
) -> Result<Report> {
    validate(questions, cases)?;
    let requests = cases
        .iter()
        .map(|case| {
            let request = Request::new(case.state.clone(), questions.clone())?;
            #[cfg(feature = "images")]
            let request = request.with_assets(options.assets.clone());
            request.validate()?;
            Ok(request)
        })
        .collect::<Result<Vec<_>>>()?;
    let warmup = options.warmup.min(requests.len());
    for request in requests.iter().take(warmup) {
        ask(provider, request).await?;
    }
    let begin = Instant::now();
    let mut observed: Vec<_> =
        stream::iter(requests.iter().enumerate().map(|(i, request)| async move {
            let start = Instant::now();
            let answers = ask(provider, request).await?;
            Ok::<_, Error>((i, answers, start.elapsed().as_secs_f64() * 1000.0))
        }))
        .buffer_unordered(options.concurrency.get())
        .try_collect()
        .await?;
    let total_ms = begin.elapsed().as_secs_f64() * 1000.0;
    observed.sort_unstable_by_key(|(i, _, _)| *i);
    let (answers, latencies) = observed.into_iter().map(|(_, a, ms)| (a, ms)).unzip();
    let mut report = score(questions, cases, answers, latencies, total_ms)?;
    report.warmup_calls = warmup;
    Ok(report)
}
/// One wrong completed flow result.
#[derive(Clone, Debug)]
pub struct FlowMiss<T> {
    /// Case position.
    pub case: usize,
    /// Expected application result.
    pub expected: T,
    /// Observed completed result.
    pub got: T,
}
/// Scored flow outcomes with their traces, expected results and latencies.
#[derive(Clone, Debug)]
pub struct FlowReport<T> {
    results: Vec<FlowOutcome<T>>,
    expected: Vec<T>,
    latencies_ms: Vec<f64>,
}
impl<T: PartialEq + Clone> FlowReport<T> {
    /// Completed or stopped outcomes, retaining every call trace.
    pub fn results(&self) -> &[FlowOutcome<T>] {
        &self.results
    }
    /// Expected application outcomes.
    pub fn expected(&self) -> &[T] {
        &self.expected
    }
    /// Latency per flow case, including all its calls.
    pub fn latencies_ms(&self) -> &[f64] {
        &self.latencies_ms
    }
    /// Number of evaluated cases.
    pub fn total(&self) -> usize {
        self.results.len()
    }
    /// Number of uncertainty stops.
    pub fn stopped(&self) -> usize {
        self.results.iter().filter(|r| !r.decided()).count()
    }
    /// Number of completed matching application values.
    pub fn correct(&self) -> usize {
        self.results
            .iter()
            .zip(&self.expected)
            .filter(|(r, e)| r.decided() && r.value.as_ref() == Some(*e))
            .count()
    }
    /// Wrong completed cases; stops are excluded.
    pub fn misses(&self) -> Vec<FlowMiss<T>> {
        self.results
            .iter()
            .zip(&self.expected)
            .enumerate()
            .filter_map(|(i, (r, e))| match &r.value {
                Some(v) if r.decided() && v != e => Some(FlowMiss {
                    case: i,
                    expected: e.clone(),
                    got: v.clone(),
                }),
                _ => None,
            })
            .collect()
    }
    /// Correct divided by completed cases, zero for no completed cases.
    pub fn accuracy_when_answered(&self) -> f64 {
        ratio(self.correct(), self.total() - self.stopped())
    }
    /// Total provider-call attempts retained in all flow traces.
    pub fn calls(&self) -> usize {
        self.results.iter().map(FlowOutcome::calls).sum()
    }
}
/// Score existing flow outcomes. Inconsistent completion/value states and failed traces are rejected.
pub fn score_flows<T: PartialEq + Clone>(
    results: Vec<FlowOutcome<T>>,
    expected: Vec<T>,
    latencies_ms: Vec<f64>,
) -> Result<FlowReport<T>> {
    if results.len() != expected.len() {
        return Err(invalid("flow outcomes and labels must align"));
    }
    check_latencies(results.len(), &latencies_ms, 0.0)?;
    for result in &results {
        validate_flow_outcome(result)?;
    }
    Ok(FlowReport {
        results,
        expected,
        latencies_ms,
    })
}
fn validate_flow_outcome<T>(result: &FlowOutcome<T>) -> Result<()> {
    if result.decided() != result.value.is_some()
        || result.trace.iter().any(|step| step.result.is_err())
    {
        return Err(invalid(
            "flow scoring requires consistent successful or stopped outcomes",
        ));
    }
    Ok(())
}
/// Run caller-supplied flow futures sequentially, retaining traces and aborting on failure.
/// There is no automatic flow warm-up or retry. Inputs are borrowed by each future.
pub async fn run_flow<'a, I, T, F, Fut>(
    cases: &'a [(I, T)],
    mut flow: F,
) -> std::result::Result<FlowReport<T>, FlowFailure>
where
    T: PartialEq + Clone,
    F: FnMut(&'a I) -> Fut,
    Fut: Future<Output = std::result::Result<FlowOutcome<T>, FlowFailure>>,
{
    let mut results = Vec::with_capacity(cases.len());
    let mut expected = Vec::with_capacity(cases.len());
    let mut latencies = Vec::with_capacity(cases.len());
    for (input, label) in cases {
        let start = Instant::now();
        let outcome = flow(input).await?;
        if let Err(error) = validate_flow_outcome(&outcome) {
            return Err(FlowFailure {
                error,
                trace: outcome.trace,
            });
        }
        results.push(outcome);
        latencies.push(start.elapsed().as_secs_f64() * 1000.0);
        expected.push(label.clone());
    }
    // Every outcome is checked before another invocation; these aligned vectors
    // and monotonic-clock latencies are constructed by this runner.
    Ok(FlowReport {
        results,
        expected,
        latencies_ms: latencies,
    })
}
fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidArgument, message)
}
// CPython math.fsum-style nonoverlapping partials and half-even final correction.
fn accurate_sum(values: impl IntoIterator<Item = f64>) -> Result<f64> {
    let mut partials: Vec<f64> = Vec::new();
    for mut x in values {
        if !x.is_finite() {
            return Err(invalid("benchmark sum is not finite"));
        }
        let mut i = 0;
        for j in 0..partials.len() {
            let mut y = partials[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            if !hi.is_finite() {
                return Err(invalid("benchmark sum overflow"));
            }
            let lo = y - (hi - x);
            if lo != 0.0 {
                partials[i] = lo;
                i += 1;
            }
            x = hi;
        }
        partials.truncate(i);
        partials.push(x);
    }
    let mut hi = partials.pop().unwrap_or(0.0);
    let mut lo = 0.0;
    while let Some(y) = partials.pop() {
        let x = hi;
        hi = x + y;
        lo = y - (hi - x);
        if lo != 0.0 {
            break;
        }
    }
    if partials
        .last()
        .is_some_and(|last| (lo < 0.0 && *last < 0.0) || (lo > 0.0 && *last > 0.0))
    {
        let y = lo * 2.0;
        let x = hi + y;
        if y == x - hi {
            hi = x;
        }
    }
    Ok(hi)
}
