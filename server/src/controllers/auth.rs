use axum::{
    extract::Query,
    routing::{get, post},
    Form, Json, Router,
};
use axum_extra::headers::authorization::{Basic, Bearer, Credentials};
use chrono::Utc;
use http::{
    header::{AUTHORIZATION, LOCATION},
    HeaderMap, StatusCode, Uri,
};
use pim::Request;
use tracing::info;

use slo::{errors, Result};

use crate::{services::authorization, valid::Valid, AppState};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        // .route("/auth", post(auth))
        // .route("/token", post(Self::token))
        // .route("/keys", get(Self::keys))
        // .route("/userinfo", get(Self::userinfo))
        .route("/authorize", post(authorize))
        // .route("/verify", get(Self::verify))
        // .route("/approval", get(Self::approval_html).post(Self::approval))
        // .route("/login.html", get(Self::porta))
        // .route("/oidc/authorize", post(Self::token))
        // .route("/provider", post(Self::create_provider))
        // .route("/auth/tokens", get(Self::token))
        // .route("/auth/:name/login", post(Self::login))
        .with_state(state)
}
//
// async fn auth(
//     _app: AppState,
//     header: HeaderMap,
//     Form(_body): Form<password::PasswordGrantOpts>,
// ) -> Result<(StatusCode, (HeaderMap, Json<()>))> {
//     Ok((StatusCode::OK, (header, Json(()))))
// }
//
// async fn token(
//     app: AppState,
//     header: HeaderMap,
//     Form(mut body): Form<password::PasswordGrantOpts>,
// ) -> Result<(StatusCode, (HeaderMap, Json<TokenResponse>))> {
//     match header.get(AUTHORIZATION) {
//         Some(value) => {
//             if let Some(v) = Basic::decode(value) {
//                 body.client_id = Some(v.username().to_owned());
//                 body.client_secret = Some(v.password().to_owned());
//             }
//         }
//         None => {
//             if body.client_id.is_none() || body.client_secret.is_none() {
//                 return Err(errors::bad_request(
//                     "client_id or client_secret is none",
//                 ));
//             }
//         }
//     }
//     let r = password::password_grant_token(&app, &body).await?;
//
//     let mut headers = HeaderMap::new();
//
//     headers.insert(
//         LOCATION,
//         format!("/v1/login.html?subject={}", 1).parse().unwrap(),
//     );
//
//     Ok((StatusCode::OK, (headers, r.into())))
// }
//
// async fn keys(_app: AppState) -> Result<(StatusCode, Json<()>)> {
//     Ok((StatusCode::OK, Json(())))
// }
//
// async fn userinfo(_app: AppState) -> Result<(StatusCode, Json<()>)> {
//     Ok((StatusCode::OK, Json(())))
// }

async fn authorize(
    app: AppState,
    Valid(Json(input)): Valid<Json<Request>>,
) -> Result<StatusCode> {
    info!("list query {:#?}", input);
    authorization::authorize(&app.store.policy, &app.matcher, &input).await?;
    Ok(StatusCode::NO_CONTENT)
}

