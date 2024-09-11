use std::fmt;
use std::fmt::Formatter;

/// define a retry error
#[derive(Debug)]
pub struct RetryError {
    /// error code
    code: i32,
    // error message
    message: String,
}

impl RetryError {
    /// new a default error with custom message
    pub fn new(message: &str) -> Self {
        RetryError {
            code: -1,
            message: message.to_string(),
        }
    }

    /// new error and specify error code and error message
    pub fn custom(code: i32, message: &str) -> Self {
        let message = message.to_string();
        RetryError {
            code,
            message,
        }
    }
}

impl fmt::Display for RetryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Err code: {}, Err message: {}", self.code, self.message)
    }
}

impl std::error::Error for RetryError {}