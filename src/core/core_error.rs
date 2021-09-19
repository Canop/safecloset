/// Core error type
#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Passphrase too short")]
    PasswordTooShort,

    #[error("MessagePack Encode error: {0}")]
    MessagePackEncode(#[from] rmp_serde::encode::Error),

    #[error("MessagePack Decode error: {0}")]
    MessagePackDecode(#[from] rmp_serde::decode::Error),

    #[error("AEAD error")]
    Aead, // The AEAD error type is opaque

    #[error("File {0} already exists")]
    FileExists(std::path::PathBuf),

    #[error("Argon2 password hash error: {0}")]
    Argon2(#[from] argon2::Error),

    #[error("Unconsistent data")]
    UnconsistentData,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Passphrase already used for an existing drawer")]
    PasswordAlreadyUsed,

    #[error("No open drawer")]
    NoOpenDrawer,

    #[error("Invalid Push Back")]
    InvalidPushBack,
}
