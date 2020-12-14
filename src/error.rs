use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[derive(Debug)]
pub struct CloseTimedOutError {
    num_waiting: i32,
}

impl CloseTimedOutError {
    pub fn new(waiting: i32) -> CloseTimedOutError {
        CloseTimedOutError {
            num_waiting: waiting,
        }
    }
}

impl Display for CloseTimedOutError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "Timed out while waiting for {} workers to close",
            self.num_waiting
        )
    }
}

impl Error for CloseTimedOutError {}

#[derive(Debug)]
pub struct DeathError {
    cause: Box<dyn Error + Send + Sync>,
}

impl DeathError {
    pub fn new(cause: Box<dyn Error + Send + Sync>) -> DeathError {
        DeathError { cause }
    }
}

impl Display for DeathError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Error from worker: {}", self.cause)
    }
}

impl Error for DeathError {}
