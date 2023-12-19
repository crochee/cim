#![allow(unused)]
#![feature(backtrace)]

use std::{error::Error as StdError, fmt, str::FromStr};

use anyhow::Context;
use axum::{response::IntoResponse, Json};
use backtrace::Backtrace;
use http::{header, HeaderValue, Response, StatusCode};
use serde_json::json;
use thiserror::Error;

pub trait ErrorCode: StdError + 'static {
    fn code(&self) -> &'static str;
}

#[derive(Error, Debug)]
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

pub struct WithBacktrace {
    source: Code,
    backtrace: Backtrace,
}

impl fmt::Debug for WithBacktrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WithBacktrace")
            .field("source", &self.source)
            .field("backtrace", &self.backtrace)
            .finish()
    }
}

impl fmt::Display for WithBacktrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

impl StdError for WithBacktrace {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}

impl From<Code> for WithBacktrace {
    fn from(source: Code) -> Self {
        WithBacktrace {
            source,
            backtrace: Backtrace::new(),
        }
    }
}

impl PartialEq for WithBacktrace {
    fn eq(&self, other: &Self) -> bool {
        self.source.code() == other.source.code()
    }
}

impl IntoResponse for WithBacktrace {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);

        let codes: Vec<&str> = self.source.code().split('.').collect();
        let status_code: Result<StatusCode, anyhow::Error> = (|| {
            if codes.len() != 3 {
                return Err(anyhow::anyhow! {"code's lenght isn't 3"});
            }
            StatusCode::from_str(codes[1]).context("could not parse from str")
        })();

        let code = match status_code {
            Ok(v) => v,
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(
                            mime::TEXT_PLAIN_UTF_8.as_ref(),
                        ),
                    )],
                    err.to_string(),
                )
                    .into_response();
            }
        };
        let payload = json!({
            "code": self.source.code(),
            "message": self.to_string(),
        });

        (code, axum::Json(payload)).into_response()
    }
}

pub fn any<E: StdError>(err: E) -> WithBacktrace {
    WithBacktrace {
        source: Code::Any(anyhow::anyhow!("{}", err)),
        backtrace: Backtrace::new(),
    }
}

pub fn not_found<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
    WithBacktrace {
        source: Code::NotFound(err.to_string()),
        backtrace: Backtrace::new(),
    }
}

pub fn forbidden<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
    WithBacktrace {
        source: Code::Forbidden(err.to_string()),
        backtrace: Backtrace::new(),
    }
}
pub fn bad_request<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
    WithBacktrace {
        source: Code::BadRequest(err.to_string()),
        backtrace: Backtrace::new(),
    }
}
