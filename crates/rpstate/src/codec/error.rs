use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodecError {
    #[cfg(feature = "json")]
    #[error("JSON codec error: {0}")]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "toml")]
    #[error("TOML error: {0}")]
    Toml(String),

    #[cfg(feature = "ron")]
    #[error("RON codec error: {0}")]
    Ron(#[from] ron::error::Error),

    #[cfg(feature = "redb")]
    #[error("MessagePack encode error: {0}")]
    MessagePackEncode(#[from] rmp_serde::encode::Error),

    #[cfg(feature = "redb")]
    #[error("MessagePack decode error: {0}")]
    MessagePackDecode(#[from] rmp_serde::decode::Error),

    #[error("Codec error: {0}")]
    Custom(String),
}
