use std::time::{Duration, Instant};

use anyhow::Result;
use axum::{
    body::Body,
    extract::{MatchedPath, Request},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use http::{
    header::{HeaderName, CONTENT_TYPE},
    HeaderValue, Uri,
};
use prometheus::{Encoder, TextEncoder};
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, CorsLayer, ExposeHeaders},
    services::ServeDir,
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use cim_slo::errors;

use crate::{
    controllers::{
        auth, group_users, groups, oidc, policies, policy_bindings,
        role_bindings, roles, users,
    },
    middlewares::MakeSpanWithTrace,
    var::{HTTP_REQUESTS_DURATION_SECONDS, HTTP_REQUESTS_TOTAL},
    AppState,
};

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "/v1", description = "Local server"),
    ),
)]
struct ApiDoc;

pub struct AppRouter;

impl AppRouter {
    pub fn build(state: AppState) -> Result<Router> {
        let cors_origin = state.config.cors_origin.parse::<HeaderValue>()?;

        let router = Router::new()
            .nest_service("/static", ServeDir::new("static"))
            .nest_service("/theme", ServeDir::new("theme"))
            .merge(oidc::op::new_router(state.clone()))
            .merge(oidc::eu::new_router(state.clone()))
            .merge(
                SwaggerUi::new("/swagger-ui")
                    .url("/v1/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .nest(
                "/v1",
                Router::new()
                    .merge(policies::new_router(state.clone()))
                    .merge(auth::new_router(state.clone()))
                    .merge(users::new_router(state.clone()))
                    .merge(roles::new_router(state.clone()))
                    .merge(group_users::new_router(state.clone()))
                    .merge(role_bindings::new_router(state.clone()))
                    .merge(policy_bindings::new_router(state.clone()))
                    .merge(groups::new_router(state)),
            )
            .layer(
                ServiceBuilder::new().layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            MakeSpanWithTrace::new().level(Level::INFO),
                        )
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Millis),
                        ),
                ),
            )
            .layer(middleware::from_fn(Self::trace))
            .fallback(Self::not_found)
            .layer(
                CorsLayer::new()
                    .expose_headers(ExposeHeaders::list(vec![
                        HeaderName::from_static("x-auth-token"),
                        HeaderName::from_static("x-account-id"),
                        HeaderName::from_static("x-user-id"),
                        HeaderName::from_static("x-trace-id"),
                    ]))
                    .allow_headers(AllowHeaders::mirror_request())
                    .allow_methods(AllowMethods::mirror_request())
                    .allow_origin(cors_origin)
                    .allow_credentials(true)
                    .max_age(Duration::from_secs(60) * 60 * 12),
            )
            .route_layer(middleware::from_fn(Self::track_metrics))
            .route("/metrics", get(Self::metrics));

        Ok(router)
    }

    async fn trace(request: Request, next: Next) -> impl IntoResponse {
        let (mut head, body) = request.into_parts();
        match head.headers.get("X-Trace-Id") {
            Some(v) => {
                let trace_header = v.clone();
                let mut response =
                    next.run(Request::from_parts(head, body)).await;
                response.headers_mut().insert("X-Trace-Id", trace_header);
                response
            }
            None => {
                let trace_header = HeaderValue::from_bytes(
                    uuid::Uuid::new_v4().hyphenated().to_string().as_bytes(),
                )
                .unwrap();
                (head.headers)
                    .entry("X-Trace-Id")
                    .or_insert(trace_header.clone());
                let mut response =
                    next.run(Request::from_parts(head, body)).await;
                response.headers_mut().insert("X-Trace-Id", trace_header);
                response
            }
        }
    }

    async fn track_metrics(request: Request, next: Next) -> impl IntoResponse {
        let path = if let Some(matched_path) =
            request.extensions().get::<MatchedPath>()
        {
            matched_path.as_str().to_owned()
        } else {
            request.uri().path().to_owned()
        };
        let start = Instant::now();
        let method = request.method().to_string();
        let response = next.run(request).await;
        let latency = start.elapsed();

        let labels = vec![method.as_str(), path.as_str()];
        HTTP_REQUESTS_TOTAL.with_label_values(&labels).inc();
        HTTP_REQUESTS_DURATION_SECONDS
            .with_label_values(&labels)
            .observe(latency.as_secs_f64());

        response
    }
    async fn metrics() -> impl IntoResponse {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();

        Response::builder()
            .status(200)
            .header(CONTENT_TYPE, encoder.format_type())
            .body(Body::from(buffer))
            .unwrap()
    }

    async fn not_found(uri: Uri) -> impl IntoResponse {
        errors::not_found(&format!("no route for {}", uri))
    }
}
