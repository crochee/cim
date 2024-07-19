use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    ops::Deref,
};

use async_trait::async_trait;
use axum::{
    extract::{
        ConnectInfo, FromRef, FromRequest, FromRequestParts, Request,
        WebSocketUpgrade,
    },
    Form, Json,
};
use http::{header, request::Parts, HeaderMap, HeaderName, Method};
use serde::de::DeserializeOwned;
use validator::Validate;

use cim_pim::{Matcher, Pim, Statement};
use cim_slo::errors::{self, Code, WithBacktrace};
use cim_storage::{policy::StatementStore, user::User, Interface};

use crate::AppState;

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

#[derive(Validate, Debug)]
pub struct Header {
    pub user: User,
    pub host: String,
    pub client_ip: IpAddr,
    list: Vec<Statement>,
    req: cim_pim::Request,
}

#[async_trait]
impl<S> FromRequestParts<S> for Header
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = WithBacktrace;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, state).await?;
        let host = Host::from_request_parts(parts, state).await?;
        let client_ip = ClientIp::from_request_parts(parts, state).await?;
        let action = match parts.method {
            Method::POST => "create",
            Method::GET => "get",
            Method::PUT => "update",
            Method::PATCH => "patch",
            Method::DELETE => "delete",
            _ => "",
        };
        let mut path =
            parts.uri.path().trim_start_matches("/v").split('/').skip(1);
        let mut resource = String::from("crn:iam:");
        if let Some(v) = path.next() {
            resource.push_str(v.trim_end_matches('s'));
        };
        if let Some(v) = path.next() {
            resource.push(':');
            resource.push_str(v);
        }
        let subject = auth.user.id.clone();
        let mut result = Header {
            user: auth.user,
            host: host.host,
            client_ip: client_ip.ip,
            list: Vec::new(),
            req: cim_pim::Request {
                resource,
                action: action.to_owned(),
                subject,
                context: HashMap::from([(
                    "client_ip".to_owned(),
                    serde_json::value::to_raw_value(&client_ip.ip.to_string())
                        .unwrap(),
                )]),
            },
        };
        let app = AppState::from_ref(state);
        // TODO:mutl statement source support
        result.list = app.store.policy.get_statement(&result.req).await?;
        result.validate().map_err(Code::Validates)?;
        Ok(result)
    }
}

impl Header {
    pub fn is_allow<M: Matcher>(
        &self,
        matcher: &Pim<M>,
        hash_map: HashMap<String, String>,
    ) -> bool {
        let mut req = self.req.clone();
        hash_map.iter().for_each(|(k, v)| {
            req.context.insert(
                k.to_string(),
                serde_json::value::to_raw_value(v).unwrap(),
            );
        });
        matcher.is_allow(self.list.clone(), &req).is_ok()
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

pub struct Host {
    pub host: String,
}

#[async_trait]
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

#[async_trait]
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
                match ConnectInfo::<SocketAddr>::from_request_parts(
                    parts, &state,
                )
                .await
                {
                    Ok(v) => v.ip(),
                    Err(_) => "0.0.0.0:0".parse().unwrap(),
                }
            }
        };

        Ok(Self { ip })
    }
}

pub struct AuthUser {
    pub user: User,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = WithBacktrace;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // TODO: check token  and parse token  replace x-user-id
        let app = AppState::from_ref(state);
        let user_id = parts
            .headers
            .get("X-User-ID")
            .ok_or_else(|| errors::forbidden("missing X-User-ID"))?
            .to_str()
            .unwrap_or_default();
        let mut user = User::default();
        app.store.user.get(user_id, &mut user).await?;

        Ok(Self { user })
    }
}
