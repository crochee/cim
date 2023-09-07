use axum::routing::post;
use axum::Router;

use crate::AppState;

pub struct AuthRouter;

impl AuthRouter {
    pub fn new_router(state: AppState) -> Router {
        Router::new()
            .route("/auth", post(Self::auth))
            .route("/token", post(Self::token))
            .route("/keys", get(Self::keys))
            .route("/userinfo", get(Self::userinfo))
            // .route("/auth/tokens", get(Self::token))
            // .route("/auth/:name/login", post(Self::login))
            .with_state(state)
    }

    pub async fn auth() {}
    pub async fn token() {}
    pub async fn keys() {}
    pub async fn userinfo() {}
}
