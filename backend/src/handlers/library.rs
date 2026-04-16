use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::db;
use crate::models::{
    CreateWebsiteFolderRequest, ReorderWebsiteFoldersRequest, UpdateWebsiteFolderRequest,
};
use crate::security::{AccessContext, AuthState};
use crate::state::AppState;

use super::{map_db_err, require_present, validate_nonempty};

fn require_authenticated_user<'a>(
    access_context: &'a AccessContext,
) -> Result<&'a str, (StatusCode, String)> {
    if access_context.auth_state != AuthState::Authenticated {
        return Err((StatusCode::FORBIDDEN, "Sign-in required".to_string()));
    }

    access_context
        .user_id
        .as_deref()
        .ok_or((StatusCode::FORBIDDEN, "Sign-in required".to_string()))
}

pub async fn list_website_folders(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_authenticated_user(&access_context)?;
    let folders = db::list_website_folders(&state.db, user_id)
        .await
        .map_err(map_db_err)?;
    Ok(Json(folders))
}

pub async fn create_website_folder(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Json(payload): Json<CreateWebsiteFolderRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_authenticated_user(&access_context)?;
    let name = validate_nonempty(&payload.name, "Folder name is required.")?;
    let folder = db::create_website_folder(&state.db, user_id, name)
        .await
        .map_err(map_db_err)?;
    Ok((StatusCode::CREATED, Json(folder)))
}

pub async fn update_website_folder(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(folder_id): Path<String>,
    Json(payload): Json<UpdateWebsiteFolderRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_authenticated_user(&access_context)?;
    let name = validate_nonempty(&payload.name, "Folder name is required.")?;
    let folder = db::update_website_folder_name(&state.db, user_id, &folder_id, name)
        .await
        .map_err(map_db_err)
        .and_then(|folder| require_present(folder, "Folder not found"))?;
    Ok(Json(folder))
}

pub async fn reorder_website_folders(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Json(payload): Json<ReorderWebsiteFoldersRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_authenticated_user(&access_context)?;
    let folders = db::reorder_website_folders(&state.db, user_id, &payload.folder_ids)
        .await
        .map_err(map_db_err)?;
    Ok(Json(folders))
}

pub async fn delete_website_folder(
    State(state): State<AppState>,
    Extension(access_context): Extension<AccessContext>,
    Path(folder_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = require_authenticated_user(&access_context)?;
    let deleted = db::delete_website_folder(&state.db, user_id, &folder_id)
        .await
        .map_err(map_db_err)?;
    if !deleted {
        return Err((StatusCode::NOT_FOUND, "Folder not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}
