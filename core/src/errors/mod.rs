use std::{error::Error, fmt, str::FromStr};

use anyhow::Context;
use axum::{
    body::{self, Full},
    response::IntoResponse,
};
use backtrace::Backtrace;
use http::{header, HeaderValue, Response, StatusCode};
use serde_json::{json, to_vec};

pub trait ErrorCode: Error + 'static {
    fn code(&self) -> &'static str;
}

#[derive(thiserror::Error, Debug)]
pub enum Code {
    #[error(transparent)]
    Any(#[from] anyhow::Error),
    #[error("Not found. {0}")]
    NotFound(String),
    #[error("Forbidden. {0}")]
    Forbidden(String),
    #[error("Authentication is required to access this resource")]
    Unauthorized,
    #[error("Please recheck the request.see: {0}")]
    Validates(#[source] validator::ValidationErrors),
    #[error("Please recheck the request.see: {0}")]
    BadRequest(String),
}

impl ErrorCode for Code {
    fn code(&self) -> &'static str {
        match self {
            Self::Any(_) => "Cim.500.1010001",
            Self::NotFound(_) => "Cim.404.1010002",
            Self::Unauthorized => "Cim.401.1010003",
            Self::Validates(_) => "Cim.422.1010004",
            Self::Forbidden(_) => "Cim.403.1010005",
            Self::BadRequest(_) => "Cim.400.1010006",
        }
    }
}

impl Code {
    pub fn any<E: Error>(err: E) -> WithBacktrace {
        WithBacktrace {
            code: Code::Any(anyhow::anyhow!("{}", err)),
            backtrace: Backtrace::new(),
        }
    }

    pub fn with(self) -> WithBacktrace {
        WithBacktrace {
            code: self,
            backtrace: Backtrace::new(),
        }
    }

    pub fn not_found<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
        WithBacktrace {
            code: Code::NotFound(err.to_string()),
            backtrace: Backtrace::new(),
        }
    }

    pub fn forbidden<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
        WithBacktrace {
            code: Code::Forbidden(err.to_string()),
            backtrace: Backtrace::new(),
        }
    }
    pub fn bad_request<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
        WithBacktrace {
            code: Code::BadRequest(err.to_string()),
            backtrace: Backtrace::new(),
        }
    }
}

pub struct WithBacktrace {
    code: Code,
    backtrace: Backtrace,
}

impl fmt::Debug for WithBacktrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?} {:?}", self.code, self.backtrace)
    }
}

impl fmt::Display for WithBacktrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl Error for WithBacktrace {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.code)
    }
}

impl From<Code> for WithBacktrace {
    fn from(code: Code) -> Self {
        WithBacktrace {
            code,
            backtrace: Backtrace::new(),
        }
    }
}

impl PartialEq for WithBacktrace {
    fn eq(&self, other: &Self) -> bool {
        self.code.code() == other.code.code()
    }
}

impl IntoResponse for WithBacktrace {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);

        let codes: Vec<&str> = self.code.code().split('.').collect();
        let status_code: Result<StatusCode, anyhow::Error> = (|| {
            if codes.len() != 3 {
                return Err(anyhow::anyhow! {"code's lenght isn't 3"});
            }
            StatusCode::from_str(codes[1]).context("could not parse from str")
        })();

        let code = match status_code {
            Ok(v) => v,
            Err(err) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(
                            mime::TEXT_PLAIN_UTF_8.as_ref(),
                        ),
                    )
                    .body(body::boxed(Full::from(err.to_string())))
                    .unwrap();
            }
        };
        let bytes = match to_vec(&json!({
            "code": self.code.code(),
            "message": self.to_string(),
        })) {
            Ok(res) => res,
            Err(err) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(
                            mime::TEXT_PLAIN_UTF_8.as_ref(),
                        ),
                    )
                    .body(body::boxed(Full::from(err.to_string())))
                    .unwrap();
            }
        };

        let mut res = Response::new(body::boxed(Full::from(bytes)));
        *res.status_mut() = code;
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );
        res
    }
}
