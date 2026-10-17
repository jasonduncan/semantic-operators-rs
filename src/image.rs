//! Explicit request-owned images and frame sequences.
//!
//! Header inspection enforces the frozen Python 0.12 limits; it does not decode
//! pixels or prove that a file is renderable. In particular, CRCs and compressed
//! payloads are not verified. No resizing, EXIF correction, or implicit I/O occurs.
//! The hosted service may have stricter limits (its 2026-10-17 documentation says
//! 280 px minimum sides); image acceptance here is not service acceptance.

use crate::{Error, ErrorKind, Question, Result, State};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use indexmap::IndexMap;
use serde_json::Value;
use std::{
    fmt,
    io::Read,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

/// Frozen minimum side, subject also to the minimum area.
pub const MIN_SIDE: u32 = 256;
/// Maximum side length in pixels.
pub const MAX_SIDE: u32 = 1568;
/// Minimum pixel area.
pub const MIN_AREA: u64 = 78_400;
/// Maximum pixel area.
pub const MAX_AREA: u64 = 2_458_624;
/// Maximum bytes of one image.
pub const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;
/// Maximum image bytes per request, counting every use.
pub const MAX_REQUEST_BYTES: usize = 32 * 1024 * 1024;
static NEXT_IMAGE: AtomicU64 = AtomicU64::new(1);

/// Supported still-image encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    /// Portable Network Graphics.
    Png,
    /// JPEG.
    Jpeg,
}
impl ImageFormat {
    /// Lowercase provider encoding name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
        }
    }
    /// MIME type for a multipart upload.
    pub const fn mime_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }
}

/// Nonfatal metadata observations; the original bytes remain unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageDiagnostic {
    /// EXIF orientation other than 1; rotate pixels explicitly if necessary.
    ExifOrientation(u16),
    /// JPEG uses a non-baseline frame marker.
    NonBaselineJpeg,
}

/// Immutable owned image bytes with a stable, process-unique reference.
///
/// Cloning preserves reference identity. References are opaque tokens, not
/// security capabilities and not portable serialization identifiers.
#[derive(Clone)]
pub struct Image {
    data: Arc<[u8]>,
    format: ImageFormat,
    width: u32,
    height: u32,
    placeholder: String,
    diagnostics: Vec<ImageDiagnostic>,
}
impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("format", &self.format)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("size", &self.size())
            .field("diagnostics", &self.diagnostics)
            .finish_non_exhaustive()
    }
}
impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.placeholder)
    }
}
impl Image {
    /// Inspect owned bytes without decoding or modifying pixels.
    pub fn from_bytes(data: impl Into<Arc<[u8]>>) -> Result<Self> {
        let data = data.into();
        if data.len() > MAX_FILE_BYTES {
            return Err(invalid("image exceeds 16 MiB per-file limit"));
        }
        let (format, width, height, diagnostics) = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            let (w, h) = png(&data)?;
            (ImageFormat::Png, w, h, Vec::new())
        } else if data.starts_with(b"\xff\xd8\xff") {
            let (w, h, d) = jpeg(&data)?;
            (ImageFormat::Jpeg, w, h, d)
        } else {
            return Err(invalid("image must be PNG or JPEG"));
        };
        if !(MIN_SIDE..=MAX_SIDE).contains(&width) || !(MIN_SIDE..=MAX_SIDE).contains(&height) {
            return Err(invalid("image sides must be 256-1568 pixels"));
        }
        if !(MIN_AREA..=MAX_AREA).contains(&(u64::from(width) * u64::from(height))) {
            return Err(invalid("image area must be 78400-2458624 pixels"));
        }
        // New Rust versions rename this to try_update; retain the Rust 1.85 API.
        #[allow(deprecated)]
        let id = NEXT_IMAGE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| invalid("image reference identity space exhausted"))?;
        let placeholder = format!(
            "image-00000000-0000-0000-{:04x}-{:012x}",
            id >> 48,
            id & 0xffff_ffff_ffff
        );
        Ok(Self {
            data,
            format,
            width,
            height,
            placeholder,
            diagnostics,
        })
    }
    /// Read a caller-selected local path, with a bounded read and redacted errors.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path).map_err(|_| invalid("cannot open image file"))?;
        let mut data = Vec::new();
        file.take((MAX_FILE_BYTES + 1) as u64)
            .read_to_end(&mut data)
            .map_err(|_| invalid("cannot read image file"))?;
        Self::from_bytes(data)
    }
    /// Decode standard padded base64, ignoring whitespace; data URLs are rejected.
    pub fn from_base64(text: &str) -> Result<Self> {
        if text.starts_with("data:") {
            return Err(invalid("base64 image must not have a data: URL prefix"));
        }
        let mut compact = String::new();
        let max_encoded = MAX_FILE_BYTES.div_ceil(3) * 4;
        for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
            if compact.len() >= max_encoded {
                return Err(invalid("base64 image exceeds 16 MiB limit"));
            }
            compact.push(ch);
        }
        let bytes = STANDARD
            .decode(compact)
            .map_err(|_| invalid("invalid base64 image data"))?;
        Self::from_bytes(bytes)
    }
    /// Original immutable bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
    /// Encoding discovered from the file signature.
    pub fn format(&self) -> ImageFormat {
        self.format
    }
    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
    /// Original encoded byte length.
    pub fn size(&self) -> usize {
        self.data.len()
    }
    /// Opaque reference to embed in state or question strings.
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }
    /// Metadata observations that have not been applied to the pixels.
    pub fn diagnostics(&self) -> &[ImageDiagnostic] {
        &self.diagnostics
    }
    /// Standard padded base64 of the original bytes.
    pub fn base64(&self) -> String {
        STANDARD.encode(&self.data)
    }
}

