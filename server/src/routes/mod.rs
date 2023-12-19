use std::{
    future::ready,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use axum::{
    extract::{MatchedPath, Request},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    Router,
};
use http::{header::HeaderName, HeaderValue, Uri};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowHeaders, AllowMethods, CorsLayer, ExposeHeaders},
    services::ServeDir,
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::{
    controllers::{
        auth::AuthRouter, groups::GroupsRouter, policies::PoliciesRouter,
        roles::RolesRouter, users::UsersRouter,
    },
    errors,
    middlewares::MakeSpanWithTrace,
    AppState,
};

lazy_static::lazy_static! {
    static ref EXPONENTIAL_SECONDS: &'static [f64] = &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,];
}

pub struct AppRouter;

impl AppRouter {
    pub fn build(cors_origin: &str, state: AppState) -> Result<Router> {
        let recorder_handle = PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full(String::from("http_requests_duration_seconds")),
                &EXPONENTIAL_SECONDS,
            )
            .context("could not setup buckets for metrics, verify matchers are correct")?
            .install_recorder()
            .context("could not install metrics recorder")?;

        let router = Router::new()
            .nest_service("/static", ServeDir::new("static"))
            .nest_service("/theme", ServeDir::new("theme"))
            .nest(
                "/v1",
                Router::new()
                    .merge(PoliciesRouter::new_router(state.clone()))
                    .merge(AuthRouter::new_router(state.clone()))
                    .merge(UsersRouter::new_router(state.clone()))
                    .merge(RolesRouter::new_router(state.clone()))
                    .merge(GroupsRouter::new_router(state)),
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
                    .allow_origin(cors_origin.parse::<HeaderValue>()?)
                    .allow_credentials(true)
                    .max_age(Duration::from_secs(60) * 60 * 12),
            )
            .route_layer(middleware::from_fn(Self::track_metrics))
            .route("/metrics", get(move || ready(recorder_handle.render())));

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
        let method = request.method().clone();
        let response = next.run(request).await;
        let latency = start.elapsed().as_secs_f64();
        let status = response.status().as_u16().to_string();

        let mut labels = vec![
            ("method", method.to_string()),
            ("path", path),
            ("status", status),
        ];
        if let Some(trace_id) = response.headers().get("X-Trace-Id") {
            labels.push(("x_trace_id", trace_id.to_str().unwrap().to_owned()));
        }

        metrics::increment_counter!("http_requests_total", &labels);
        metrics::histogram!("http_requests_duration_seconds", latency, &labels);

        response
    }

    async fn not_found(uri: Uri) -> impl IntoResponse {
        errors::not_found(&format!("no route for {}", uri))
    }
}
