use axum::{ http::StatusCode, response::{ IntoResponse, Response }, Json };

use serde_json::json;

// membuat error enum kustom yang bisa diubah Axum menjadi respons JSON yang rapi.

// Enum ini akan menampung semua kemungkinan error di aplikasi
pub enum AppError {
    SqlxError(sqlx::Error),
    NotFound(String),
    // bisa menambah error lain di sini (e.g., AuthError)
    UserAlreadyExists,
    WrongCredentials,
    HashingError,
    TokenCreationError,
    MissingToken,
    InvalidToken,
    TokenExpired,
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
                (StatusCode::INTERNAL_SERVER_ERROR, "Terjadi kesalahan pada server.".to_string())
            }

            AppError::HashingError => {
                // (Tambahan) Ini 5xx, sebaiknya di-log
                tracing::error!("Hashing Error: Gagal memproses password.");
                (StatusCode::INTERNAL_SERVER_ERROR, "Gagal memproses password.".to_string())
            }
            AppError::TokenCreationError => {
                // (Tambahan) Ini 5xx, sebaiknya di-log
                tracing::error!("JWT Error: Gagal membuat token.");
                (StatusCode::INTERNAL_SERVER_ERROR, "Gagal membuat token.".to_string())
            }

            // --- Error 4xx (Client Error) ---
            // Ini tidak perlu di-log sebagai 'error' karena ini kesalahan user
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::UserAlreadyExists =>
                (
                    StatusCode::CONFLICT, // 409 Conflict
                    "User dengan email ini sudah terdaftar.".to_string(),
                ),
            AppError::WrongCredentials =>
                (
                    StatusCode::UNAUTHORIZED, // 401 Unauthorized
                    "Email atau password salah.".to_string(),
                ),
            AppError::MissingToken =>
                (StatusCode::UNAUTHORIZED, "Token autentikasi tidak ditemukan.".to_string()),
            AppError::InvalidToken =>
                (StatusCode::UNAUTHORIZED, "Token autentikasi tidak valid.".to_string()),
            AppError::TokenExpired =>
                (StatusCode::UNAUTHORIZED, "Token autentikasi telah kedaluwarsa.".to_string()),
        };

        // Buat body JSON untuk error
        let body = Json(
            json!({
            "status": "error",
            "message": error_message,
        })
        );

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

// Tipe alias publik untuk Result<T, AppError>
pub type AppResult<T> = Result<T, AppError>;
