use std::sync::Arc;
use uuid::Uuid;

use axum::extract::Path;
use axum::{
    Extension, Json, Router,
    extract::Query,
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use validator::Validate;

use crate::{
    AppState,
    db::UserExt,
    dtos::{PostDto, PostListResponseDto, RequestQueryDto, Response},
    error::{ErrorMessage, HttpError},
    middleware::JWTAuthMiddleware,
    models::Post,
};

pub fn post_handler() -> Router {
    Router::new()
        .route("/post", post(create_post))
        .route("/post/:id", get(get_post_by_id))
        .route("/posts", get(all_posts))
        .route("/post/:id", put(update_post))
        .route("/post/:id", delete(delete_post))
}

pub async fn create_post(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<PostDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(format!("Validation error: {}", e)))?;

    let user = &user.user;
    let user_id = user.id;
    println!("AUTH USER = {:?}", user_id);

    let create_post = app_state
        .db_client
        .create_post(user_id, &body.title, &body.content)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((
        axum::http::StatusCode::CREATED,
        Json(Response {
            status: "success",
            message: "Post created successfully!".to_string(),
        }),
    ))
}

pub async fn get_post_by_id(
    Path(post_id): Path<Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, HttpError> {
    let post = app_state
        .db_client
        .get_post(post_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .ok_or(HttpError::not_found("Post not found"))?;

    Ok(Json(post))
}

pub async fn all_posts(
    Query(query_params): Query<RequestQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let page = query_params.page.unwrap_or(1);
    let limit = query_params.limit.unwrap_or(10);

    let posts = app_state
        .db_client
        .get_posts(page as u32, limit)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok(Json(PostListResponseDto {
        status: "success".to_string(),
        results: posts.len() as i64,
        posts,
    }))
}

pub async fn update_post(
    Path(post_id): Path<Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<PostDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(format!("Validation error: {}", e)))?;

    let user = &user.user;
    let user_id = user.id;

    let updated_post = app_state
        .db_client
        .update_post(post_id, user_id, &body.title, &body.content)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((axum::http::StatusCode::OK, Json(updated_post)))
}

pub async fn delete_post(
    Path(post_id): Path<Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    let user = &user.user;
    let user_id = user.id;

    let deleted_post = app_state
        .db_client
        .delete_post(post_id, user_id)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    Ok((
        axum::http::StatusCode::OK,
        Json(Response {
            status: "success",
            message: "Post deleted successfully!".to_string(),
        }),
    ))
}
