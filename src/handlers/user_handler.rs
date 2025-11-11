use axum::{ extract::{ State, Extension }, http::StatusCode, response::Json };

use std::sync::Arc;
use redis::AsyncCommands;
use std::time::Duration;
use chrono::Utc;
use jsonwebtoken::{ encode, Header, EncodingKey };
use humantime; // Untuk parse durasi "7d"

use crate::{
    models::{
        user_model::{ User, RegisterPayload, LoginPayload, TokenClaims, TokenResponse },
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
    Json(payload): Json<RegisterPayload>
) -> AppResult<(StatusCode, Json<ApiResponse<User>>)> {
    // Cek apakah user sudah ada
    let existing_user = sqlx
        ::query("SELECT id FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_optional(&state.db_pool).await?;

    if existing_user.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    // Hash password
    let password_hash = hash_password(payload.password).await?;

    // Simpan user baru
    let insert_result = sqlx
        ::query("INSERT INTO users (email, password_hash) VALUES (?, ?)")
        .bind(&payload.email)
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
    Json(payload): Json<LoginPayload>
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
    let password_valid = verify_password(payload.password, user.password_hash).await?;

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

    // let exp = (now + expires_in).timestamp();

    // --- PERUBAHAN DI SINI ---
    // 1. Buat objek DateTime untuk waktu kedaluwarsa
    let expires_at_datetime = now + expires_in; 
    
    // 2. Buat timestamp (angka i64) KHUSUS UNTUK CLAIM JWT
    let exp_timestamp = expires_at_datetime.timestamp();

    let claims = TokenClaims {
        sub: user.id,
        exp: exp_timestamp, // Claim JWT (exp) HARUS tetap angka
    };
    // --- AKHIR PERUBAHAN ---


    // let claims = TokenClaims {
    //     sub: user.id,
    //     exp,
    // };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret_key.as_ref())
    ).map_err(|_| AppError::TokenCreationError)?;

    // 4. Kirim respons
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Login berhasil.".to_string(),
        data: TokenResponse {
            access_token: token,
            // expires_at: exp, // Timestamp kedaluwarsa
            expires_at: expires_at_datetime,
        },
    };

    Ok(Json(response))
}

// === LOGOUT ===
pub async fn logout(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<TokenClaims>, // 1. Ambil claims dari middleware
    Extension(token_str): Extension<String> // 2. Ambil raw token string dari middleware
) -> AppResult<Json<ApiResponse<()>>> {
    // 3. Dapatkan koneksi Redis
    let mut redis_conn = state.redis_client.get_multiplexed_async_connection().await.map_err(|e| {
        tracing::error!("Gagal konek Redis: {}", e);
        AppError::SqlxError(sqlx::Error::PoolClosed) // Placeholder error
    })?;

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
        ).await
        .map_err(|e| {
            tracing::error!("Gagal set Redis: {}", e);
            AppError::SqlxError(sqlx::Error::PoolClosed) // Placeholder error
        })?;

    // 6. Kirim respons sukses
    let response = ApiResponse {
        status: "success".to_string(),
        message: "Logout berhasil.".to_string(),
        data: (), // data: null (akan di-skip oleh serde)
    };

    Ok(Json(response))
}
