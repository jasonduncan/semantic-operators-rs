//! TypeSafe transport contract tests using synthetic local inputs.
#![cfg(feature = "typesafe")]
use indexmap::IndexMap;
use semantic_operators::providers::typesafe::*;
use semantic_operators::*;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

struct Script {
    response: HttpResponse,
    requests: Mutex<Vec<HttpRequest>>,
}
impl Transport for Script {
    fn send(&self, request: HttpRequest) -> TransportFuture<'_> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            Ok(self.response.clone())
        })
    }
}
fn setup(value: Value) -> (TypeSafe, Arc<Script>) {
    let script = Arc::new(Script {
        response: HttpResponse {
            status: 200,
            body: value.to_string().into_bytes(),
        },
        requests: Mutex::new(vec![]),
    });
    (
        TypeSafe::with_transport(
            Config::new("SECRET-KEY", "selected-alias").unwrap(),
            script.clone(),
        ),
        script,
    )
}
fn request(questions: Vec<(&str, Question)>) -> Request {
    Request::new(
        State::text("PRIVATE-STATE"),
        questions.into_iter().map(|(n, q)| (n.into(), q)).collect(),
    )
    .unwrap()
}
fn boolean_request() -> Request {
    request(vec![(
        "q",
        Boolean::new("PRIVATE-QUESTION").unwrap().into(),
    )])
}
fn boolean_response() -> Value {
    json!({"model":"reported-model", "usage":{"input_tokens":17,"output_tokens":3}, "answers":{"q":{"type":"noul","noul":0.8}}})
}

#[tokio::test]
async fn boolean_null_criteria_and_redaction() {
    let (provider, transport) = setup(boolean_response());
    let result = ask(&provider, &boolean_request()).await.unwrap();
    assert_eq!(result["q"].value(), Some(&AnswerValue::Boolean(true)));
    assert_eq!(
        result["q"].call().unwrap().model.as_deref(),
        Some("reported-model")
    );
    assert_eq!(result["q"].call().unwrap().input_tokens, Some(17));
    assert!(result["q"].raw().is_none());
    {
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].body["questions"]["q"].get("criteria").is_none());
        assert_eq!(requests[0].body["model"], "selected-alias");
        for debug in [
            format!("{provider:?}"),
            format!("{:?}", requests[0]),
            format!("{:?}", transport.response),
        ] {
            for secret in [
                "SECRET-KEY",
                "PRIVATE-STATE",
                "PRIVATE-QUESTION",
                "reported-model",
            ] {
                assert!(!debug.contains(secret));
            }
        }
    }
    let described = request(vec![(
        "q",
        Boolean::new("Question")
            .unwrap()
            .with_descriptions(Some("yes".into()), None)
            .into(),
    )]);
    ask(&provider, &described).await.unwrap();
    assert_eq!(
        transport.requests.lock().unwrap()[1].body["questions"]["q"]["criteria"],
        json!({"true":"yes", "false":null})
    );
}

#[tokio::test]
async fn mixed_types_order_normalization_and_shared_call() {
    let request = request(vec![
        (
            "score",
            Score::new("rate", vec!["low".into(), "high".into()])
                .unwrap()
                .into(),
        ),
        (
            "choice",
            Choice::new(
                "choose",
                IndexMap::from([("red".into(), None), ("blue".into(), Some("cool".into()))]),
            )
            .unwrap()
            .into(),
        ),
    ]);
    let (provider, transport) = setup(
        json!({"model":"reported", "usage":{"input_tokens":false,"output_tokens":-1}, "answers":{
            "choice":{"type":"choice", "choice":"blue", "confidence":0.1, "probabilities":{"blue":0.8,"red":0.2}},
            "score":{"type":"score", "score":0.7, "confidence":0.1, "legend":{"0":"low","1":"high"}, "probabilities":{"1":0.7,"0":0.3}}
        }}),
    );
    let answers = ask(&provider, &request).await.unwrap();
    assert_eq!(
        answers.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["score", "choice"]
    );
    assert_eq!(
        answers["score"]
            .probabilities()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["low", "high"]
    );
    assert_eq!(answers["score"].confidence(), 0.7);
    assert_eq!(answers["choice"].confidence(), 0.8);
    assert!(Arc::ptr_eq(
        answers["score"].call().unwrap(),
        answers["choice"].call().unwrap()
    ));
    assert_eq!(answers["score"].call().unwrap().input_tokens, None);
    assert_eq!(answers["score"].call().unwrap().output_tokens, None);
    let requests = transport.requests.lock().unwrap();
    assert!(!requests[0].images);
    assert_eq!(
        requests[0].body["questions"]["choice"]["criteria"],
        json!({"red":null,"blue":"cool"})
    );
    assert_eq!(
        requests[0].body["questions"]["score"]["criteria"],
        json!(["low", "high"])
    );
}

