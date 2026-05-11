use axum::extract::{Path, State};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use axum_extra::TypedHeader;
use headers::authorization::Bearer;
use headers::Authorization;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::{CreateElementBody, UserDto};
use crate::domain::BoardRole;
use crate::presentation::error::ApiError;
use crate::presentation::state::AppState;
use crate::presentation::ws::board_ws;

#[derive(Deserialize)]
pub struct RegisterBody {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct BoardCreateBody {
    pub title: String,
}

#[derive(Deserialize)]
pub struct BoardUpdateBody {
    pub title: String,
}

#[derive(Deserialize)]
pub struct AddMemberBody {
    pub email: String,
    pub role: BoardRole,
}

#[derive(Deserialize)]
pub struct SetRoleBody {
    pub role: BoardRole,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/me", get(me))
        .route("/api/boards", get(list_boards).post(create_board))
        .route(
            "/api/boards/{id}",
            get(get_board).patch(update_board).delete(delete_board),
        )
        .route("/api/boards/{id}/members", get(list_members).post(add_member))
        .route(
            "/api/boards/{id}/members/{user_id}",
            patch(set_member_role).delete(remove_member),
        )
        .route(
            "/api/boards/{id}/elements",
            get(list_elements).post(add_element).delete(clear_elements),
        )
        .route(
            "/api/boards/{id}/elements/{element_id}",
            delete(delete_element),
        )
        .route("/api/boards/{id}/ws", get(board_ws))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let (user_id, token) = state
        .auth
        .register(&body.email, &body.password, &body.display_name)
        .await?;
    Ok(Json(TokenResponse { token, user_id }))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let (user_id, token) = state.auth.login(&body.email, &body.password).await?;
    Ok(Json(TokenResponse { token, user_id }))
}

async fn me(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<UserDto>, ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    let dto = state.auth.me(uid).await?;
    Ok(Json(dto))
}

async fn list_boards(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<Vec<crate::domain::Board>>, ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    let boards = state.boards.list_boards(uid).await?;
    Ok(Json(boards))
}

async fn create_board(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(body): Json<BoardCreateBody>,
) -> Result<Json<crate::domain::Board>, ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    let b = state.boards.create_board(uid, &body.title).await?;
    Ok(Json(b))
}

async fn get_board(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::domain::Board>, ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    let b = state.boards.get_board(id, uid).await?;
    Ok(Json(b))
}

async fn update_board(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<Uuid>,
    Json(body): Json<BoardUpdateBody>,
) -> Result<(), ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    state.boards.update_board(id, uid, &body.title).await?;
    Ok(())
}

async fn delete_board(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<Uuid>,
) -> Result<(), ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    state.boards.delete_board(id, uid).await?;
    Ok(())
}

async fn list_members(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::application::MemberDto>>, ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    let m = state.boards.list_members(id, uid).await?;
    Ok(Json(m))
}

async fn add_member(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<Uuid>,
    Json(body): Json<AddMemberBody>,
) -> Result<(), ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    state
        .boards
        .add_member_by_email(id, uid, &body.email, body.role)
        .await?;
    Ok(())
}

async fn set_member_role(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path((board_id, user_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<SetRoleBody>,
) -> Result<(), ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    state
        .boards
        .set_member_role(board_id, uid, user_id, body.role)
        .await?;
    Ok(())
}

async fn remove_member(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path((board_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<(), ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    state.boards.remove_member(board_id, uid, user_id).await?;
    Ok(())
}

async fn list_elements(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::domain::BoardElement>>, ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    let els = state.boards.list_elements(id, uid).await?;
    Ok(Json(els))
}

async fn add_element(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateElementBody>,
) -> Result<Json<crate::domain::BoardElement>, ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    let el = state
        .boards
        .add_element(id, uid, &body.element_type, body.payload.clone(), body.z_index)
        .await?;
    let msg = serde_json::to_string(&crate::ws_protocol::ServerWsMessage::ElementAdded {
        element: el.clone(),
    })
    .unwrap_or_default();
    state.rooms.publish(id, msg).await;
    Ok(Json(el))
}

async fn delete_element(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path((board_id, element_id)): Path<(Uuid, Uuid)>,
) -> Result<(), ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    state.boards.remove_element(board_id, uid, element_id).await?;
    let msg = serde_json::to_string(&crate::ws_protocol::ServerWsMessage::ElementRemoved {
        id: element_id,
    })
    .unwrap_or_default();
    state.rooms.publish(board_id, msg).await;
    Ok(())
}

async fn clear_elements(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<Uuid>,
) -> Result<(), ApiError> {
    let uid = state.auth.verify_token(auth.token())?;
    state.boards.clear_elements(id, uid).await?;
    let msg = serde_json::to_string(&crate::ws_protocol::ServerWsMessage::Cleared).unwrap_or_default();
    state.rooms.publish(id, msg).await;
    Ok(())
}
