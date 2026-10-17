//! Offline image structure, ownership, and localization contracts.
#![cfg(feature = "images")]
use indexmap::IndexMap;
use semantic_operators::image::{
    Assets, Image, ImageDiagnostic, ImageFormat, MAX_FILE_BYTES, frames,
};
use semantic_operators::locate::{LocateOptions, locate};
use semantic_operators::{
    AnswerValue, Answers, Boolean, BoxQuestion, Choice, ErrorKind, Provider, ProviderFuture,
    Question, Region, Request, State, make_answer,
};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

// Structural test fixtures, intentionally not decoded by the header inspector.
fn chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = (payload.len() as u32).to_be_bytes().to_vec();
    bytes.extend(kind);
    bytes.extend(payload);
    bytes.extend([0; 4]);
    bytes
}
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = width.to_be_bytes().to_vec();
    header.extend(height.to_be_bytes());
    header.extend([8, 0, 0, 0, 0]);
    bytes.extend(chunk(b"IHDR", &header));
    bytes.extend(chunk(b"IDAT", &[]));
    bytes.extend(chunk(b"IEND", &[]));
    bytes
}
fn jpeg(width: u16, height: u16, marker: u8, orientation: Option<u16>, little: bool) -> Vec<u8> {
    let mut bytes = vec![0xff, 0xd8];
    if let Some(orientation) = orientation {
        let mut tiff = if little {
            b"II\x2a\0\x08\0\0\0".to_vec()
        } else {
            b"MM\0\x2a\0\0\0\x08".to_vec()
        };
        let enc = |n: u16| {
            if little {
                n.to_le_bytes()
            } else {
                n.to_be_bytes()
            }
        };
        tiff.extend(enc(1));
        tiff.extend(enc(0x112));
        tiff.extend(enc(3));
        tiff.extend(if little {
            1u32.to_le_bytes()
        } else {
            1u32.to_be_bytes()
        });
        tiff.extend(enc(orientation));
        tiff.extend([0; 6]);
        let mut exif = b"Exif\0\0".to_vec();
        exif.extend(tiff);
        bytes.extend([0xff, 0xe1]);
        bytes.extend(((exif.len() + 2) as u16).to_be_bytes());
        bytes.extend(exif);
    }
    bytes.extend([0xff, marker, 0, 11, 8]);
    bytes.extend(height.to_be_bytes());
    bytes.extend(width.to_be_bytes());
    bytes.extend([1, 1, 0x11, 0, 0xff, 0xd9]);
    bytes
}
fn image() -> Image {
    Image::from_bytes(png(300, 400)).unwrap()
}
fn questions() -> IndexMap<String, Question> {
    IndexMap::from([("q".into(), Boolean::new("Present?").unwrap().into())])
}