#[tokio::test]
async fn invalid_outputs_fail_without_becoming_abstentions() {
    let mut variants = vec![
        Value::Null,
        json!({}),
        json!({"answers":[]}),
        json!({"answers":{"other":{"type":"noul","noul":0.8}}}),
    ];
    for answer in [
        json!({"type":"choice","noul":0.8}),
        json!({"type":"noul","noul":-0.1}),
        json!({"type":"noul","noul":true}),
        json!({"type":"noul"}),
    ] {
        variants.push(json!({"answers":{"q":answer}}));
    }
    let mut extra = boolean_response();
    extra["answers"]["extra"] = json!({"type":"noul","noul":0.7});
    variants.push(extra);
    for value in variants {
        let (provider, transport) = setup(value);
        assert_eq!(
            ask(&provider, &boolean_request()).await.unwrap_err().kind(),
            ErrorKind::InvalidResponse
        );
        assert_eq!(transport.requests.lock().unwrap().len(), 1);
    }
}

#[tokio::test]
async fn http_failures_and_malformed_json_never_retry_or_echo_body() {
    for status in [401, 403, 429, 500, 503, 302] {
        let script = Arc::new(Script {
            response: HttpResponse {
                status,
                body: b"SECRET-ECHO".to_vec(),
            },
            requests: Mutex::new(vec![]),
        });
        let provider =
            TypeSafe::with_transport(Config::new("secret", "model").unwrap(), script.clone());
        let error = ask(&provider, &boolean_request()).await.unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Provider);
        assert!(!format!("{error:?}").contains("SECRET-ECHO"));
        assert_eq!(script.requests.lock().unwrap().len(), 1);
    }
    let script = Arc::new(Script {
        response: HttpResponse {
            status: 200,
            body: b"not-json SECRET-ECHO".to_vec(),
        },
        requests: Mutex::new(vec![]),
    });
    let provider = TypeSafe::with_transport(Config::new("secret", "model").unwrap(), script);
    let error = ask(&provider, &boolean_request()).await.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidResponse);
    assert!(!error.to_string().contains("SECRET-ECHO"));
}

struct Pending {
    entered: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
}
struct Guard(Arc<AtomicUsize>);
impl Drop for Guard {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
impl Transport for Pending {
    fn send(&self, _: HttpRequest) -> TransportFuture<'_> {
        Box::pin(async move {
            self.entered.fetch_add(1, Ordering::SeqCst);
            let _guard = Guard(self.dropped.clone());
            std::future::pending().await
        })
    }
}
#[tokio::test(start_paused = true)]
async fn deadline_drops_pending_transport_without_retry() {
    let entered = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicUsize::new(0));
    let transport = Arc::new(Pending {
        entered: entered.clone(),
        dropped: dropped.clone(),
    });
    let provider = TypeSafe::with_transport(
        Config::new("secret", "model")
            .unwrap()
            .with_timeout(Duration::from_secs(1))
            .unwrap(),
        transport,
    );
    let request = boolean_request();
    let future = ask(&provider, &request);
    assert_eq!(entered.load(Ordering::SeqCst), 0);
    assert_eq!(future.await.unwrap_err().kind(), ErrorKind::Timeout);
    assert_eq!(entered.load(Ordering::SeqCst), 1);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn response_limit_and_opt_in_raw_data() {
    let (_, script) = setup(boolean_response());
    let provider = TypeSafe::with_transport(
        Config::new("secret", "model")
            .unwrap()
            .with_max_response_bytes(10)
            .unwrap(),
        script.clone(),
    );
    assert_eq!(
        ask(&provider, &boolean_request()).await.unwrap_err().kind(),
        ErrorKind::InvalidResponse
    );
    let provider = TypeSafe::with_transport(
        Config::new("secret", "model")
            .unwrap()
            .with_raw_answers(true),
        script,
    );
    assert!(
        ask(&provider, &boolean_request()).await.unwrap()["q"]
            .raw()
            .is_some()
    );
}
#[test]
fn config_validation_and_runtime_error() {
    assert!(Config::new("", "model").is_err());
    assert!(Config::new("key\r\ninject", "model").is_err());
    for endpoint in [
        "file:///secret",
        "https://key@api.example/x",
        "https://api.example/?secret=x",
        "https://api.example/#secret",
    ] {
        assert!(
            Config::new("secret", "model")
                .unwrap()
                .with_endpoint(endpoint)
                .is_err()
        );
    }
    let (provider, _) = setup(boolean_response());
    assert_eq!(
        futures_executor::block_on(ask(&provider, &boolean_request()))
            .unwrap_err()
            .kind(),
        ErrorKind::Runtime
    );
}

