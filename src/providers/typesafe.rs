//! TypeSafe System One adapter with explicit configuration and no implicit retries.
use crate::{
    AnswerValue, Answers, Call, Error, ErrorKind, Provider, ProviderFuture, Question, Region,
    Request, Result, make_answer,
};
use futures_util::FutureExt;
use indexmap::IndexMap;
use serde_json::{Value, json};
use std::{fmt, future::Future, pin::Pin, sync::Arc, time::Duration};

/// Official System One endpoint.
pub const DEFAULT_ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
const MAX_REQUEST_BYTES: usize = 32 * 1024 * 1024;
/// Explicit adapter settings. Debug excludes credentials, URL, and model.
#[derive(Clone)]
pub struct Config {
    api_key: String,
    model: String,
    endpoint: String,
    timeout: Duration,
    max_response_bytes: usize,
    retain_raw: bool,
}
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("timeout", &self.timeout)
            .field("max_response_bytes", &self.max_response_bytes)
            .field("retain_raw", &self.retain_raw)
            .finish_non_exhaustive()
    }
}
impl Config {
    /// Explicit credentials and model; default 30-second timeout and 4 MiB response limit.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        let api_key = api_key.into();
        let model = model.into();
        if api_key.trim().is_empty()
            || api_key.chars().any(char::is_control)
            || model.trim().is_empty()
        {
            return Err(argument("TypeSafe requires nonblank credentials and model"));
        }
        Ok(Self {
            api_key,
            model,
            endpoint: DEFAULT_ENDPOINT.into(),
            timeout: Duration::from_secs(30),
            max_response_bytes: 4 * 1024 * 1024,
            retain_raw: false,
        })
    }
    /// Set the complete HTTP(S) endpoint. Userinfo, query strings, and fragments are rejected.
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Result<Self> {
        let endpoint = endpoint.into();
        let url =
            reqwest::Url::parse(&endpoint).map_err(|_| argument("invalid TypeSafe endpoint"))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(argument("invalid TypeSafe endpoint"));
        }
        self.endpoint = endpoint;
        Ok(self)
    }
    /// Bound the whole transport operation, including a custom transport.
    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self> {
        if timeout.is_zero() {
            return Err(argument("timeout must be positive"));
        }
        self.timeout = timeout;
        Ok(self)
    }
    /// Bound response allocation and JSON parsing.
    pub fn with_max_response_bytes(mut self, bytes: usize) -> Result<Self> {
        if bytes == 0 {
            return Err(argument("response byte limit must be positive"));
        }
        self.max_response_bytes = bytes;
        Ok(self)
    }
    /// Opt into retaining raw answer JSON in memory; never logs it.
    pub fn with_raw_answers(mut self, retain: bool) -> Self {
        self.retain_raw = retain;
        self
    }
}
/// Multipart upload. Debug excludes image content and names.
#[derive(Clone)]
pub struct Upload {
    /// Multipart field name.
    pub name: String,
    /// Validated MIME type.
    pub content_type: String,
    /// Immutable image data.
    pub bytes: Arc<[u8]>,
}
impl fmt::Debug for Upload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Upload")
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}
/// Request passed to a trusted transport; Debug redacts credentials and all content.
#[derive(Clone)]
pub struct HttpRequest {
    /// Complete endpoint.
    pub endpoint: String,
    /// Secret bearer token; custom transports must protect it.
    pub api_key: String,
    /// JSON body, or first multipart part.
    pub body: Value,
    /// Deduplicated image parts.
    pub uploads: Vec<Upload>,
    /// Set the image danger-zone header.
    pub images: bool,
    /// Whole-operation timeout.
    pub timeout: Duration,
    /// Maximum response body length.
    pub max_response_bytes: usize,
}
impl fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpRequest")
            .field("uploads", &self.uploads.len())
            .field("images", &self.images)
            .finish_non_exhaustive()
    }
}
/// Bounded HTTP response; Debug excludes content.
#[derive(Clone)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Raw response bytes.
    pub body: Vec<u8>,
}
impl fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpResponse")
            .field("status", &self.status)
            .field("body_bytes", &self.body.len())
            .finish()
    }
}
/// Transport future; dropping it cancels local work.
pub type TransportFuture<'a> = Pin<Box<dyn Future<Output = Result<HttpResponse>> + Send + 'a>>;
/// Caller-owned transport. Never log keys/content or start I/O until polled.
pub trait Transport: Send + Sync {
    /// Perform one attempt with no detached tasks or retries.
    fn send(&self, request: HttpRequest) -> TransportFuture<'_>;
}
/// Reusable rustls transport with redirects disabled.
#[derive(Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}
impl ReqwestTransport {
    /// Construct an HTTP client without reading provider credentials from the environment.
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .retry(reqwest::retry::never())
                .build()
                .map_err(|_| Error::new(ErrorKind::Provider, "could not build HTTP client"))?,
        })
    }
}
fn transport_error(error: reqwest::Error) -> Error {
    Error::new(
        if error.is_timeout() {
            ErrorKind::Timeout
        } else {
            ErrorKind::Provider
        },
        "TypeSafe transport failed",
    )
}
impl Transport for ReqwestTransport {
    fn send(&self, request: HttpRequest) -> TransportFuture<'_> {
        Box::pin(async move {
            if tokio::runtime::Handle::try_current().is_err() {
                return Err(Error::new(
                    ErrorKind::Runtime,
                    "TypeSafe HTTP transport requires Tokio",
                ));
            }
            std::panic::AssertUnwindSafe(async move {
                let mut builder = self
                    .client
                    .post(&request.endpoint)
                    .bearer_auth(&request.api_key)
                    .timeout(request.timeout)
                    .header("Accept", "application/json");
                if request.images {
                    builder = builder.header("TypeSafe-Danger-Zone-Images", "true");
                }
                let (body, content_type) = encode_http_body(&request)?;
                builder = builder.header("Content-Type", content_type).body(body);
                let mut response = builder.send().await.map_err(transport_error)?;
                let status = response.status().as_u16();
                if response
                    .content_length()
                    .is_some_and(|n| n > request.max_response_bytes as u64)
                {
                    return Err(invalid("response exceeds byte limit"));
                }
                let mut body = Vec::new();
                while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
                    if chunk.len() > request.max_response_bytes.saturating_sub(body.len()) {
                        return Err(invalid("response exceeds byte limit"));
                    }
                    body.extend_from_slice(&chunk);
                }
                Ok(HttpResponse { status, body })
            })
            .catch_unwind()
            .await
            .map_err(|_| {
                Error::new(
                    ErrorKind::Runtime,
                    "TypeSafe HTTP transport requires enabled Tokio I/O and timer drivers",
                )
            })?
        })
    }
}
struct BoundedBody(Vec<u8>);
impl std::io::Write for BoundedBody {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_REQUEST_BYTES.saturating_sub(self.0.len()) {
            return Err(std::io::Error::other("request body too large"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
// Encode ourselves to enforce the exact complete body limit before any network I/O.
fn encode_http_body(request: &HttpRequest) -> Result<(Vec<u8>, String)> {
    let mut bounded = BoundedBody(Vec::new());
    serde_json::to_writer(&mut bounded, &request.body)
        .map_err(|_| argument("request exceeds 32 MiB or could not be encoded"))?;
    let json = bounded.0;
    if request.uploads.is_empty() {
        if json.len() > MAX_REQUEST_BYTES {
            return Err(argument("request exceeds 32 MiB"));
        }
        return Ok((json, "application/json".into()));
    }
    // Choose a deterministic boundary absent from every part, including adversarial bytes.
    let boundary = (0..16)
        .map(|suffix| format!("semops-rust-multipart-{suffix}"))
        .find(|candidate| {
            !json
                .windows(candidate.len())
                .any(|w| w == candidate.as_bytes())
                && !request.uploads.iter().any(|u| {
                    u.bytes
                        .windows(candidate.len())
                        .any(|w| w == candidate.as_bytes())
                })
        })
        .ok_or_else(|| argument("could not choose multipart boundary within bounded attempts"))?;
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"request\"\r\nContent-Type: application/json\r\n\r\n").as_bytes());
    body.extend_from_slice(&json);
    body.extend_from_slice(b"\r\n");
    for upload in &request.uploads {
        if upload
            .name
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '-')
            || !matches!(upload.content_type.as_str(), "image/png" | "image/jpeg")
        {
            return Err(argument("invalid multipart metadata"));
        }
        let header = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\nContent-Type: {}\r\n\r\n",
            upload.name, upload.name, upload.content_type
        );
        if body
            .len()
            .saturating_add(header.len())
            .saturating_add(upload.bytes.len())
            .saturating_add(2)
            > MAX_REQUEST_BYTES
        {
            return Err(argument("request exceeds 32 MiB"));
        }
        body.extend_from_slice(header.as_bytes());
        body.extend_from_slice(&upload.bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    if body.len() > MAX_REQUEST_BYTES {
        return Err(argument("request exceeds 32 MiB"));
    }
    Ok((body, format!("multipart/form-data; boundary={boundary}")))
}
/// Hosted provider using an explicit transport and configuration.
pub struct TypeSafe {
    config: Config,
    transport: Arc<dyn Transport>,
}
impl fmt::Debug for TypeSafe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeSafe")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
impl TypeSafe {
    /// Build the default reusable transport.
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self::with_transport(
            config,
            Arc::new(ReqwestTransport::new()?),
        ))
    }
    /// Use a trusted injected transport, for tests or application networking.
    pub fn with_transport(config: Config, transport: Arc<dyn Transport>) -> Self {
        Self { config, transport }
    }
    fn encode(&self, request: &Request) -> Result<HttpRequest> {
        let mut questions = serde_json::Map::new();
        for (name, question) in request.questions() {
            let mut q = json!({"type": kind(question), "instructions": question.instructions()});
            match question {
                Question::Boolean(b) => {
                    if b.true_description().is_some() || b.false_description().is_some() {
                        q["criteria"] =
                            json!({"true": b.true_description(), "false": b.false_description()});
                    }
                }
                Question::Choice(c) => {
                    if c.options().len() > 255 {
                        return Err(Error::new(
                            ErrorKind::Unsupported,
                            "TypeSafe Choice supports at most 255 options",
                        ));
                    }
                    q["criteria"] = serde_json::to_value(c.options())
                        .map_err(|_| argument("could not encode criteria"))?;
                }
                Question::Score(s) => {
                    if s.levels().len() > 10 {
                        return Err(Error::new(
                            ErrorKind::Unsupported,
                            "TypeSafe Score supports at most 10 levels",
                        ));
                    }
                    q["criteria"] = json!(s.levels());
                }
                Question::Box(_) => {}
            }
            questions.insert(name.clone(), q);
        }
        #[allow(unused_mut)]
        let mut body = json!({"model": self.config.model, "state": request.state().as_value(), "questions": questions});
        #[allow(unused_mut)]
        let mut uploads = Vec::new();
        #[cfg(feature = "images")]
        {
            request
                .assets()
                .validate_service_request(request.state(), request.questions())?;
            let references = request
                .assets()
                .references(request.state(), request.questions())?;
            let mut assets = serde_json::Map::new();
            for reference in references {
                let image = reference.image();
                if image.width() < 280 || image.height() < 280 {
                    return Err(Error::new(
                        ErrorKind::InvalidImage,
                        "current TypeSafe API requires image sides of at least 280 pixels",
                    ));
                }
                let name = format!("asset-{}", uploads.len());
                assets.insert(
                    image.placeholder().to_owned(),
                    json!({"type":"image", "part":name}),
                );
                uploads.push(Upload {
                    name,
                    content_type: image.format().mime_type().to_owned(),
                    bytes: Arc::from(image.bytes()),
                });
            }
            if !assets.is_empty() {
                body["assets"] = Value::Object(assets);
            }
        }
        let has_box = request
            .questions()
            .values()
            .any(|q| matches!(q, Question::Box(_)));
        if has_box && uploads.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidImage,
                "TypeSafe Box requires an attached image",
            ));
        }
        let images = !uploads.is_empty() || has_box;
        let encoded = HttpRequest {
            endpoint: self.config.endpoint.clone(),
            api_key: self.config.api_key.clone(),
            body,
            uploads,
            images,
            timeout: self.config.timeout,
            max_response_bytes: self.config.max_response_bytes,
        };
        encode_http_body(&encoded)?;
        Ok(encoded)
    }
}
impl Provider for TypeSafe {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            if tokio::runtime::Handle::try_current().is_err() {
                return Err(Error::new(
                    ErrorKind::Runtime,
                    "TypeSafe requires a Tokio runtime",
                ));
            }
            let encoded = self.encode(request)?;
            let response =
                crate::provider::deadline(self.config.timeout, self.transport.send(encoded))
                    .await??;
            if !(200..300).contains(&response.status) {
                return Err(Error::new(
                    ErrorKind::Provider,
                    format!("TypeSafe HTTP status {}", response.status),
                ));
            }
            if response.body.len() > self.config.max_response_bytes {
                return Err(invalid("response exceeds byte limit"));
            }
            let value: Value = serde_json::from_slice(&response.body)
                .map_err(|_| invalid("response is not valid JSON"))?;
            decode(request, &value, self.config.retain_raw)
        })
    }
}
fn argument(message: &str) -> Error {
    Error::new(ErrorKind::InvalidArgument, message)
}
fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidResponse, message)
}
fn kind(question: &Question) -> &'static str {
    match question {
        Question::Boolean(_) => "noul",
        Question::Choice(_) => "choice",
        Question::Score(_) => "score",
        Question::Box(_) => "box",
    }
}
fn number(value: &Value) -> Result<f64> {
    value
        .as_f64()
        .filter(|n| n.is_finite())
        .ok_or_else(|| invalid("response contains an invalid number"))
}
fn decode(request: &Request, response: &Value, retain_raw: bool) -> Result<Answers> {
    let raw_answers = response
        .get("answers")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("response has no answers object"))?;
    if raw_answers.len() != request.questions().len()
        || request
            .questions()
            .keys()
            .any(|name| !raw_answers.contains_key(name))
    {
        return Err(invalid("response question set does not match request"));
    }
    let call = Arc::new(Call {
        provider: "TypeSafe".into(),
        model: response
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_owned),
        input_tokens: response["usage"]["input_tokens"].as_u64(),
        output_tokens: response["usage"]["output_tokens"].as_u64(),
    });
    let mut answers = IndexMap::new();
    for (name, question) in request.questions() {
        let raw = &raw_answers[name];
        if raw.get("type").and_then(Value::as_str) != Some(kind(question)) {
            return Err(invalid("answer type does not match question"));
        }
        let mut probabilities = IndexMap::new();
        let mut confidence = None;
        let value = match question {
            Question::Boolean(_) => {
                let p = number(&raw["noul"])?;
                probabilities.insert("true".into(), p);
                probabilities.insert(
                    "false".into(),
                    crate::complement(p).map_err(|_| invalid("invalid Boolean probability"))?,
                );
                AnswerValue::Boolean(p > 0.5)
            }
            Question::Choice(c) => {
                number(&raw["confidence"])?;
                let p = raw["probabilities"]
                    .as_object()
                    .ok_or_else(|| invalid("choice probabilities must be an object"))?;
                if p.len() != c.options().len() {
                    return Err(invalid("choice outcome set does not match question"));
                }
                for name in c.options().keys() {
                    probabilities
                        .insert(name.clone(), number(p.get(name).unwrap_or(&Value::Null))?);
                }
                AnswerValue::Choice(
                    raw["choice"]
                        .as_str()
                        .ok_or_else(|| invalid("choice is not a string"))?
                        .to_owned(),
                )
            }
            Question::Score(s) => {
                number(&raw["confidence"])?;
                let legend = raw["legend"]
                    .as_object()
                    .ok_or_else(|| invalid("score legend must be an object"))?;
                if legend.iter().any(|(index, value)| {
                    index.parse::<usize>().is_err()
                        || !matches!(value, Value::String(_) | Value::Object(_) | Value::Array(_))
                }) {
                    return Err(invalid("score legend is malformed"));
                }
                let p = raw["probabilities"]
                    .as_object()
                    .ok_or_else(|| invalid("score probabilities must be an object"))?;
                if p.len() != s.levels().len() {
                    return Err(invalid("score outcome set does not match question"));
                }
                for (i, level) in s.levels().iter().enumerate() {
                    probabilities.insert(
                        level.clone(),
                        number(p.get(&i.to_string()).unwrap_or(&Value::Null))?,
                    );
                }
                AnswerValue::Score(number(&raw["score"])?)
            }
            Question::Box(_) => {
                let b = &raw["box"];
                let region = Region::new(
                    number(&b["top"])?,
                    number(&b["bottom"])?,
                    number(&b["left"])?,
                    number(&b["right"])?,
                )
                .map_err(|_| invalid("invalid box coordinates"))?;
                if !raw["confidence"].is_null() {
                    confidence = Some(number(&raw["confidence"])?);
                }
                AnswerValue::Box(region)
            }
        };
        let mut answer = make_answer(question, value, probabilities, confidence)
            .map_err(|_| invalid("answer failed semantic validation"))?
            .with_call(call.clone());
        if retain_raw {
            answer = answer.with_raw(raw.clone());
        }
        answers.insert(name.clone(), answer);
    }
    Ok(answers)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> HttpRequest {
        HttpRequest {
            endpoint: DEFAULT_ENDPOINT.into(),
            api_key: "secret".into(),
            body: json!({"state":"test","assets":{"photo":{"type":"image","part":"asset-0"}}}),
            uploads: vec![Upload {
                name: "asset-0".into(),
                content_type: "image/png".into(),
                bytes: Arc::from(b"image-semops-rust-multipart-0\0".as_slice()),
            }],
            images: true,
            timeout: Duration::from_secs(1),
            max_response_bytes: 1000,
        }
    }
    #[test]
    fn multipart_request_first_and_collision_safe_boundary() {
        let request = request();
        let (body, content_type) = encode_http_body(&request).unwrap();
        assert_eq!(
            content_type,
            "multipart/form-data; boundary=semops-rust-multipart-1"
        );
        let text = String::from_utf8(body).unwrap();
        let first = text.find("name=\"request\"").unwrap();
        let image = text.find("name=\"asset-0\"").unwrap();
        assert!(first < image);
        assert!(text.contains("Content-Type: application/json\r\n\r\n"));
        assert!(text.contains("filename=\"asset-0\"\r\nContent-Type: image/png\r\n\r\n"));
        assert!(text.ends_with("--semops-rust-multipart-1--\r\n"));
    }
    #[test]
    fn multipart_metadata_and_full_size_are_checked() {
        let mut collision = request();
        collision.uploads[0].bytes = Arc::from(
            (0..16)
                .map(|n| format!("semops-rust-multipart-{n} "))
                .collect::<String>()
                .into_bytes(),
        );
        assert_eq!(
            encode_http_body(&collision).unwrap_err().kind(),
            ErrorKind::InvalidArgument
        );
        let mut request = request();
        request.uploads[0].name = "bad\r\ninjection".into();
        assert!(encode_http_body(&request).is_err());
        request.uploads[0].name = "asset-0".into();
        request.uploads[0].bytes = Arc::from(vec![0; MAX_REQUEST_BYTES]);
        assert_eq!(
            encode_http_body(&request).unwrap_err().kind(),
            ErrorKind::InvalidArgument
        );
    }
}
