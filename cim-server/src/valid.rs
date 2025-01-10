use std::{
    net::{IpAddr, SocketAddr},
    ops::Deref,
};

use axum::{
    extract::{
        ConnectInfo, FromRequest, FromRequestParts, Request, WebSocketUpgrade,
    },
    Form, Json,
};
use http::{header, request::Parts, HeaderMap, HeaderName};
use serde::{de::DeserializeOwned, Deserialize};
use validator::Validate;

use cim_slo::errors::{self, Code, WithBacktrace};

pub struct Valid<T>(pub T);

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

#[derive(Deserialize, Validate)]
pub struct ListWatchParam<T> {
    pub watch: Option<bool>,
    #[serde(flatten)]
    pub param: T,
}

pub enum ListWatch<T> {
    List(T),
    Watch(T),
    Ws((WebSocketUpgrade, T)),
}

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
        let Valid(param) =
            Valid::<ListWatchParam<T>>::from_request_parts(parts, state)
                .await?;
        if header_eq(&parts.headers, header::UPGRADE, "websocket")
            && header_contains(&parts.headers, header::CONNECTION, "upgrade")
        {
            let ws = WebSocketUpgrade::from_request_parts(parts, state)
                .await
                .map_err(errors::any)?;
            return Ok(Self::Ws((ws, param.param)));
        }
        if param.watch.unwrap_or_default() {
            return Ok(Self::Watch(param.param));
        }
        Ok(Self::List(param.param))
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

pub struct Host {
    pub host: String,
}

impl<S> FromRequestParts<S> for Host
where
    S: Send + Sync,
{
    type Rejection = WithBacktrace;
    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let host = if let Some(referer) = parts.headers.get(header::REFERER) {
            referer.to_str().unwrap().to_string()
        } else {
            let protocol = if let Some(protocol) =
                parts.headers.get("X-Forwarded-Proto")
            {
                protocol.to_str().unwrap()
            } else {
                "http"
            };
            let host = if let Some(host) = parts.headers.get("X-Forwarded-Host")
            {
                host.to_str().unwrap()
            } else {
                parts.headers.get(header::HOST).unwrap().to_str().unwrap()
            };
            format!("{protocol}://{host}")
        };
        Ok(Self { host })
    }
}

pub struct ClientIp {
    pub ip: IpAddr,
}

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = WithBacktrace;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let ip = if let Some(real_ip) = parts.headers.get("X-Real-IP") {
            let ip = real_ip.to_str().unwrap_or_default();
            match ip.find(',') {
                Some(idx) => &ip[..idx],
                None => ip,
            }
            .parse()
            .ok()
        } else {
            None
        };
        let ip = match ip {
            Some(v) => v,
            None => {
                let v = ConnectInfo::<SocketAddr>::from_request_parts(
                    parts, &state,
                )
                .await
                .map_err(errors::any)?;
                v.ip()
            }
        };

        Ok(Self { ip })
    }
}
