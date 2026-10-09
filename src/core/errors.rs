use hex::FromHexError;
use std::fmt;

#[derive(Debug, Clone)]
pub struct AppendToHistoryError;

impl fmt::Display for AppendToHistoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cannot append block to history")
    }
}

impl From<FromHexError> for AppendToHistoryError {
    fn from(_: FromHexError) -> AppendToHistoryError {
        AppendToHistoryError {}
    }
}

#[derive(Debug, Clone)]
pub struct TransactionValidationError;

impl fmt::Display for TransactionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Transaction failed to validate")
    }
}

#[derive(Debug, Clone)]
pub struct EmptySignatureError {
    msg: String,
}

impl EmptySignatureError {
    pub fn new(error_msg: String) -> EmptySignatureError {
        EmptySignatureError { msg: error_msg }
    }
}

impl fmt::Display for EmptySignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for EmptySignatureError {}

impl std::error::Error for AppendToHistoryError {}
impl std::error::Error for TransactionValidationError {}
