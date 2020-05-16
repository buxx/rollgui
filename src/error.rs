use std::error::Error;
use std::fmt;

use crate::server::client::ClientError;

#[derive(Debug)]
pub struct RollingError {
    pub message: String,
}

impl RollingError {
    pub fn new(message: String) -> RollingError {
        RollingError { message }
    }
}

impl Error for RollingError {}

impl fmt::Display for RollingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}", self.message)
    }
}

impl From<ClientError> for RollingError {
    fn from(error: ClientError) -> Self {
        return match error {
            ClientError::NotFound { response } => Self { message: response },
            ClientError::PlayerNotFound { response } => Self { message: response },
            ClientError::ClientSideError { response } => Self { message: response },
            ClientError::ServerSideError { response } => Self { message: response },
            ClientError::UnknownError { response } => Self { message: response },
        };
    }
}
