//! Run `cargo run --features images --example images`: explicit assets and localization.
#[cfg(feature = "images")]
mod support;

#[cfg(not(feature = "images"))]
fn main() {
    println!("Run cargo run --features images --example images for the offline image fixture.");
}

#[cfg(feature = "images")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use semantic_operators::image::{Assets, Image};
    use semantic_operators::locate::{LocateOptions, locate};
    use semantic_operators::{Boolean, IndexMap, Request, State};
    use serde_json::json;
    use support::SyntheticProvider;
    futures_executor::block_on(async {
        // Structural PNG fixture: sufficient for header inspection, deliberately
        // not a pixel-decodable image and never transmitted to a service.
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend(13u32.to_be_bytes());
        bytes.extend(b"IHDR");
        bytes.extend(300u32.to_be_bytes());
        bytes.extend(300u32.to_be_bytes());
        bytes.extend([8, 0, 0, 0, 0]);
        bytes.extend([0; 4]);
        bytes.extend(0u32.to_be_bytes());
        bytes.extend(b"IDAT");
        bytes.extend([0; 4]);
        let image = Image::from_bytes(bytes)?;
        let mut assets = Assets::new();
        let photo = assets.attach(image);
        let state = State::try_from(json!({"photo":photo}))?;
        let request = Request::new(
            state,
            IndexMap::from([(
                "initial".into(),
                Boolean::new("Is there an object?")?.into(),
            )]),
        )?
        .with_assets(assets);
        let provider = SyntheticProvider::default();
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
        .await?;
        println!("OFFLINE STRUCTURAL FIXTURE — no pixels decoded and no model called.");
        println!("Found: {:?}; region: {:?}", located.found, located.region);
        println!(
            "Synthetic calls: {} (presence and Box together)",
            provider.calls()
        );
        Ok(())
    })
}
