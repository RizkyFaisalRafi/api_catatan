use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use serde_json::json;

// membuat error enum kustom yang bisa diubah Axum menjadi respons JSON yang rapi.

// Enum ini akan menampung semua kemungkinan error di aplikasi
pub enum AppError {
    SqlxError(sqlx::Error),
    NotFound(String),
    // bisa menambah error lain di sini (e.g., AuthError)
}

// Ini adalah "magic" nya.
// implement 'IntoResponse' agar Axum tahu
// cara mengubah AppError menjadi Respons HTTP.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Tentukan status HTTP dan pesan error
        let (status_code, error_message) = match self {
            AppError::SqlxError(e) => {
                // Penting: Jangan bocorkan detail error database ke user
                tracing::error!("SQLx Error: {}", e); // Log error untuk debugging
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Terjadi kesalahan pada server.".to_string(),
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        // Buat body JSON untuk error
        let body = Json(json!({
            "status": "error",
            "message": error_message,
        }));

        // Kembalikan (StatusCode, JsonBody)
        (status_code, body).into_response()
    }
}

// Ini adalah helper agar bisa menggunakan '?'
// pada sqlx::Error, yang akan otomatis mengubahnya
// menjadi AppError::SqlxError
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::SqlxError(e)
    }
}