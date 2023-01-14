use std::str::FromStr;

use http::StatusCode;
use serde::Serialize;

use super::Error;

#[derive(Serialize, Debug)]
pub struct Message {
    pub code: String,
    pub message: String,
    pub result: Option<String>,
}

impl Message {
    pub fn status_code(&self) -> Result<StatusCode, Error> {
        let codes: Vec<&str> = self.code.split('.').collect();
        if codes.len() != 3 {
            return Err(Error::Any(anyhow::anyhow! {"code's lenght isn't 3"}));
        }
        StatusCode::from_str(codes[1]).map_err(Error::any)
    }
}

impl From<Error> for Message {
    fn from(val: Error) -> Self {
        match val {
            Error::Any(err) => Message {
                code: "CIM.500.1010001".to_string(),
                message: "Internal Server Error".to_string(),
                result: Some(err.to_string()),
            },
            Error::NotFound(err) => Message {
                code: "CIM.404.1010002".to_string(),
                message: "Not Found".to_string(),
                result: Some(err),
            },
            Error::Forbidden(err) => Message {
                code: "CIM.403.1010003".to_string(),
                message: "Forbidden".to_string(),
                result: Some(err),
            },
            Error::Unauthorized => Message {
                code: "CIM.401.1010004".to_string(),
                message: Error::Unauthorized.to_string(),
                result: None,
            },
            Error::Validates(err) => Message {
                code: "CIM.422.1010005".to_string(),
                message: "Unprocessable Entity".to_string(),
                result: Some(err.to_string()),
            },
            Error::BadRequest(err) => Message {
                code: "CIM.400.1010007".to_string(),
                message: err,
                result: None,
            },
        }
    }
}

impl From<&Error> for Message {
    fn from(val: &Error) -> Self {
        val.to_owned().into()
    }
}
