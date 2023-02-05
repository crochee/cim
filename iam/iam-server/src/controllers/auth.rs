use axum::{
    extract::Query,
    routing::{get, post},
    Extension, Form, Json, Router,
};
use chrono::Utc;
use http::{header::LOCATION, HeaderMap, StatusCode, Uri};

use tracing::info;

use cim_core::{Error, Result};

use crate::{
    models::{req::Request, ID},
    pkg::{
        security::{encrypt, verify},
        valid::Valid,
        HtmlTemplate,
    },
    repo::providers::Content,
    services::{
        authentication::{Info, Scopes},
        templates::{Approval, ApprovalInfo, Connector, ConnectorInfo, Porta},
        DynService,
    },
};

pub struct AuthRouter;

impl AuthRouter {
    pub fn new_router() -> Router {
        Router::new()
            .route("/auth", post(Self::authorize))
            .route("/approval", get(Self::approval_html).post(Self::approval))
            .route("/login.html", get(Self::porta))
            .route("/token", post(Self::token))
            .route("/provider", post(Self::create_provider))
        // .route("/auth/tokens", get(Self::token))
        // .route("/auth/:name/login", post(Self::login))
    }

    async fn authorize(
        Extension(srv): Extension<DynService>,
        Valid(Json(input)): Valid<Json<Request>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?}", input);
        srv.authorization().authorize(&input).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn approval_html(
        Extension(srv): Extension<DynService>,
        Query(approval_info): Query<ApprovalInfo>,
    ) -> Result<HtmlTemplate<Approval>> {
        info!("list query {:#?}", approval_info);
        let auth_req = srv
            .authentication()
            .authreq()
            .get(&approval_info.req)
            .await?;
        if !auth_req.logged_in {
            return Err(Error::Any(anyhow::anyhow!(
                "Login process not yet finalized."
            )));
        }
        if verify(&approval_info.hmac, &auth_req.id, &auth_req.hmac_key)? {
            return Err(Error::Unauthorized);
        }
        let provider = srv
            .authentication()
            .get_provider(&auth_req.client_id)
            .await?;
        Ok(HtmlTemplate(Approval {
            req_path: Default::default(),
            client: provider.name,
            scopes: auth_req.scopes,
            auth_req_id: approval_info.req,
        }))
    }

    async fn approval(
        Extension(srv): Extension<DynService>,
        Query(approval_info): Query<ApprovalInfo>,
    ) -> Result<(StatusCode, HeaderMap)> {
        info!("list query {:#?}", approval_info);
        if approval_info.approval.unwrap_or_default().ne("approve") {
            return Err(Error::Any(anyhow::anyhow!("Approval rejected.")));
        }
        let auth_req = srv
            .authentication()
            .authreq()
            .get(&approval_info.req)
            .await?;
        if !auth_req.logged_in {
            return Err(Error::Any(anyhow::anyhow!(
                "Login process not yet finalized."
            )));
        }
        if verify(&approval_info.hmac, &auth_req.id, &auth_req.hmac_key)? {
            return Err(Error::Unauthorized);
        }
        if auth_req.expiry < Utc::now().timestamp() {
            return Err(Error::Any(anyhow::anyhow!(
                "User session has expired."
            )));
        }
        srv.authentication().authreq().delete(&auth_req.id).await?;

        let u: Uri = auth_req.redirect_url.parse().map_err(Error::any)?;
        let mut url = u.to_string();
        if u.query().is_none() {
            url.push('?');
        } else {
            url.push('&');
        }
        for v in auth_req.response_types.iter() {
            if v.eq("code") {
                url.push_str(format!("code={}&state={}", 415, 45465).as_str());
            }
        }
        let mut headers = HeaderMap::new();
        headers.insert(LOCATION, url.parse().unwrap());

        Ok((StatusCode::SEE_OTHER, headers))
    }

