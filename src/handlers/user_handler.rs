use axum::{ extract::{ State, Extension }, http::StatusCode, response::Json };

use std::sync::Arc;
use redis::AsyncCommands;
use std::time::Duration;
// use chrono::Utc;
use chrono::{ Utc, FixedOffset }; // <-- MODIFIKASI BARIS INI
use jsonwebtoken::{ encode, Header, EncodingKey };
use humantime; // Untuk parse durasi "7d"
use crate::extractor::ApiJson; // Impor ApiJson extractor

use crate::{ // <-- TAMBAHKAN Role DI SINI
    models::{
        user_model::{ User, RegisterPayload, LoginPayload, TokenClaims, TokenResponse, UserProfile, Role },
        api_response::ApiResponse,
    },
    error::{ AppError, AppResult },
    AppState,
};

// Helper untuk hash password
// Penting: bcrypt mahal, jalankan di blocking thread
async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task
        ::spawn_blocking(move || { bcrypt::hash(password, bcrypt::DEFAULT_COST) }).await
        .map_err(|_| AppError::HashingError)
        ? // JoinError
        .map_err(|_| AppError::HashingError) // BcryptError
}

// Helper untuk verifikasi password
async fn verify_password(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task
        ::spawn_blocking(move || { bcrypt::verify(password, &hash) }).await
        .map_err(|_| AppError::HashingError)?
        .map_err(|_| AppError::HashingError)
}

// === REGISTER ===
pub async fn register(
    State(state): State<Arc<AppState>>,
    // Json(payload): Json<RegisterPayload>
    ApiJson(payload): ApiJson<RegisterPayload>
) -> AppResult<(StatusCode, Json<ApiResponse<User>>)> {
    // Cek apakah user sudah ada
    let existing_user = sqlx
        ::query("SELECT id FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&state.db_pool).await?;

    if existing_user.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    // Cek apakah username sudah ada (BARU)
    let existing_user_username = sqlx
        ::query("SELECT id FROM users WHERE username = ?")
        .bind(&payload.username)
        .fetch_optional(&state.db_pool).await?;

    if existing_user_username.is_some() {
        return Err(AppError::UsernameTaken); // Error baru
    }

    // Hash password
    let password_hash = hash_password(payload.password.clone()).await?;

    // Simpan user baru
    // Peran (role) secara otomatis di-set sebagai 'user'
    let insert_result = sqlx
        ::query(
            "INSERT INTO users (email, full_name, username, role, password_hash) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&payload.email)
        .bind(&payload.full_name) // <-- TAMBAHKAN INI
        .bind(&payload.username)
        .bind(Role::User.as_ref()) // <-- UBAH MENJADI STRING "user"
        .bind(&password_hash)
        .execute(&state.db_pool).await?;

    let new_user_id = insert_result.last_insert_id() as u32;

    // Ambil dan kembalikan data user baru
    let new_user = sqlx
        ::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(new_user_id)
        .fetch_one(&state.db_pool).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: "Registrasi berhasil.".to_string(),
        data: new_user,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// === LOGIN ===
pub async fn login(
    State(state): State<Arc<AppState>>,
    // Json(payload): Json<LoginPayload>
    ApiJson(payload): ApiJson<LoginPayload>,
) -> AppResult<Json<ApiResponse<TokenResponse>>> {
    // Cari user berdasarkan email
    let user = sqlx
        ::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_one(&state.db_pool).await
        .map_err(|e| {
            match e {
                sqlx::Error::RowNotFound => AppError::WrongCredentials,
                _ => e.into(),
            }
        })?;

    // Verifikasi password
    let password_valid = verify_password(payload.password.clone(), user.password_hash).await?;

    if !password_valid {
        return Err(AppError::WrongCredentials);
    }

    tracing::info!("Nilai JWT_EXPIRES_IN dari config: {}", &state.config.jwt_expires_in);

    // Buat token JWT
    let now = Utc::now();

    let expires_in = humantime
        ::parse_duration(state.config.jwt_expires_in.trim())
        .unwrap_or(Duration::from_secs(7 * 24 * 60 * 60));

    tracing::info!("Durasi token yang ter-parsing: {:?}", expires_in);

    // Buat variabel DateTime untuk waktu kedaluwarsa
    let expires_at_datetime = now + expires_in;

    // Buat variabel timestamp (i64) UNTUK CLAIM TOKEN
    let exp_timestamp = expires_at_datetime.timestamp();

    // Buat variabel String (terformat) UNTUK RESPONS JSON
    let wib = FixedOffset::east_opt(7 * 3600).unwrap();
    let expires_at_wib = expires_at_datetime.with_timezone(&wib);
    let formatted_expires_at = expires_at_wib.format("%Y-%m-%d %H:%M:%S WIB").to_string();

    // Buat claims token (HARUS PAKAI i64)
    let claims = TokenClaims {
        sub: user.id,
        exp: exp_timestamp, // <-- Gunakan timestamp (i64), BUKAN string
    };

    // ... (kode encode token) ...
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret_key.as_ref())
    ).map_err(|_| AppError::TokenCreationError)?;

    // Kirim respons
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Login berhasil.".to_string(),
        data: TokenResponse {
            access_token: token,
            // expires_at: exp, // Timestamp kedaluwarsa
            expires_at: formatted_expires_at,
        },
    };

    Ok(Json(response))
}

