# Images and localization

Implemented 2026-10-17 behind the `images` feature. Image bytes are immutable and
owned with `Arc<[u8]>`. Clones retain identity. A request owns its `Assets`; no
global registry, file lookup, or URL fetch resolves a string automatically.
`Image::from_path` and `Image::from_base64` are explicit input operations.

## Validation and service differences

The library's `Image::from_bytes` preserves Python 0.12's local dimensional
contract: PNG/JPEG stills, at most 16 MiB, sides 256–1568 px, and area
78,400–2,458,624 px. `Assets::references` scans state followed by questions in
first-use order, including object keys, and counts every occurrence against
32 MiB while producing one upload reference per image. Unused attached images
are omitted. Unknown UUID-shaped image references fail locally.

Inspection checks signatures and bounded chunk/segment structure to find
sizes. It **does not decode pixels, validate PNG CRCs, or prove renderability**.
Malformed/truncated inspected chunks are rejected more strictly than Python's
header scanner. APNG control chunks before image data are rejected. EXIF
orientation and non-baseline JPEG produce inspectable diagnostics; no pixels are
changed. Service decoding can reject a file accepted by header inspection.

Current official [image documentation](https://docs.typesafe.ai/images), inspected
2026-10-17, differs from that frozen baseline: each side must be at least 280 px,
and references in object keys are not replaced. The TypeSafe-specific
`Assets::validate_service_request` applies these additional checks, rejecting
image references in object keys rather than silently treating them as pictures.
The service documents separate 32 MiB request-body and processed/expanded-image
limits. Local source-byte accounting cannot predict server re-encoding size;
body framing and text also consume request space. Local acceptance does not prove
hosted acceptance. Images and Box remain experimental in the official docs.

## Construction and ownership

Create an `Assets`, attach images, embed the returned strings in `State`, then
use `Request::with_assets`. References are process-unique counter-derived opaque
UUID-shaped tokens, not random UUIDs or persistent cross-process identifiers.
Do not use them as security capabilities. The request keeps images alive through
an awaited call. `Debug` omits raw bytes and asset references.

`frames(&mut assets, images, seconds_ago)` produces a `State` array with `frame`
and optional `seconds_ago` fields. It preserves caller order; callers supply
oldest-first order. Like Python, it does not sort, require nonnegative timestamps,
or reject nonmonotonic timestamps. Rust requires finite timestamps to represent
them in JSON. Empty inputs and mismatched lengths fail before assets are changed.

## Localization

`locate(provider, request, target, options)` preserves state/assets and replaces
questions with a Boolean `present` and a Box `box`, sent in one call. It returns
both original normalized answers and exposes a region only when presence is
confidently true and the Box is independently decided. A target that is absent,
a tied presence result, and insufficient presence confidence never expose a
region. Presence can be true while the Box threshold leaves region absent.

The official [Box documentation](https://docs.typesafe.ai/primitives/box), inspected
2026-10-17, requires an image, the experimental-images header, and Jev 1.14
(`jev-preview` or `jev-1.14.0`). Model aliases are not capability proofs; these
statements document the service snapshot, not a hard-coded permanent model list.
Offline tests exercise structure, byte limits, diagnostics, request ownership,
first-use order, frame validation, and every presence/Box decision combination.
No model quality or live image-upload qualification is claimed.
