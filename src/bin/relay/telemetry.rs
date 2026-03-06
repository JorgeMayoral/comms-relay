use axum::{
    body::Body,
    extract::{MatchedPath, Request},
};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
use tracing::Span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

pub fn get_tracing_layer()
-> TraceLayer<SharedClassifier<ServerErrorsAsFailures>, fn(&Request<Body>) -> Span> {
    TraceLayer::new_for_http().make_span_with(make_span as fn(&Request<Body>) -> Span)
}

fn make_span(req: &Request<Body>) -> Span {
    let method = req.method();
    let uri = req.uri();
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|matched_path| matched_path.as_str());
    tracing::debug_span!("request", %method, %uri, matched_path)
}
