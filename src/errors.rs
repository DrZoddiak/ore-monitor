use std::fmt::Display;

use reqwest::Error;

#[derive(Debug)]
pub(crate) enum OreError {
    SerializationError(serde_json::Error),
    ReqwestError(Error),
}

impl Display for OreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            self::OreError::SerializationError(err) => {
                writeln!(f, "Failed to serialize JSON : {}", err)
            }
            self::OreError::ReqwestError(err) => writeln!(f, "Request Error: {}", err),
            _ => writeln!(f, "Error not implemented!"),
        }
    }
}
