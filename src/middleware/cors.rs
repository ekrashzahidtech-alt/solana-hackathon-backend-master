use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

use crate::config::Settings;

pub fn build_cors(settings: &Settings) -> CorsLayer {
    // Allow requests from any origin (useful for browser clients).
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
