//! Optional hosted-provider integrations.
/// TypeSafe System One adapter and injectable transport.
#[cfg(feature = "typesafe")]
#[cfg_attr(docsrs, doc(cfg(feature = "typesafe")))]
pub mod typesafe;
