use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{ IntoResponse, Response },
    Json,
};
use serde_json::json;

// --- Struct wrapper baru (Newtype Pattern) ---
// bungkus error Axum dengan struct milik kita.
// #[derive(Debug)]
// pub struct ApiJsonRejection(JsonRejection);

// --- Ubah AppError ---
pub enum AppError {
    SqlxError(sqlx::Error),
    RedisError(redis::RedisError), // <-- TAMBAHKAN INI
    NotFound(String),
    UserAlreadyExists,
    WrongCredentials,
    HashingError,
    TokenCreationError,
    MissingToken,
    InvalidToken,
    TokenExpired,
    Forbidden, // <-- TAMBAHKAN INI
    UsernameTaken,
    // JsonRejection(ApiJsonRejection),
    JsonRejection(JsonRejection), // <-- Gunakan JsonRejection secara langsung
}

// --- Implementasi 'IntoResponse' untuk AppError ---
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            AppError::SqlxError(e) => {
                tracing::error!("SQLx Error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Terjadi kesalahan pada server.".to_string())
            }
            AppError::RedisError(e) => { // <-- TAMBAHKAN INI
                tracing::error!("Redis Error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Terjadi kesalahan pada server.".to_string())
            }
            AppError::HashingError => {
                tracing::error!("Hashing Error: Gagal memproses password.");
                (StatusCode::INTERNAL_SERVER_ERROR, "Gagal memproses password.".to_string())
            }
            AppError::TokenCreationError => {
                tracing::error!("JWT Error: Gagal membuat token.");
                (StatusCode::INTERNAL_SERVER_ERROR, "Gagal membuat token.".to_string())
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::UserAlreadyExists =>
                (StatusCode::CONFLICT, "User dengan email ini sudah terdaftar.".to_string()),
            AppError::WrongCredentials =>
                (StatusCode::UNAUTHORIZED, "Email atau password salah.".to_string()),
            AppError::MissingToken =>
                (StatusCode::UNAUTHORIZED, "Token autentikasi tidak ditemukan.".to_string()),
            AppError::InvalidToken =>
                (StatusCode::UNAUTHORIZED, "Token autentikasi tidak valid.".to_string()),
            AppError::TokenExpired =>
                (StatusCode::UNAUTHORIZED, "Token autentikasi telah kedaluwarsa.".to_string()),
            AppError::Forbidden =>
                (StatusCode::FORBIDDEN, "Anda tidak memiliki hak akses untuk sumber daya ini.".to_string()),
            AppError::UsernameTaken =>
                (StatusCode::CONFLICT, "Username ini sudah digunakan.".to_string()),

            // --- TAMBAHKAN LOGIKA UNTUK MENANGANI JSON REJECTION ---
            AppError::JsonRejection(rejection) => {
                // Logika yang sebelumnya gagal, sekarang ada di sini.
                // Kita tidak perlu lagi '.0' karena tidak ada wrapper.
                match rejection {
                    JsonRejection::JsonDataError(e) => {
                        (StatusCode::UNPROCESSABLE_ENTITY, e.to_string())
                    }
                    JsonRejection::JsonSyntaxError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
                    JsonRejection::MissingJsonContentType(_) =>
                        (
                            StatusCode::BAD_REQUEST,
                            "Header Content-Type tidak ada atau salah.".to_string(),
                        ),
                    _ => (StatusCode::BAD_REQUEST, rejection.to_string()),
                }
            }
        };

        let body = Json(
            json!({
            "status": "error",
            "message": error_message,
        })
        );

        (status_code, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::SqlxError(e)
    }
}

// --- Implementasi 'From' untuk mengubah redis::RedisError -> AppError ---
impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        AppError::RedisError(e)
    }
}

// Tipe alias publik untuk Result<T, AppError>
pub type AppResult<T> = Result<T, AppError>;

// --- Implementasi 'From' untuk mengubah JsonRejection -> AppError ---
// Ini adalah "lem" yang akan otomatis mengubah error Axum
// menjadi error AppError::JsonRejection.
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        // AppError::JsonRejection(ApiJsonRejection(rejection))
        AppError::JsonRejection(rejection) // <-- Langsung bungkus tanpa struct tambahan
    }
}
