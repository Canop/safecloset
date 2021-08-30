/// SafeCloset error type
#[derive(thiserror::Error, Debug)]
pub enum SafeClosetError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Core error: {0}")]
    Core(#[from] crate::core::CoreError),

    #[error("Termimad error: {0}")]
    Termimad(#[from] termimad::Error),

    #[error("Crossbeam channel error: {0}")]
    Crossbeam(#[from] crossbeam::channel::RecvError),

    #[error("Internal error: {0}")]
    Internal(String),
}