#[cfg(feature = "images")]
#[tokio::test]
async fn images_upload_once_and_current_service_preflight() {
    use semantic_operators::image::{Assets, Image};
    fn image(w: u32, h: u32) -> Image {
        fn chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
            let mut out = (payload.len() as u32).to_be_bytes().to_vec();
            out.extend(kind);
            out.extend(payload);
            out.extend([0; 4]);
            out
        }
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut header = w.to_be_bytes().to_vec();
        header.extend(h.to_be_bytes());
        header.extend([8, 0, 0, 0, 0]);
        out.extend(chunk(b"IHDR", &header));
        out.extend(chunk(b"IDAT", &[]));
        out.extend(chunk(b"IEND", &[]));
        Image::from_bytes(out).unwrap()
    }
    let (provider, transport) = setup(boolean_response());
    let mut assets = Assets::new();
    let id = assets.attach(image(280, 280));
    let req = Request::new(
        State::text(format!("{id} {id}")),
        IndexMap::from([(
            "q".into(),
            Boolean::new(format!("inspect {id}")).unwrap().into(),
        )]),
    )
    .unwrap()
    .with_assets(assets.clone());
    ask(&provider, &req).await.unwrap();
    {
        let requests = transport.requests.lock().unwrap();
        assert_eq!(requests[0].uploads.len(), 1);
        assert!(requests[0].images);
        assert_eq!(
            requests[0].body["assets"][&id],
            json!({"type":"image","part":"asset-0"})
        );
    }
    let bad_keys = Request::new(
        State::try_from(json!({id.clone():"ignored"})).unwrap(),
        IndexMap::from([("q".into(), Boolean::new("test").unwrap().into())]),
    )
    .unwrap()
    .with_assets(assets);
    assert_eq!(
        ask(&provider, &bad_keys).await.unwrap_err().kind(),
        ErrorKind::InvalidImage
    );
    let mut assets = Assets::new();
    let id = assets.attach(image(256, 307));
    let small = Request::new(
        State::text(id),
        IndexMap::from([("q".into(), Boolean::new("test").unwrap().into())]),
    )
    .unwrap()
    .with_assets(assets);
    assert_eq!(
        ask(&provider, &small).await.unwrap_err().kind(),
        ErrorKind::InvalidImage
    );
    assert_eq!(transport.requests.lock().unwrap().len(), 1);
    let mut assets = Assets::new();
    let id = assets.attach(image(280, 280));
    let box_request = Request::new(
        State::text(id),
        IndexMap::from([("q".into(), BoxQuestion::new("locate").unwrap().into())]),
    )
    .unwrap()
    .with_assets(assets);
    let (box_provider, _) = setup(
        json!({"answers":{"q":{"type":"box","box":{"top":0.1,"bottom":0.8,"left":0.2,"right":0.7},"confidence":null}}}),
    );
    let answers = ask(&box_provider, &box_request).await.unwrap();
    assert_eq!(answers["q"].confidence(), 0.0);
    assert!(matches!(answers["q"].value(), Some(AnswerValue::Box(_))));
}