#[test]
fn header_formats_dimensions_and_frozen_boundaries() {
    let a = image();
    assert_eq!(a.format(), ImageFormat::Png);
    assert_eq!((a.width(), a.height()), (300, 400));
    let b = Image::from_bytes(jpeg(1568, 1568, 0xc0, None, false)).unwrap();
    assert_eq!(b.format().mime_type(), "image/jpeg");
    for (w, h) in [(280, 280), (256, 307), (1568, 1568)] {
        Image::from_bytes(png(w, h)).unwrap();
    }
    for (w, h) in [(255, 400), (400, 1569), (256, 256), (u32::MAX, u32::MAX)] {
        assert!(Image::from_bytes(png(w, h)).is_err());
    }
    let mut largest = png(300, 300);
    largest.resize(MAX_FILE_BYTES, 0);
    Image::from_bytes(largest.clone()).unwrap();
    largest.push(0);
    assert!(Image::from_bytes(largest).is_err());
}
#[test]
fn malformed_and_animated_images_are_rejected_without_panicking() {
    let mut animated = png(300, 300);
    animated.splice(33..33, chunk(b"acTL", &[0; 8]));
    assert!(
        Image::from_bytes(animated)
            .unwrap_err()
            .message()
            .contains("animated")
    );
    let mut truncated = png(300, 300);
    truncated[33..37].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(Image::from_bytes(truncated).is_err());
    for data in [png(300, 300), jpeg(300, 300, 0xc0, Some(6), false)] {
        // Every prefix ends before the first complete image data/frame segment.
        let boundary = if data.starts_with(b"\x89PNG") {
            45
        } else {
            data.len() - 2
        };
        for end in 0..boundary {
            assert!(
                Image::from_bytes(data[..end].to_vec()).is_err(),
                "prefix {end}"
            );
        }
    }
    for data in [
        vec![],
        b"GIF89a".to_vec(),
        vec![0xff, 0xd8, 0xff, 0xe1, 0, 0],
        vec![0xff, 0xd8, 0xff, 0xd9],
    ] {
        assert_eq!(
            Image::from_bytes(data).unwrap_err().kind(),
            ErrorKind::InvalidImage
        );
    }
}
#[test]
fn diagnostics_preserve_pixels_and_redact_bytes() {
    for little in [false, true] {
        let bytes = jpeg(300, 300, 0xc2, Some(6), little);
        let a = Image::from_bytes(bytes.clone()).unwrap();
        assert_eq!(a.bytes(), bytes);
        assert_eq!(
            a.diagnostics(),
            &[
                ImageDiagnostic::ExifOrientation(6),
                ImageDiagnostic::NonBaselineJpeg
            ]
        );
    }
    assert!(
        Image::from_bytes(jpeg(300, 300, 0xc1, Some(1), false))
            .unwrap()
            .diagnostics()
            .is_empty()
    );
    assert!(!format!("{:?}", image()).contains("data:"));
}
#[test]
fn explicit_base64_and_path_loading() {
    let a = image();
    let spaced = format!(" \n{}\t", a.base64());
    assert_eq!(Image::from_base64(&spaced).unwrap().bytes(), a.bytes());
    for invalid in ["data:image/png;base64,abcd", "not base64!", "===="] {
        assert!(Image::from_base64(invalid).is_err());
    }
    let path = std::env::temp_dir().join(format!(
        "semops-{}-{}.png",
        std::process::id(),
        a.placeholder()
    ));
    std::fs::write(&path, a.bytes()).unwrap();
    assert_eq!(Image::from_path(&path).unwrap().bytes(), a.bytes());
    std::fs::remove_file(path).unwrap();
    assert!(
        Image::from_path("/nonexistent/semops-image")
            .unwrap_err()
            .message()
            .contains("cannot open")
    );
}
#[test]
fn assets_keep_identity_lifetime_first_use_order_and_occurrences() {
    let a = image();
    let b = image();
    assert_ne!(a.placeholder(), b.placeholder());
    let mut assets = Assets::new();
    let a_id = assets.attach(a.clone());
    assert_eq!(assets.attach(a), a_id);
    let b_id = assets.attach(b);
    drop(assets.attach(image()));
    assert_eq!(assets.len(), 3);
    let state = State::try_from(json!({b_id.clone():format!("{a_id} {b_id} {a_id}")})).unwrap();
    let q = Boolean::new(format!("Is {a_id} red?"))
        .unwrap()
        .with_descriptions(Some(b_id.clone()), None);
    let refs = assets
        .references(&state, &IndexMap::from([("q".into(), q.into())]))
        .unwrap();
    assert_eq!(
        refs.iter()
            .map(|r| (r.image().placeholder(), r.uses()))
            .collect::<Vec<_>>(),
        vec![(b_id.as_str(), 3), (a_id.as_str(), 3)]
    );
    assert!(
        assets
            .validate_service_request(&state, &questions())
            .is_err()
    );
    assert!(Assets::new().validate(&state, &questions()).is_err());
    assert!(
        assets
            .references(
                &State::try_from(json!("image-like text and image-00000000")).unwrap(),
                &questions()
            )
            .unwrap()
            .is_empty()
    );
}
#[test]
fn question_descriptions_and_names_are_scanned() {
    let a = image();
    let mut assets = Assets::new();
    let id = assets.attach(a);
    let choices = IndexMap::from([(id.clone(), Some(id.clone())), ("other".into(), None)]);
    let q = Choice::new(format!("pick {id}"), choices).unwrap();
    let state = State::try_from(json!({"nested":[[id.clone()]]})).unwrap();
    let questions = IndexMap::from([(id.clone(), q.into())]);
    let refs = assets.references(&state, &questions).unwrap();
    assert_eq!(refs[0].uses(), 5);
}
#[test]
fn expanded_limit_counts_repeated_uses_but_not_unused_assets() {
    let mut data = png(300, 300);
    data.resize(MAX_FILE_BYTES, 0);
    let mut assets = Assets::new();
    let id = assets.attach(Image::from_bytes(data).unwrap());
    let two = State::try_from(json!([id.clone(), id.clone()])).unwrap();
    assert_eq!(assets.references(&two, &questions()).unwrap()[0].uses(), 2);
    let three = State::try_from(json!([id.clone(), id.clone(), id.clone()])).unwrap();
    assert!(assets.validate(&three, &questions()).is_err());
    assert!(
        assets
            .references(&State::try_from(json!("no image")).unwrap(), &questions())
            .unwrap()
            .is_empty()
    );
}
#[test]
fn service_preflight_is_separate_from_frozen_image_parity() {
    let a = Image::from_bytes(png(256, 307)).unwrap();
    let mut assets = Assets::new();
    let id = assets.attach(a);
    let state = State::try_from(json!({"photo":id})).unwrap();
    assets.validate(&state, &questions()).unwrap();
    assert!(
        assets
            .validate_service_request(&state, &questions())
            .is_err()
    );
    let mut assets = Assets::new();
    let id = assets.attach(image());
    let state = State::try_from(json!({"photo":id})).unwrap();
    assets
        .validate_service_request(&state, &questions())
        .unwrap();
}
#[test]
fn frames_preserve_order_timestamps_and_validate_before_attaching() {
    let mut assets = Assets::new();
    let a = image();
    let b = image();
    let state = frames(&mut assets, [a.clone(), b.clone()], Some(&[2.0, 0.0])).unwrap();
    assert_eq!(
        state.as_value(),
        &json!([{"seconds_ago":2.0,"frame":a.placeholder()},{"seconds_ago":0.0,"frame":b.placeholder()}])
    );
    let mut assets = Assets::new();
    assert!(frames(&mut assets, [a.clone()], Some(&[1.0, 0.0])).is_err());
    assert!(assets.is_empty());
    assert!(frames(&mut assets, [a.clone()], Some(&[f64::NAN])).is_err());
    assert!(assets.is_empty());
    assert!(frames(&mut assets, [], None).is_err());
    frames(&mut assets, [a, b], Some(&[-1.0, 2.0])).unwrap();
}

