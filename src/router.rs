use std::sync::Arc;

use axum::{Extension, Router, middleware};
use tower_http::trace::TraceLayer;

use crate::{
    AppState,
    handler::{auth::auth_handler, post::post_handler, user::users_handler},
    middleware::JWTAuthMiddleware,
    middleware::auth
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .nest("/auth", auth_handler());

    let protected_routes = Router::new()
        .merge(users_handler())
        .nest("/posts", post_handler())
        .layer(middleware::from_fn(auth));

    let api_routes = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
        .layer(Extension(app_state));

    Router::new().nest("/api", api_routes)
}
