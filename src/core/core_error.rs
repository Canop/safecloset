/// Core error type
#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Passphrase too short")]
    PasswordTooShort,

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("AEAD error")]
    Aead, // The AEAD error type is opaque

    #[error("File {0} already exists")]
    FileExists(std::path::PathBuf),

    #[error("Argon2 password hash error: {0}")]
    Argon2(#[from] argon2::Error),

    #[error("Attempt to close a drawer which isn't the last open one")]
    WrongOpenId,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Passphrase already used for an existing drawer")]
    PasswordAlreadyUsed,
}
