use std::ops::Deref;

use async_trait::async_trait;
use axum::{
    body::HttpBody,
    extract::{FromRequest, FromRequestParts, Query},
    Json,
};
use http::{request::Parts, Request};
use serde::de::DeserializeOwned;
use validator::Validate;

use cim_core::Error;

pub struct Valid<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Valid<Query<T>>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = Error;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let value = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|err| Error::BadRequest(err.to_string()))?;
        value.deref().validate().map_err(Error::Validates)?;
        Ok(Self(value))
    }
}

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Valid<Json<T>>
where
    S: Send + Sync,
    B: Send + 'static,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    T: DeserializeOwned + Validate,
{
    type Rejection = Error;
    async fn from_request(
        req: Request<B>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let value = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| Error::BadRequest(err.to_string()))?;
        value.deref().validate().map_err(Error::Validates)?;
        Ok(Self(value))
    }
}

#[derive(Debug, Default)]
pub struct Header {
    pub account_id: String,
    pub user_id: String,
    pub source: Option<String>,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for Header
where
    S: Send + Sync,
    B: Send + 'static,
{
    type Rejection = Error;

    async fn from_request(
        req: Request<B>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut result = Header::default();
        let header = req.headers();
        result.account_id = header
            .get("X-Account-ID")
            .ok_or_else(|| {
                Error::Forbidden("miss request header X-Account-ID".to_string())
            })?
            .to_str()
            .unwrap_or_default()
            .to_string();
        result.user_id = header
            .get("X-User-ID")
            .ok_or_else(|| {
                Error::Forbidden("miss request header X-User-ID".to_string())
            })?
            .to_str()
            .unwrap_or_default()
            .to_string();

        if let Some(v) = header.get("X-Source") {
            result.source = Some(v.to_str().unwrap_or_default().to_owned());
        };
        Ok(result)
    }
}
