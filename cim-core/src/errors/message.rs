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
                code: "kuth.500.1010001".to_string(),
                message: "服务器内部错误".to_string(),
                result: Some(err.to_string()),
            },
            Error::NotFound(err) => Message {
                code: "kuth.404.1010002".to_string(),
                message: "资源不存在".to_string(),
                result: Some(err),
            },
            Error::Forbidden(err) => Message {
                code: "kuth.403.1010003".to_string(),
                message: "非法操作".to_string(),
                result: Some(err),
            },
            Error::Unauthorized => Message {
                code: "kuth.401.1010004".to_string(),
                message: Error::Unauthorized.to_string(),
                result: None,
            },
            Error::Validates(err) => Message {
                code: "kuth.422.1010005".to_string(),
                message: "请求参数不正确".to_string(),
                result: Some(err.to_string()),
            },
            Error::BadRequest(err) => Message {
                code: "kuth.400.1010007".to_string(),
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
