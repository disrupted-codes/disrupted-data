use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct DisruptedDataError {
    pub message: String,
}

impl fmt::Display for DisruptedDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for DisruptedDataError {}
