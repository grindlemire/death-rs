use std::fmt::{Display, Formatter, Result};

// use thiserror::Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an error from the worker")]
    Worker,
    #[error("Timed out while waiting for {0} workers to close")]
    TimedOut(i32),

    #[error(transparent)]
    io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}

// #[derive(Debug)]
// pub struct CloseTimedOutError {
//     num_waiting: i32,
// }

// impl CloseTimedOutError {
//     pub fn new(waiting: i32) -> CloseTimedOutError {
//         CloseTimedOutError {
//             num_waiting: waiting,
//         }
//     }
// }

// impl Display for CloseTimedOutError {
//     fn fmt(&self, f: &mut Formatter) -> Result {
//         write!(
//             f,
//             "Timed out while waiting for {} workers to close",
//             self.num_waiting
//         )
//     }
// }

// impl Error for CloseTimedOutError {}

// #[derive(Debug)]
// pub struct DeathError {
//     cause: Box<dyn Error + Send + Sync>,
// }

// impl DeathError {
//     pub fn new(cause: Box<dyn Error + Send + Sync>) -> DeathError {
//         DeathError { cause }
//     }
// }

// impl Display for DeathError {
//     fn fmt(&self, f: &mut Formatter) -> Result {
//         write!(f, "Error from worker: {}", self.cause)
//     }
// }

// impl Error for DeathError {}
