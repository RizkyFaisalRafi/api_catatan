use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

// Mewakili tabel 'users' di database
#[derive(FromRow, Debug, Serialize)]
pub struct User {
    pub id: u32,
    pub email: String,
    #[serde(skip_serializing)] // Tidak serialisasi ke JSON
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

// Payload untuk registrasi dan login
#[derive(Deserialize, Debug)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

// Struct untuk data di dalam token JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: u32,
    pub exp: i64,
}

// Struct untuk response token JWT
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_at: DateTime<Utc>, // Timestamp kedaluwarsa
}