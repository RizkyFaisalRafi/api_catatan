use bcrypt;

use crate::{
    domain::{
        models::user::{LoginPayload, RegisterPayload, User},
        repositories::user_repository::DynUserRepository,
    },
    utils::error::{AppError, AppResult},
};

pub struct UserService {
    user_repo: DynUserRepository,
}

impl UserService {
    pub fn new(user_repo: DynUserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn register_user(&self, payload: RegisterPayload) -> AppResult<User> {
        // Cek apakah email sudah ada
        if self.user_repo.find_by_email(&payload.email).await?.is_some() {
            return Err(AppError::UserAlreadyExists);
        }

        // Cek apakah username sudah ada
        if self.user_repo.find_by_username(&payload.username).await?.is_some() {
            return Err(AppError::UsernameTaken);
        }

        // Hash password (helper ini bisa dipindahkan ke modul utilitas)
        let password_hash = hash_password(payload.password).await?;

        // Buat entitas User
        let new_user_data = User {
            id: 0, // ID akan di-generate oleh database
            email: payload.email,
            full_name: payload.full_name,
            username: payload.username,
            role: "user".to_string(), // Default role
            password_hash,
            created_at: chrono::Utc::now(), // Akan di-override oleh DB, tapi baik untuk ada
        };

        // Simpan ke repositori
        self.user_repo.create(&new_user_data).await
    }

    pub async fn login_user(&self, payload: LoginPayload) -> AppResult<User> {
        // Cari user berdasarkan email
        let user = self.user_repo
            .find_by_email(&payload.email).await?
            .ok_or(AppError::WrongCredentials)?;

        // Verifikasi password
        let password_valid = verify_password(payload.password, user.password_hash.clone()).await?;
        if !password_valid {
            return Err(AppError::WrongCredentials);
        }

        Ok(user)
    }
}

// Helper ini bisa dipindah ke modul terpisah, misal `application/utils/security.rs`
async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || bcrypt::hash(password, bcrypt::DEFAULT_COST))
        .await
        .map_err(|_| AppError::HashingError)?
        .map_err(|_| AppError::HashingError)
}

async fn verify_password(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || bcrypt::verify(password, &hash))
        .await
        .map_err(|_| AppError::HashingError)?
        .map_err(|_| AppError::HashingError)
}