    async fn create_provider(
        Extension(srv): Extension<DynService>,
        Json(input): Json<Content>,
    ) -> Result<(StatusCode, Json<ID>)> {
        info!("list query {:#?}", input);
        let id = srv.authentication().create_provider(&input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn porta(
        Extension(srv): Extension<DynService>,
        Query(connector): Query<Connector>,
    ) -> Result<HtmlTemplate<Porta>> {
        let mut p = Porta {
            invalid: connector.invalid.unwrap_or_default(),
            username: connector.subject.unwrap_or_default(),
            ..Default::default()
        };
        for provider in srv.authentication().providers().await? {
            match &connector.client_id {
                Some(v) => {
                    if provider.id.eq(v) {
                        p.prompt = provider.prompt;
                        p.post_url =
                            format!("/v1/auth?connector={}", provider.id)
                    } else {
                        p.connectors.push(ConnectorInfo {
                            name: provider.name,
                            logo: provider.logo_url,
                            url: format!(
                                "/v1/login.html?connector_id={}",
                                provider.id
                            ),
                        })
                    }
                }
                None => {
                    if provider.name.eq("cim") {
                        p.prompt = provider.prompt;
                        p.post_url =
                            format!("/v1/token?connector={}", provider.id)
                    } else {
                        p.connectors.push(ConnectorInfo {
                            name: provider.name,
                            logo: provider.logo_url,
                            url: format!(
                                "/v1/login.html?connector_id={}",
                                provider.id,
                            ),
                        })
                    }
                }
            }
        }
        Ok(HtmlTemplate(p))
    }

    async fn token(
        Extension(srv): Extension<DynService>,
        Query(s): Query<Scopes>,
        Form(info): Form<Info>,
    ) -> Result<(StatusCode, HeaderMap)> {
        info!("{:?}", info);
        let (identity, ok) = srv.authentication().login(&s, &info).await?;
        if !ok {
            let mut headers = HeaderMap::new();

            headers.insert(
                LOCATION,
                format!("/v1/login.html?subject={}", info.subject)
                    .parse()
                    .unwrap(),
            );

            return Ok((StatusCode::FOUND, headers));
        }

        let mut headers = HeaderMap::new();

        headers.insert(
            LOCATION,
            format!("/v1/login.html?subject={}", info.subject)
                .parse()
                .unwrap(),
        );

        Ok((StatusCode::SEE_OTHER, headers))
    }
    // async fn token(Path(name): Path<String>) -> Result<HtmlTemplate<Password>> {
    //     info!("{}", name);
    //     let t = Password {
    //         post_url: format!("http://127.0.0.1:30050/v1/auth/{}/login", name),
    //         back_link: "".to_owned(),
    //         username: "".to_owned(),
    //         prompt: "UserID".to_owned(),
    //         invalid: false,
    //         req_path: "".to_owned(),
    //     };
    //     Ok(HtmlTemplate(t))
    // }
    // async fn login(
    //     Extension(pool): Extension<MySqlPool>,
    //     Path(name): Path<String>,
    //     Query(scopes): Query<Scopes>,
    //     Valid(Form(info)): Valid<Form<Info>>,
    // ) -> Result<Any<Password>> {
    //     if name.eq("user") {
    //         let (identity, ok) =
    //             UserIDPassword::new(pool).login(&scopes, &info).await?;
    //         if !ok {
    //             return Err(Error::Forbidden(format!(
    //                 "Forbidden for {}",
    //                 info.subject
    //             )));
    //         }
    //         info!("{:?}", identity);
    //         // get token
    //         return Ok(Any::Header(HeaderMap::new()));
    //     }
    //     info!("{}", name);
    //     let t = Password {
    //         post_url: "http://127.0.0.1:5555/callback".to_owned(),
    //         back_link: "".to_owned(),
    //         username: info.subject,
    //         prompt: "UserID".to_owned(),
    //         invalid: false,
    //         req_path: "".to_owned(),
    //     };
    //     Ok(Any::Html(HtmlTemplate(t)))
    // }
}
