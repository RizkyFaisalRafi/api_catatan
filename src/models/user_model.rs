use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

// Enum untuk Role, agar lebih aman dan terstruktur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Admin,
}

// Implementasi agar kita bisa menulis Role::User.as_ref() -> "user"
impl AsRef<str> for Role {
    fn as_ref(&self) -> &str {
        match self {
            Role::User => "user",
            Role::Admin => "admin",
        }
    }
}

// Mewakili tabel 'users' di database
#[derive(FromRow, Debug, Serialize)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub full_name: String,
    pub username: String,
    pub role: String, // <-- BACA SEBAGAI STRING DARI DB
    #[serde(skip_serializing)] // Tidak serialisasi ke JSON
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

// Payload untuk registrasi dan login
#[derive(Deserialize, Debug)]
pub struct RegisterPayload {
    pub email: String,
    pub full_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

// Struct untuk data profil yang akan dikirim ke klien
// Perhatikan tidak ada password_hash di sini.
#[derive(FromRow, Debug, Serialize)]
pub struct UserProfile {
    pub id: u32,
    pub email: String,
    pub full_name: String,
    pub username: String,
    pub role: String, // <-- BACA SEBAGAI STRING DARI DB
    pub created_at: DateTime<Utc>,
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
    // pub expires_at: DateTime<Utc>, // Timestamp kedaluwarsa
    pub expires_at: String, // <-- UBAH DARI 'i64' atau 'DateTime<Utc>' MENJADI 'String'
}