// === GET PROFILE ===
pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<TokenClaims> // Ambil user ID dari token
) -> AppResult<Json<ApiResponse<UserProfile>>> {
    let user_id = claims.sub;

    // Ambil data profil user dari database, kecualikan password_hash
    let user_profile = sqlx
        ::query_as::<_, UserProfile>(
            "SELECT id, email, full_name, username, role, created_at FROM users WHERE id = ?"
        )
        .bind(user_id)
        .fetch_one(&state.db_pool).await
        .map_err(|e| {
            match e {
                sqlx::Error::RowNotFound =>
                    AppError::NotFound(format!("Profil user dengan id {} tidak ditemukan", user_id)),
                _ => e.into(),
            }
        })?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: "Profil user berhasil diambil.".to_string(),
        data: user_profile,
    };

    Ok(Json(response))
}

// === LOGOUT ===
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<TokenClaims>, // 1. Ambil claims dari middleware
    Extension(token_str): Extension<String> // 2. Ambil raw token string dari middleware
) -> AppResult<Json<ApiResponse<()>>> {
    let mut redis_conn = state.redis_client.get_multiplexed_async_connection().await?;
    // 4. Hitung sisa masa berlaku token
    let now = Utc::now().timestamp();
    let ttl = claims.exp - now; // Time-to-live in seconds

    // Jika 'ttl' negatif (sudah kedaluwarsa), Redis akan auto-delete.
    // Kita set minimal 1 detik untuk amannya.
    if ttl <= 0 {
        return Err(AppError::TokenExpired); // Seharusnya tidak terjadi
    }

    // 5. Simpan token ke blacklist di Redis dengan TTL
    //    Value-nya '1' (atau apapun) tidak penting, yang penting KEY-nya
    let _: () = redis_conn
        .set_ex(
            token_str, // Key: token itu sendiri
            1, // Value: '1'
            ttl as u64 // Expiry: sisa waktu token
        ).await?;
    // 6. Kirim respons sukses
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Logout berhasil.".to_string(),
        data: (), // data: null (akan di-skip oleh serde)
    };

    Ok(Json(response))
}

// === GET ALL USERS ===
pub async fn get_all_users(
    State(state): State<Arc<AppState>>
) -> AppResult<Json<ApiResponse<Vec<UserProfile>>>> {
    // Ambil semua data profil user dari database, urutkan berdasarkan yang terbaru
    let users = sqlx
        ::query_as::<_, UserProfile>(
            "SELECT id, email, full_name, username, role, created_at FROM users ORDER BY created_at DESC"
        )
        .fetch_all(&state.db_pool).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: "Data semua user berhasil diambil.".to_string(),
        data: users,
    };

    Ok(Json(response))
}
