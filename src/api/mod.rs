use std::sync::Arc;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

mod admin;
pub mod auth;
mod bot;
mod system;
mod user;
pub mod ws;

pub fn router(state: Arc<AppState>) -> Router {
    let api = Router::new()
        .nest("/bot", bot::router())
        .nest("/user", user::router())
        .nest("/admin", admin::router())
        .nest("/system", system::router())
        .with_state(state.clone());

    Router::new()
        .nest("/api/v1", api)
        // Serve compiled frontend from frontend/dist/ at "/"
        .fallback_service(
            ServeDir::new("frontend/dist/browser")
                .fallback(ServeFile::new("frontend/dist/browser/index.html")),
        )
        .layer(TraceLayer::new_for_http())
}
