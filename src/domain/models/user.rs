use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Enum untuk Role, agar lebih aman dan terstruktur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
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

// Entitas Domain 'User'
// Mewakili tabel 'users' di database
#[derive(FromRow, Debug, Serialize)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub full_name: String,
    pub username: String,
    pub role: String,
    #[serde(skip_serializing)]
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

// DTO (Data Transfer Object) untuk profil, bukan entitas murni
// Struct untuk data profil yang akan dikirim ke klien
// Perhatikan tidak ada password_hash di sini.
#[derive(FromRow, Debug, Serialize)]
pub struct UserProfile {
    pub id: u32,
    pub email: String,
    pub full_name: String,
    pub username: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

// Struct untuk data di dalam token JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: u32,
    pub role: String,
    pub exp: i64,
}

// Struct untuk response token JWT
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_at: String,
}