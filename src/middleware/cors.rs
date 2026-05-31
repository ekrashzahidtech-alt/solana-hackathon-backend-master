use axum::http::Method;
use tower_http::cors::CorsLayer;

use crate::config::Settings;

pub fn build_cors(_settings: &Settings) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(tower_http::cors::Any)
}