struct Localizer {
    p: f64,
    box_confidence: f64,
    calls: AtomicUsize,
}
impl Provider for Localizer {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(
                request
                    .questions()
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                ["present", "box"]
            );
            assert_eq!(
                request
                    .assets()
                    .references(request.state(), request.questions())?
                    .len(),
                1
            );
            let presence = make_answer(
                &request.questions()["present"],
                AnswerValue::Boolean(self.p >= 0.5),
                IndexMap::from([("true".into(), self.p), ("false".into(), 1.0 - self.p)]),
                None,
            )?;
            let region = make_answer(
                &request.questions()["box"],
                AnswerValue::Box(Region::new(0.1, 0.8, 0.2, 0.7)?),
                IndexMap::new(),
                Some(self.box_confidence),
            )?;
            Ok(Answers::from([
                ("present".into(), presence),
                ("box".into(), region),
            ]))
        })
    }
}
#[test]
fn locate_requires_presence_and_independent_box_threshold_in_one_call() {
    futures_executor::block_on(async {
        let mut assets = Assets::new();
        let id = assets.attach(image());
        let request = Request::new(State::try_from(json!({"photo":id})).unwrap(), questions())
            .unwrap()
            .with_assets(assets);
        for (p, box_conf, found, has_region) in [
            (0.9, 0.9, Some(true), true),
            (0.1, 0.9, Some(false), false),
            (0.5, 0.9, None, false),
            (0.7, 0.9, None, false),
            (0.9, 0.2, Some(true), false),
        ] {
            let provider = Localizer {
                p,
                box_confidence: box_conf,
                calls: AtomicUsize::new(0),
            };
            let located = locate(
                &provider,
                &request,
                "a mug",
                LocateOptions {
                    where_: "`photo`".into(),
                    min_presence: 0.8,
                    min_box_confidence: 0.5,
                },
            )
            .await
            .unwrap();
            assert_eq!(located.found, found);
            assert_eq!(located.region.is_some(), has_region);
            assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
            assert_eq!(located.box_answer.confidence(), box_conf);
        }
        let provider = Localizer {
            p: 0.9,
            box_confidence: 0.9,
            calls: AtomicUsize::new(0),
        };
        assert!(
            locate(&provider, &request, " ", LocateOptions::default())
                .await
                .is_err()
        );
        assert!(
            locate(
                &provider,
                &request,
                "a mug",
                LocateOptions {
                    min_presence: f64::NAN,
                    ..Default::default()
                }
            )
            .await
            .is_err()
        );
        assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
        assert!(matches!(
            BoxQuestion::new("a mug").unwrap().into(),
            Question::Box(_)
        ));
    });
}

#[test]
fn arbitrary_truncated_headers_and_exif_offsets_never_panic() {
    // Deterministic malformed input exercise: varied marker/chunk lengths and
    // truncated TIFF offsets cover checked slicing without a decoder dependency.
    for length in [0u32, 1, 2, 3, 8, 13, 255, 65535, u32::MAX] {
        let mut bytes = png(300, 300);
        bytes[8..12].copy_from_slice(&length.to_be_bytes());
        let _ = Image::from_bytes(bytes);
        let mut bytes = png(300, 300);
        bytes[33..37].copy_from_slice(&length.to_be_bytes());
        let _ = Image::from_bytes(bytes);
    }
    for length in [0u16, 1, 2, 3, 4, 65535] {
        let mut bytes = jpeg(300, 300, 0xc0, Some(6), false);
        bytes[4..6].copy_from_slice(&length.to_be_bytes());
        let _ = Image::from_bytes(bytes);
    }
    let mut bytes = jpeg(300, 300, 0xc0, Some(6), false);
    bytes[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    let parsed = Image::from_bytes(bytes).unwrap();
    assert!(parsed.diagnostics().is_empty());
}
