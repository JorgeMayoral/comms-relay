use std::time::Duration;

use axum::{
    body::Body,
    extract::{MatchedPath, Request},
    response::Response,
};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{DefaultOnRequest, TraceLayer},
};
use tracing::Span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type HttpTraceLayer = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    fn(&Request<Body>) -> Span,
    DefaultOnRequest,
    fn(&Response<Body>, Duration, &Span),
>;

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn get_tracing_layer() -> HttpTraceLayer {
    TraceLayer::new_for_http()
        .make_span_with(make_span as fn(&Request<Body>) -> Span)
        .on_response(on_response as fn(&Response<Body>, Duration, &Span))
}

fn make_span(req: &Request<Body>) -> Span {
    let method = req.method();
    let uri = req.uri();
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str);
    tracing::info_span!("request", %method, %uri, matched_path, status = tracing::field::Empty)
}

fn on_response(res: &Response<Body>, latency: Duration, span: &Span) {
    let status = res.status().as_u16();
    span.record("status", status);
    match status {
        500..=599 => tracing::error!(latency = ?latency, status, "response"),
        400..=499 => tracing::warn!(latency = ?latency, status, "response"),
        _ => tracing::info!(latency = ?latency, status, "response"),
    }
}
