#[derive(thiserror::Error, Debug)]
pub enum PulseApplicationError {
    #[error("failed to open window: {0}")]
    FailedToOpenWindow(String),
}

#[derive(thiserror::Error, Debug)]
pub enum PulseError {
    #[error(transparent)]
    Application(#[from] PulseApplicationError),
}
