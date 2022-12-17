mod message;

use axum::{
    body::{self, Full},
    response::{IntoResponse, Response},
};

use http::{header, HeaderValue, StatusCode};

use message::Message;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Any(#[from] anyhow::Error),
    #[error("{0} isn't found")]
    NotFound(String),
    #[error("{0}")]
    Validates(#[source] validator::ValidationErrors),
    #[error("{0}")]
    Forbidden(String),
    #[error("authentication is required to access this resource")]
    Unauthorized,
    #[error("{0}")]
    BadRequest(String),
}

impl Error {
    #[inline]
    pub fn any<E>(err: E) -> Self
    where
        E: std::error::Error,
    {
        Self::Any(anyhow::format_err!("{}", err))
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        let content: Message = self.into();
        let other_content: Message = other.into();
        content.code == other_content.code
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let content: Message = self.into();
        tracing::error!("{:?}", content);
        let code = match content.status_code() {
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
        let bytes = match serde_json::to_vec(&content) {
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
