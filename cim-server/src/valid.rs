use std::ops::Deref;

use async_trait::async_trait;
use axum::{
    extract::{FromRequest, FromRequestParts, Request, WebSocketUpgrade},
    Form, Json,
};
use http::{header, request::Parts, HeaderMap, HeaderName};
use serde::de::DeserializeOwned;
use validator::Validate;

use cim_slo::errors::{self, Code, WithBacktrace};

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
            .map_err(|err| errors::bad_request(&err))?;
        value.validate().map_err(Code::Validates)?;
        Ok(Self(value))
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for Valid<Json<T>>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = WithBacktrace;
    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let value = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| errors::bad_request(&err))?;
        value.deref().validate().map_err(Code::Validates)?;
        Ok(Self(value))
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for Valid<Form<T>>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = WithBacktrace;
    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let value = Form::<T>::from_request(req, state)
            .await
            .map_err(|err| errors::bad_request(&err))?;
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
                    errors::forbidden("miss request header X-Account-ID")
                })?
                .to_str()
                .unwrap_or_default()
                .to_string(),
            user_id: parts
                .headers
                .get("X-User-ID")
                .ok_or_else(|| {
                    errors::forbidden("miss request header X-User-ID")
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

pub enum ListWatch<T> {
    List(T),
    Ws((WebSocketUpgrade, T)),
}

#[async_trait]
impl<S, T> FromRequestParts<S> for ListWatch<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + Send,
{
    type Rejection = WithBacktrace;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Valid(param) = Valid::<T>::from_request_parts(parts, state).await?;
        if header_eq(&parts.headers, header::UPGRADE, "websocket")
            && header_contains(&parts.headers, header::CONNECTION, "upgrade")
        {
            let ws = WebSocketUpgrade::from_request_parts(parts, state)
                .await
                .map_err(errors::any)?;
            return Ok(Self::Ws((ws, param)));
        }
        Ok(Self::List(param))
    }
}

fn header_eq(
    headers: &HeaderMap,
    key: HeaderName,
    value: &'static str,
) -> bool {
    if let Some(header) = headers.get(&key) {
        header.as_bytes().eq_ignore_ascii_case(value.as_bytes())
    } else {
        false
    }
}

fn header_contains(
    headers: &HeaderMap,
    key: HeaderName,
    value: &'static str,
) -> bool {
    let header = if let Some(header) = headers.get(&key) {
        header
    } else {
        return false;
    };

    if let Ok(header) = std::str::from_utf8(header.as_bytes()) {
        header.to_ascii_lowercase().contains(value)
    } else {
        false
    }
}
