use axum::{ extract::{ State, Path, Extension }, http::StatusCode, response::Json };
use std::sync::Arc;

use crate::{
    application::note_service::NoteService,
    domain::models::{
        api_response::ApiResponse,
        note::{CreateNotePayload, Note, UpdateNotePayload},
        user::TokenClaims,
    },
    presentation::extractor::ApiJson,
    utils::error::AppResult,
    AppState,
};

// === CREATE ===
pub async fn create_note(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<TokenClaims>, // Ambil user ID dari token
    ApiJson(payload): ApiJson<CreateNotePayload>
) -> AppResult<(StatusCode, Json<ApiResponse<Note>>)> {
    let user_id = claims.sub; // ID user yang membuat catatan
    let note_service = NoteService::new(state.note_repo.clone());
    let new_note = note_service.create_note(payload.0, user_id).await?;
    
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Catatan berhasil dibuat.".to_string(),
        data: new_note,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

// === READ (Get All) ===
pub async fn get_all_notes(State(
    state,
): State<Arc<AppState>>, Extension(claims): Extension<TokenClaims>) -> AppResult<Json<ApiResponse<Vec<Note>>>> {
    let user_id = claims.sub;
    let note_service = NoteService::new(state.note_repo.clone());
    let notes = note_service.get_all_notes(user_id).await?;
    
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Data catatan berhasil diambil.".to_string(),
        data: notes,
    };
    Ok(Json(response))
}

// === READ (Get One by ID) ===
pub async fn get_note_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
    Extension(claims): Extension<TokenClaims>
) -> AppResult<Json<ApiResponse<Note>>> {
    let user_id = claims.sub;
    let note_service = NoteService::new(state.note_repo.clone());
    let note = note_service.get_note_by_id(id, user_id).await?;
    
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Catatan berhasil ditemukan.".to_string(),
        data: note,
    };
    Ok(Json(response))
}

// === UPDATE ===
pub async fn update_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
    Extension(claims): Extension<TokenClaims>,
    ApiJson(payload): ApiJson<UpdateNotePayload>
) -> AppResult<Json<ApiResponse<Note>>> {
    let user_id = claims.sub;
    let note_service = NoteService::new(state.note_repo.clone());
    let note = note_service.update_note(id, user_id, payload.0).await?;
    
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Catatan berhasil diperbarui.".to_string(),
        data: note,
    };
    Ok(Json(response))
}

// === DELETE ===
pub async fn delete_note(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u32>,
    Extension(claims): Extension<TokenClaims>
) -> AppResult<Json<ApiResponse<()>>> {
    let user_id = claims.sub;
    let note_service = NoteService::new(state.note_repo.clone());
    note_service.delete_note(id, user_id).await?;
    
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Catatan berhasil dihapus.".to_string(),
        data: (), // Unit type untuk data kosong
    };
    Ok(Json(response))
}
