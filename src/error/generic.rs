use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericError {
    details: String,
}

#[allow(dead_code)]
impl GenericError {
    pub fn new<S: Into<String>>(msg: S) -> GenericError {
        GenericError {
            details: msg.into(),
        }
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for GenericError {
    fn description(&self) -> &str {
        &self.details
    }
}