// async fn verify(
//     app: AppState,
//     header: HeaderMap,
// ) -> Result<(StatusCode, Json<Claims>)> {
//     if let Some(value) = header.get(AUTHORIZATION) {
//         if let Some(v) = Bearer::decode(value) {
//             let token_handler =
//                 AccessToken::new(app.key_rotator.clone(), 0);
//             let claims = token_handler.verify(v.token()).await?;
//             return Ok((StatusCode::OK, claims.into()));
//         }
//     }
//     Err(errors::unauthorized())
// }
//
// async fn approval_html(
//     app: AppState,
//     Query(approval_info): Query<ApprovalInfo>,
// ) -> Result<HtmlTemplate<Approval>> {
//     info!("list query {:#?}", approval_info);
//     let auth_req = app.store.get_authrequests(&approval_info.req).await?;
//     if !auth_req.logged_in {
//         return Err(errors::anyhow(anyhow::anyhow!(
//             "Login process not yet finalized."
//         )));
//     }
//     if verify(&approval_info.hmac, &auth_req.id, &auth_req.hmac_key)? {
//         return Err(errors::unauthorized());
//     }
//     let provider = app.store.get_provider(&auth_req.client_id).await?;
//     Ok(HtmlTemplate(Approval {
//         req_path: Default::default(),
//         client: provider.name,
//         scopes: auth_req.scopes,
//         auth_req_id: approval_info.req,
//     }))
// }
//
// async fn approval(
//     app: AppState,
//     Query(approval_info): Query<ApprovalInfo>,
// ) -> Result<(StatusCode, HeaderMap)> {
//     info!("list query {:#?}", approval_info);
//     if approval_info.approval.unwrap_or_default().ne("approve") {
//         return Err(errors::anyhow(anyhow::anyhow!("Approval rejected.")));
//     }
//     let auth_req = app.store.get_authrequests(&approval_info.req).await?;
//     if !auth_req.logged_in {
//         return Err(errors::anyhow(anyhow::anyhow!(
//             "Login process not yet finalized."
//         )));
//     }
//     if verify(&approval_info.hmac, &auth_req.id, &auth_req.hmac_key)? {
//         return Err(errors::unauthorized());
//     }
//     if auth_req.expiry < Utc::now().timestamp() {
//         return Err(errors::anyhow(anyhow::anyhow!(
//             "User session has expired."
//         )));
//     }
//     app.store.delete_authrequests(&auth_req.id).await?;
//
//     let u: Uri = auth_req.redirect_url.parse().map_err(errors::any)?;
//     let mut url = u.to_string();
//     if u.query().is_none() {
//         url.push('?');
//     } else {
//         url.push('&');
//     }
//     for v in auth_req.response_types.iter() {
//         if v.eq("code") {
//             url.push_str(format!("code={}&state={}", 415, 45465).as_str());
//         }
//     }
//     let mut headers = HeaderMap::new();
//     headers.insert(LOCATION, url.parse().unwrap());
//
//     Ok((StatusCode::SEE_OTHER, headers))
// }
//
// async fn create_provider(
//     app: AppState,
//     Json(input): Json<Content>,
// ) -> Result<(StatusCode, Json<ID>)> {
//     info!("list query {:#?}", input);
//     let id = app.store.create_provider(&input).await?;
//     Ok((StatusCode::CREATED, id.into()))
// }
//
// async fn porta(
//     app: AppState,
//     Query(connector): Query<Connector>,
// ) -> Result<HtmlTemplate<Porta>> {
//     let mut p = Porta {
//         invalid: connector.invalid.unwrap_or_default(),
//         username: connector.subject.unwrap_or_default(),
//         ..Default::default()
//     };
//     for provider in app.store.list_provider().await? {
//         match &connector.client_id {
//             Some(v) => {
//                 if provider.id.eq(v) {
//                     p.prompt = provider.prompt;
//                     p.post_url =
//                         format!("/v1/auth?connector={}", provider.id)
//                 } else {
//                     p.connectors.push(ConnectorInfo {
//                         name: provider.name,
//                         logo: provider.logo_url,
//                         url: format!(
//                             "/v1/login.html?connector_id={}",
//                             provider.id
//                         ),
//                     })
//                 }
//             }
//             None => {
//                 if provider.name.eq("cim") {
//                     p.prompt = provider.prompt;
//                     p.post_url =
//                         format!("/v1/token?connector={}", provider.id)
//                 } else {
//                     p.connectors.push(ConnectorInfo {
//                         name: provider.name,
//                         logo: provider.logo_url,
//                         url: format!(
//                             "/v1/login.html?connector_id={}",
//                             provider.id,
//                         ),
//                     })
//                 }
//             }
//         }
//     }
//     Ok(HtmlTemplate(p))
// }