/// Request-owned association of references and images. Unused images are not sent.
#[derive(Clone, Default)]
pub struct Assets {
    images: IndexMap<String, Image>,
}
impl fmt::Debug for Assets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Assets")
            .field("image_count", &self.images.len())
            .finish_non_exhaustive()
    }
}
/// One referenced image, with every occurrence accounted for.
#[derive(Debug)]
pub struct ImageReference<'a> {
    image: &'a Image,
    uses: usize,
}
impl<'a> ImageReference<'a> {
    /// Referenced image.
    pub fn image(&self) -> &'a Image {
        self.image
    }
    /// Number of occurrences in state and questions, including keys for baseline parity.
    pub fn uses(&self) -> usize {
        self.uses
    }
}
impl Assets {
    /// Create an empty request-owned registry.
    pub fn new() -> Self {
        Self::default()
    }
    /// Retain an image and return its stable reference. Reattaching clones is idempotent.
    pub fn attach(&mut self, image: Image) -> String {
        let key = image.placeholder.clone();
        self.images.entry(key.clone()).or_insert(image);
        key
    }
    /// Number of attached images (including currently unused images).
    pub fn len(&self) -> usize {
        self.images.len()
    }
    /// Whether the registry has no attached images.
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
    /// Validate unknown references and repeated-use byte limits before I/O.
    pub fn validate(&self, state: &State, questions: &IndexMap<String, Question>) -> Result<()> {
        self.references(state, questions).map(|_| ())
    }
    /// Collect referenced images in first-use order, preserving repeated-use counts.
    ///
    /// Object keys are scanned for Python 0.12 parity. The current hosted service
    /// does not replace references inside keys; a key is not an image placement.
    pub fn references<'a>(
        &'a self,
        state: &State,
        questions: &IndexMap<String, Question>,
    ) -> Result<Vec<ImageReference<'a>>> {
        let mut found = IndexMap::<String, usize>::new();
        self.walk(state.as_value(), &mut found)?;
        for (name, question) in questions {
            self.text(name, &mut found)?;
            self.text(question.instructions(), &mut found)?;
            match question {
                Question::Boolean(q) => {
                    if let Some(text) = q.true_description() {
                        self.text(text, &mut found)?;
                    }
                    if let Some(text) = q.false_description() {
                        self.text(text, &mut found)?;
                    }
                }
                Question::Choice(q) => {
                    for (name, description) in q.options() {
                        self.text(name, &mut found)?;
                        if let Some(text) = description {
                            self.text(text, &mut found)?;
                        }
                    }
                }
                Question::Score(q) => {
                    for text in q.levels() {
                        self.text(text, &mut found)?;
                    }
                }
                Question::Box(_) => {}
            }
        }
        let mut total = 0usize;
        let mut result = Vec::new();
        for (key, uses) in found {
            let image = self
                .images
                .get(&key)
                .ok_or_else(|| invalid("unknown image reference"))?;
            total = image
                .size()
                .checked_mul(uses)
                .and_then(|bytes| total.checked_add(bytes))
                .ok_or_else(|| invalid("image request size overflow"))?;
            if total > MAX_REQUEST_BYTES {
                return Err(invalid(
                    "images exceed 32 MiB request limit, counting every use",
                ));
            }
            result.push(ImageReference { image, uses });
        }
        Ok(result)
    }
    /// Apply TypeSafe's documented 2026-10-17 preflight in addition to baseline limits.
    ///
    /// Current service sides start at 280 px and object keys are never image
    /// placements. Processed image size and server decoding cannot be proved here.
    pub fn validate_service_request(
        &self,
        state: &State,
        questions: &IndexMap<String, Question>,
    ) -> Result<()> {
        for reference in self.references(state, questions)? {
            if reference.image.width() < 280 || reference.image.height() < 280 {
                return Err(invalid("TypeSafe image sides must be at least 280 pixels"));
            }
        }
        let reject_key = |key: &str| -> Result<()> {
            let mut found = IndexMap::new();
            self.text(key, &mut found)?;
            if !found.is_empty() {
                return Err(invalid(
                    "TypeSafe does not replace image references in object keys",
                ));
            }
            Ok(())
        };
        let mut stack = vec![state.as_value()];
        while let Some(value) = stack.pop() {
            match value {
                Value::Object(values) => {
                    for (key, value) in values {
                        reject_key(key)?;
                        stack.push(value);
                    }
                }
                Value::Array(values) => stack.extend(values),
                _ => {}
            }
        }
        for (name, question) in questions {
            reject_key(name)?;
            if let Question::Choice(choice) = question {
                for name in choice.options().keys() {
                    reject_key(name)?;
                }
            }
        }
        Ok(())
    }
    fn walk(&self, value: &Value, found: &mut IndexMap<String, usize>) -> Result<()> {
        enum Item<'a> {
            Value(&'a Value),
            Key(&'a str),
        }
        let mut stack = vec![Item::Value(value)];
        while let Some(item) = stack.pop() {
            match item {
                Item::Key(key) => self.text(key, found)?,
                Item::Value(Value::String(text)) => self.text(text, found)?,
                Item::Value(Value::Array(values)) => {
                    stack.extend(values.iter().rev().map(Item::Value))
                }
                Item::Value(Value::Object(values)) => {
                    for (key, value) in values.iter().rev() {
                        stack.push(Item::Value(value));
                        stack.push(Item::Key(key));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn text(&self, text: &str, found: &mut IndexMap<String, usize>) -> Result<()> {
        for (start, _) in text.match_indices("image-") {
            let Some(candidate) = text.get(start..start + 42) else {
                continue;
            };
            if !is_reference(candidate) {
                continue;
            }
            if !self.images.contains_key(candidate) {
                return Err(invalid(
                    "unknown image reference; attach its image to this request",
                ));
            }
            let count = found.entry(candidate.to_owned()).or_default();
            *count = count
                .checked_add(1)
                .ok_or_else(|| invalid("image reference count overflow"))?;
        }
        Ok(())
    }
}
fn is_reference(text: &str) -> bool {
    text.len() == 42
        && text.as_bytes()[6..].iter().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                *b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(b)
            }
        })
}

/// Build a frame array in caller-supplied oldest-first order and attach its images.
///
/// Timestamps are preserved, not sorted. Finite values are required by JSON;
/// negative and nonmonotonic values remain allowed like the Python reference.
/// Validation occurs before mutating the asset registry.
pub fn frames(
    assets: &mut Assets,
    images: impl IntoIterator<Item = Image>,
    seconds_ago: Option<&[f64]>,
) -> Result<State> {
    let images: Vec<_> = images.into_iter().collect();
    if images.is_empty() {
        return Err(invalid("frames needs at least one image"));
    }
    if let Some(times) = seconds_ago {
        if times.len() != images.len() {
            return Err(invalid("frames and seconds_ago lengths differ"));
        }
        if times.iter().any(|t| !t.is_finite()) {
            return Err(invalid("frame timestamps must be finite"));
        }
    }
    let frames = images
        .into_iter()
        .enumerate()
        .map(|(i, image)| {
            let mut value = serde_json::Map::new();
            if let Some(times) = seconds_ago {
                value.insert("seconds_ago".into(), Value::from(times[i]));
            }
            value.insert("frame".into(), Value::String(assets.attach(image)));
            Value::Object(value)
        })
        .collect();
    State::try_from(Value::Array(frames))
}
fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidImage, message)
}
fn be32(data: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(..4)?.try_into().ok()?))
}
fn be16(data: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes(data.get(..2)?.try_into().ok()?))
}
fn png(data: &[u8]) -> Result<(u32, u32)> {
    if data.len() < 33
        || data.get(12..16) != Some(b"IHDR".as_slice())
        || be32(&data[8..]) != Some(13)
    {
        return Err(invalid("malformed PNG: missing or invalid IHDR"));
    }
    let width = be32(&data[16..]).ok_or_else(|| invalid("malformed PNG width"))?;
    let height = be32(&data[20..]).ok_or_else(|| invalid("malformed PNG height"))?;
    let mut pos = 8usize;
    while pos < data.len() {
        let header = data
            .get(pos..pos.saturating_add(8))
            .ok_or_else(|| invalid("malformed PNG chunk header"))?;
        let length = be32(header).ok_or_else(|| invalid("malformed PNG chunk size"))? as usize;
        let end = pos
            .checked_add(12)
            .and_then(|p| p.checked_add(length))
            .ok_or_else(|| invalid("malformed PNG chunk overflow"))?;
        if end > data.len() {
            return Err(invalid("malformed PNG: truncated chunk"));
        }
        let kind = &header[4..8];
        if kind == b"acTL" {
            return Err(invalid("animated PNGs are not accepted"));
        }
        if kind == b"IDAT" || kind == b"IEND" {
            return Ok((width, height));
        }
        pos = end;
    }
    Err(invalid("malformed PNG: missing image data"))
}
fn jpeg(data: &[u8]) -> Result<(u32, u32, Vec<ImageDiagnostic>)> {
    let mut pos = 2usize;
    let mut diagnostics = Vec::new();
    while let Some(header) = data.get(pos..pos.saturating_add(2)) {
        if header[0] != 0xff {
            return Err(invalid("malformed JPEG: bad marker"));
        }
        let marker = header[1];
        if marker == 0xff {
            pos += 1;
            continue;
        }
        if matches!(marker, 0xd8 | 0x01 | 0xd0..=0xd7) {
            pos += 2;
            continue;
        }
        if matches!(marker, 0xd9 | 0xda) {
            break;
        }
        let length = data
            .get(pos + 2..)
            .and_then(be16)
            .ok_or_else(|| invalid("malformed JPEG segment length"))? as usize;
        if length < 2 {
            return Err(invalid("malformed JPEG segment length"));
        }
        let end = pos
            .checked_add(2)
            .and_then(|p| p.checked_add(length))
            .ok_or_else(|| invalid("malformed JPEG segment overflow"))?;
        let segment = data
            .get(pos + 4..end)
            .ok_or_else(|| invalid("malformed JPEG: truncated segment"))?;
        if marker == 0xe1 && segment.starts_with(b"Exif\0\0") {
            if let Some(orientation) = exif_orientation(&segment[6..]) {
                if orientation != 1 {
                    diagnostics.push(ImageDiagnostic::ExifOrientation(orientation));
                }
            }
        }
        if matches!(marker,0xc0..=0xc3|0xc5..=0xc7|0xc9..=0xcb|0xcd..=0xcf) {
            let height = segment
                .get(1..)
                .and_then(be16)
                .ok_or_else(|| invalid("malformed JPEG frame header"))?;
            let width = segment
                .get(3..)
                .and_then(be16)
                .ok_or_else(|| invalid("malformed JPEG frame header"))?;
            if !matches!(marker, 0xc0 | 0xc1) {
                diagnostics.push(ImageDiagnostic::NonBaselineJpeg);
            }
            return Ok((u32::from(width), u32::from(height), diagnostics));
        }
        pos = end;
    }
    Err(invalid("malformed JPEG: no frame header with size"))
}
fn exif_orientation(tiff: &[u8]) -> Option<u16> {
    let little = match tiff.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16_at = |pos: usize| -> Option<u16> {
        let bytes = tiff.get(pos..pos.checked_add(2)?)?.try_into().ok()?;
        Some(if little {
            u16::from_le_bytes(bytes)
        } else {
            u16::from_be_bytes(bytes)
        })
    };
    let bytes = tiff.get(4..8)?.try_into().ok()?;
    let offset = if little {
        u32::from_le_bytes(bytes)
    } else {
        u32::from_be_bytes(bytes)
    } as usize;
    let count = usize::from(u16_at(offset)?);
    for i in 0..count {
        let pos = offset.checked_add(2)?.checked_add(i.checked_mul(12)?)?;
        if u16_at(pos)? == 0x0112 {
            return u16_at(pos.checked_add(8)?);
        }
    }
    None
}
