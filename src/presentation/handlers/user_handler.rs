use axum::{ extract::{ State, Extension }, http::StatusCode, response::Json };

use std::sync::Arc;
use std::time::Duration;
use chrono::{ FixedOffset, Utc };
use jsonwebtoken::{ encode, Header, EncodingKey };
use humantime;
use redis::AsyncCommands;
use crate::{
    application::user_service::UserService,
    domain::models::{
        api_response::ApiResponse,
        user::{LoginPayload, RegisterPayload, TokenClaims, TokenResponse, User, UserProfile},
    },
    presentation::extractor::ApiJson,
    utils::error::{AppError, AppResult},
    AppState,
};

// === REGISTER ===
pub async fn register(
    State(state): State<Arc<AppState>>,
    ApiJson(payload): ApiJson<RegisterPayload>
) -> AppResult<(StatusCode, Json<ApiResponse<User>>)> {
    // Handler menjadi "tipis", hanya mendelegasikan ke service
    let user_service = UserService::new(state.user_repo.clone());
    let new_user = user_service.register_user(payload.0).await?;

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
    ApiJson(payload): ApiJson<LoginPayload>,
) -> AppResult<Json<ApiResponse<TokenResponse>>> {
    // Delegasikan logika login ke service
    let user_service = UserService::new(state.user_repo.clone());
    let user = user_service.login_user(payload.0).await?;

    // Buat token JWT
    let now = Utc::now();
    let expires_in = humantime
        ::parse_duration(state.config.jwt_expires_in.trim())
        .unwrap_or(Duration::from_secs(7 * 24 * 60 * 60));
    let expires_at_datetime = now + expires_in;
    let exp_timestamp = expires_at_datetime.timestamp();

    let wib = FixedOffset::east_opt(7 * 3600).unwrap();
    let expires_at_wib = expires_at_datetime.with_timezone(&wib);
    let formatted_expires_at = expires_at_wib.format("%Y-%m-%d %H:%M:%S WIB").to_string();

    // Buat claims token (HARUS PAKAI i64)
    let claims = TokenClaims {
        sub: user.id,
        exp: exp_timestamp,
    };

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
    let user_profile = state.user_repo
        .find_profile_by_id(user_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Profil user dengan id {} tidak ditemukan", user_id)))?;

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
    let now = Utc::now().timestamp();
    let ttl = claims.exp - now;

    if ttl <= 0 {
        return Err(AppError::TokenExpired);
    }

    let _: () = redis_conn
        .set_ex(
            token_str,
            1,
            ttl as u64
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
    let users = state.user_repo.get_all_profiles().await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: "Data semua user berhasil diambil.".to_string(),
        data: users,
    };

    Ok(Json(response))
}