// Exercise the real HTTP client against a loopback server: no hosted service or credentials.
#[tokio::test]
async fn reqwest_sends_bearer_json_and_bounds_chunked_response() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    fn server(response: String) -> (String, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/v1/systemone", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let handle = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let mut stream = loop {
                match listener.accept() {
                    Ok((s, _)) => break s,
                    Err(e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            && start.elapsed() < Duration::from_secs(5) =>
                    {
                        std::thread::sleep(Duration::from_millis(5))
                    }
                    Err(e) => panic!("local server: {e}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 1024];
            loop {
                let n = stream.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                request.extend(&buffer[..n]);
                if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&request[..end]).to_lowercase();
                    let length = header
                        .lines()
                        .find_map(|l| {
                            l.strip_prefix("content-length: ")
                                .and_then(|v| v.parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            stream.write_all(response.as_bytes()).unwrap();
            String::from_utf8(request).unwrap()
        });
        (endpoint, handle)
    }
    let response = boolean_response().to_string();
    let (endpoint, handle) = server(format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
        response.len()
    ));
    let provider = TypeSafe::new(
        Config::new("synthetic-key", "model")
            .unwrap()
            .with_endpoint(endpoint)
            .unwrap(),
    )
    .unwrap();
    ask(&provider, &boolean_request()).await.unwrap();
    let request = handle.join().unwrap();
    assert!(
        request
            .to_lowercase()
            .contains("authorization: bearer synthetic-key\r\n")
    );
    assert!(
        request
            .to_lowercase()
            .contains("content-type: application/json\r\n")
    );
    let (endpoint,handle)=server("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n6\r\n123456\r\n6\r\n789012\r\n0\r\n\r\n".into());
    let provider = TypeSafe::new(
        Config::new("synthetic-key", "model")
            .unwrap()
            .with_endpoint(endpoint)
            .unwrap()
            .with_max_response_bytes(10)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        ask(&provider, &boolean_request()).await.unwrap_err().kind(),
        ErrorKind::InvalidResponse
    );
    handle.join().unwrap();
}

#[test]
fn missing_timer_driver_returns_error_before_dispatch() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let (provider, transport) = setup(boolean_response());
    assert_eq!(
        runtime
            .block_on(ask(&provider, &boolean_request()))
            .unwrap_err()
            .kind(),
        ErrorKind::Runtime
    );
    assert!(transport.requests.lock().unwrap().is_empty());
}
#[test]
fn public_http_transport_returns_runtime_errors() {
    fn http_request() -> HttpRequest {
        HttpRequest {
            endpoint: "http://127.0.0.1:9/v1/systemone".into(),
            api_key: "synthetic".into(),
            body: json!({}),
            uploads: vec![],
            images: false,
            timeout: Duration::from_millis(20),
            max_response_bytes: 100,
        }
    }
    let transport = ReqwestTransport::new().unwrap();
    assert_eq!(
        futures_executor::block_on(transport.send(http_request()))
            .unwrap_err()
            .kind(),
        ErrorKind::Runtime
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    assert_eq!(
        runtime
            .block_on(transport.send(http_request()))
            .unwrap_err()
            .kind(),
        ErrorKind::Runtime
    );
}
#[tokio::test]
async fn known_service_limits_and_box_without_image_fail_before_dispatch() {
    let (provider, transport) = setup(boolean_response());
    let box_request = request(vec![("q", BoxQuestion::new("locate").unwrap().into())]);
    assert_eq!(
        ask(&provider, &box_request).await.unwrap_err().kind(),
        ErrorKind::InvalidImage
    );
    let oversized_score = request(vec![(
        "q",
        Score::new("rate", (0..11).map(|n| format!("level {n}")).collect())
            .unwrap()
            .into(),
    )]);
    assert_eq!(
        ask(&provider, &oversized_score).await.unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    let oversized_choice = request(vec![(
        "q",
        Choice::new(
            "choose",
            (0..256).map(|n| (format!("choice {n}"), None)).collect(),
        )
        .unwrap()
        .into(),
    )]);
    assert_eq!(
        ask(&provider, &oversized_choice).await.unwrap_err().kind(),
        ErrorKind::Unsupported
    );
    assert!(transport.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn malformed_choice_and_score_sdk_fields_are_rejected() {
    let q = request(vec![(
        "q",
        Score::new("rate", vec!["low".into(), "high".into()])
            .unwrap()
            .into(),
    )]);
    let valid = json!({"type":"score","score":0.7,"confidence":0.1,"legend":{"0":"low","1":"high"},"probabilities":{"0":0.3,"1":0.7}});
    for field in ["confidence", "legend", "probabilities"] {
        let mut malformed = valid.clone();
        malformed.as_object_mut().unwrap().remove(field);
        let (provider, _) = setup(json!({"answers":{"q":malformed}}));
        assert_eq!(
            ask(&provider, &q).await.unwrap_err().kind(),
            ErrorKind::InvalidResponse
        );
    }
    for probabilities in [
        json!({"low":0.3,"high":0.7}),
        json!({"0":0.3,"1":0.7,"2":0.0}),
        json!({"0":0.7,"1":0.7}),
    ] {
        let mut malformed = valid.clone();
        malformed["probabilities"] = probabilities;
        let (provider, _) = setup(json!({"answers":{"q":malformed}}));
        assert_eq!(
            ask(&provider, &q).await.unwrap_err().kind(),
            ErrorKind::InvalidResponse
        );
    }
    let q = request(vec![(
        "q",
        Choice::new(
            "choose",
            IndexMap::from([("a".into(), None), ("b".into(), None)]),
        )
        .unwrap()
        .into(),
    )]);
    let (provider, _) = setup(
        json!({"answers":{"q":{"type":"choice","choice":"a","probabilities":{"a":0.8,"b":0.2}}}}),
    );
    assert_eq!(
        ask(&provider, &q).await.unwrap_err().kind(),
        ErrorKind::InvalidResponse
    );
}
