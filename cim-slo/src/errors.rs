use std::{error::Error as StdError, fmt};

use backtrace::Backtrace;
use http::StatusCode;
use thiserror::Error;

pub trait ErrorCode: StdError + 'static {
    fn code(&self) -> (StatusCode, &'static str);
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
    fn code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::Any(_) => (StatusCode::INTERNAL_SERVER_ERROR, "1010001"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "1010002"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "1010003"),
            Self::Validates(_) => (StatusCode::UNPROCESSABLE_ENTITY, "1010004"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "1010005"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "1010006"),
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
    fn from(code: Code) -> Self {
        WithBacktrace {
            source: code,
            backtrace: Backtrace::new(),
        }
    }
}

impl From<WithBacktrace> for Code {
    fn from(value: WithBacktrace) -> Self {
        value.source
    }
}

impl PartialEq for WithBacktrace {
    fn eq(&self, other: &Self) -> bool {
        let (_, src_code) = self.source.code();
        let (_, dst_code) = other.source.code();
        src_code == dst_code
    }
}

#[inline]
pub fn any<E: StdError>(err: E) -> WithBacktrace {
    WithBacktrace {
        source: Code::Any(anyhow::anyhow!("{}", err.to_string())),
        backtrace: Backtrace::new(),
    }
}

#[inline]
pub fn anyhow(err: anyhow::Error) -> WithBacktrace {
    WithBacktrace {
        source: Code::Any(err),
        backtrace: Backtrace::new(),
    }
}

#[inline]
pub fn not_found<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
    WithBacktrace {
        source: Code::NotFound(err.to_string()),
        backtrace: Backtrace::new(),
    }
}

#[inline]
pub fn forbidden<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
    WithBacktrace {
        source: Code::Forbidden(err.to_string()),
        backtrace: Backtrace::new(),
    }
}

#[inline]
pub fn unauthorized() -> WithBacktrace {
    WithBacktrace {
        source: Code::Unauthorized,
        backtrace: Backtrace::new(),
    }
}

#[inline]
pub fn bad_request<S: ToString + ?Sized>(err: &S) -> WithBacktrace {
    WithBacktrace {
        source: Code::BadRequest(err.to_string()),
        backtrace: Backtrace::new(),
    }
}

#[cfg(feature = "axum-resp")]
mod axum {
    use axum::response::IntoResponse;
    use serde_json::json;

    use super::ErrorCode;

    impl IntoResponse for super::WithBacktrace {
        fn into_response(self) -> axum::response::Response {
            tracing::error!("{:?}", self);

            let (status_code, code) = self.source.code();

            let payload = json!({
                "code": code,
                "message": self.to_string(),
            });

            (status_code, axum::Json(payload)).into_response()
        }
    }
}
