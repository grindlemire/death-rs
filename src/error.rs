#[derive(thiserror::Error, Debug)]
pub enum Error {
    // #[error("incoming error")]
    // Incoming(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Worker failed to shut down cleanly: {0}")]
    Shutdown(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Timed out while waiting for {0} workers to close")]
    TimedOut(i32),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Channel(#[from] crossbeam_channel::RecvError),
}
