pub mod field;

use std::ops::Deref;

use async_trait::async_trait;
use axum::{
    body::HttpBody,
    extract::{FromRequest, FromRequestParts},
    Form, Json,
};
use http::{request::Parts, Request};
use serde::de::DeserializeOwned;
use validator::Validate;

use cim_core::{Code, WithBacktrace};

pub struct Valid<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Valid<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = WithBacktrace;
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or_default();
        let value: T = serde_urlencoded::from_str(query)
            .map_err(|err| Code::bad_request(&err))?;
        value.validate().map_err(Code::Validates)?;
        Ok(Self(value))
    }
}

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Valid<Json<T>>
where
    S: Send + Sync,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    T: DeserializeOwned + Validate,
{
    type Rejection = WithBacktrace;
    async fn from_request(
        req: Request<B>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let value = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| Code::bad_request(&err))?;
        value.deref().validate().map_err(Code::Validates)?;
        Ok(Self(value))
    }
}

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Valid<Form<T>>
where
    S: Send + Sync,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    T: DeserializeOwned + Validate,
{
    type Rejection = WithBacktrace;
    async fn from_request(
        req: Request<B>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let value = Form::<T>::from_request(req, state)
            .await
            .map_err(|err| Code::bad_request(&err))?;
        value.deref().validate().map_err(Code::Validates)?;
        Ok(Self(value))
    }
}

#[derive(Debug, Default, Validate)]
pub struct Header {
    #[validate(length(min = 1))]
    pub account_id: String,
    #[validate(length(min = 1))]
    pub user_id: String,
    #[validate(length(min = 1))]
    pub source: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for Header
where
    S: Send + Sync,
{
    type Rejection = WithBacktrace;
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut result = Header {
            account_id: parts
                .headers
                .get("X-Account-ID")
                .ok_or_else(|| {
                    Code::forbidden("miss request header X-Account-ID")
                })?
                .to_str()
                .unwrap_or_default()
                .to_string(),
            user_id: parts
                .headers
                .get("X-User-ID")
                .ok_or_else(|| {
                    Code::forbidden("miss request header X-User-ID")
                })?
                .to_str()
                .unwrap_or_default()
                .to_string(),
            ..Default::default()
        };

        if let Some(v) = parts.headers.get("X-Source") {
            result.source = Some(v.to_str().unwrap_or_default().to_owned());
        };
        result.validate().map_err(Code::Validates)?;
        Ok(result)
    }
}
