use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct RollingError {
    pub message: String,
}

impl RollingError {
    pub fn new(message: String) -> RollingError {
        RollingError { message }
    }
}

impl Error for RollingError {

}

impl fmt::Display for RollingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}", self.message)
    }
}
