use std::collections::HashMap;

use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::{request::Parts, Method};
use validator::Validate;

use cim_pim::{Matcher, Pim, Statement};
use cim_slo::{
    errors::{self, Code, WithBacktrace},
    Result,
};
use cim_storage::{policy::StatementStore, user::User, Interface};

use crate::{
    valid::{ClientIp, Host},
    AppState,
};

#[derive(Validate, Debug)]
pub struct Info {
    pub user: User,
    statements: Vec<Statement>,
    req: cim_pim::Request,
}

#[async_trait]
impl<S> FromRequestParts<S> for Info
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

        let mut user = User {
            id: user_id.to_owned(),
            ..Default::default()
        };
        app.store.user.get(&mut user).await?;

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
        let subject = user.id.clone();
        let req = cim_pim::Request {
            resource,
            action: action.to_owned(),
            subject,
            context: HashMap::from([
                (
                    "client_ip".to_owned(),
                    serde_json::value::to_raw_value(&client_ip.ip.to_string())
                        .unwrap(),
                ),
                (
                    "host".to_owned(),
                    serde_json::value::to_raw_value(&host.host).unwrap(),
                ),
            ]),
        };
        // TODO:mutl statement source support
        let statements = app.store.statement.get_statement(&req).await?;

        let result = Self {
            user,
            statements,
            req,
        };
        result.validate().map_err(Code::Validates)?;
        Ok(result)
    }
}

impl Info {
    pub fn is_allow<M: Matcher>(
        &mut self,
        matcher: &Pim<M>,
        hash_map: HashMap<String, String>,
    ) -> Result<()> {
        for (k, v) in hash_map.iter() {
            self.req.context.insert(
                k.to_string(),
                serde_json::value::to_raw_value(v).unwrap(),
            );
        }
        matcher
            .is_allow(&self.statements, &self.req)
            .map_err(|err| errors::forbidden(err.to_string().as_str()))
    }
}

#[derive(Debug)]
pub struct Auth {
    pub user: User,
}

#[async_trait]
impl<S> FromRequestParts<S> for Auth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = WithBacktrace;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let info = Info::from_request_parts(parts, state).await?;
        if let Err(err) = AppState::from_ref(state)
            .matcher
            .is_allow(&info.statements, &info.req)
        {
            return Err(errors::forbidden(err.to_string().as_str()));
        }

        Ok(Self { user: info.user })
    }
}
