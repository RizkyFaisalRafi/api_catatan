use axum::{ extract::{ State, Path }, http::StatusCode, response::Json };
// use sqlx::MySqlPool;
use std::sync::Arc;

use crate::models::note_model::{ Note, CreateNotePayload, UpdateNotePayload };
use crate::error::AppError; // Untuk AppError dan AppResult
use crate::models::api_response::ApiResponse; // Untuk respons API yang konsisten
use crate::AppState; // buat AppState di main.rs

// Tipe alias untuk 'Result', agar lebih rapi
type AppResult<T> = Result<T, AppError>;

// === CREATE ===
pub async fn create_note(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateNotePayload>
) -> AppResult<(StatusCode, Json<ApiResponse<Note>>)> {
    let insert_result = sqlx
        ::query("INSERT INTO notes (title, content) VALUES (?, ?)")
        .bind(&payload.title)
        .bind(&payload.content)
        .execute(&state.db_pool).await?; // <-- '?' sekarang otomatis memanggil impl From<sqlx::Error>

    let new_id = insert_result.last_insert_id() as u32;

    // Ambil data yang baru saja di-insert
    let new_note = sqlx
        ::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(new_id)
        .fetch_one(&state.db_pool).await?; // <-- '?' juga di sini

    // Bungkus dengan ApiResponse
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
): State<Arc<AppState>>) -> AppResult<Json<ApiResponse<Vec<Note>>>> {
    // <-- Tipe return diubah

    let notes = sqlx
        // Ambil semua catatan dari DB urutkan berdasarkan created_at terbaru
        ::query_as::<_, Note>("SELECT * FROM notes ORDER BY created_at DESC")
        .fetch_all(&state.db_pool).await?; // <-- Error handling sederhana

    // Bungkus dengan ApiResponse
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
    Path(id): Path<u32>
) -> AppResult<Json<ApiResponse<Note>>> {
    // <-- Tipe return diubah

    let note = sqlx
        ::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db_pool).await
        .map_err(|e| {
            match e {
                // Kita ubah error spesifik menjadi AppError::NotFound
                sqlx::Error::RowNotFound =>
                    AppError::NotFound(format!("Catatan dengan id {} tidak ditemukan", id)),
                _ => e.into(), // Error lain dikonversi
            }
        })?;

    // Bungkus dengan ApiResponse
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
    Json(payload): Json<UpdateNotePayload>
) -> AppResult<Json<ApiResponse<Note>>> {
    // <-- Tipe return diubah

    let mut note = sqlx
        ::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
        .bind(id)
        .fetch_one(&state.db_pool).await
        .map_err(|e| {
            match e {
                sqlx::Error::RowNotFound =>
                    AppError::NotFound(format!("Catatan dengan id {} tidak ditemukan", id)),
                _ => e.into(),
            }
        })?;

    if let Some(title) = payload.title {
        note.title = title;
    }
    if let Some(content) = payload.content {
        note.content = Some(content);
    }

    sqlx
        ::query("UPDATE notes SET title = ?, content = ? WHERE id = ?")
        .bind(&note.title)
        .bind(&note.content)
        .bind(id)
        .execute(&state.db_pool).await?;

    // Bungkus dengan ApiResponse
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
    Path(id): Path<u32>
) -> AppResult<Json<ApiResponse<()>>> {
    // <-- Tipe return diubah (data: null)

    let result = sqlx
        ::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&state.db_pool).await?;

    if result.rows_affected() == 0 {
        // Kembalikan error
        Err(AppError::NotFound(format!("Catatan dengan id {} tidak ditemukan", id)))
    } else {
        // Bungkus dengan ApiResponse (data bisa kosong/unit type '()')
        let response = ApiResponse {
            status: "success".to_string(),
            message: "Catatan berhasil dihapus.".to_string(),
            data: (), // Unit type untuk data kosong
        };
        Ok(Json(response))
    }
}
