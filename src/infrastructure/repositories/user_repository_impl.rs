use async_trait::async_trait;
use sqlx::MySqlPool;

use crate::domain::{
    models::user::{User, UserProfile},
    repositories::user_repository::UserRepository,
};
use crate::utils::error::{AppError, AppResult};

// Ini adalah "adapter" yang mengimplementasikan port UserRepository.
pub struct UserRepositoryImpl {
    db_pool: MySqlPool,
}

impl UserRepositoryImpl {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(user)
    }

    async fn find_by_id(&self, id: u32) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(user)
    }

    async fn find_profile_by_id(&self, id: u32) -> AppResult<Option<UserProfile>> {
        let profile = sqlx::query_as::<_, UserProfile>(
            "SELECT id, email, full_name, username, role, created_at FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db_pool)
        .await?;
        Ok(profile)
    }

    async fn get_all_profiles(&self) -> AppResult<Vec<UserProfile>> {
        let users = sqlx::query_as::<_, UserProfile>(
            "SELECT id, email, full_name, username, role, created_at FROM users ORDER BY created_at DESC",
        )
        .fetch_all(&self.db_pool)
        .await?;
        Ok(users)
    }

    async fn create(&self, user_data: &User) -> AppResult<User> {
        let insert_result = sqlx::query(
            "INSERT INTO users (email, full_name, username, role, password_hash) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&user_data.email)
        .bind(&user_data.full_name)
        .bind(&user_data.username)
        .bind(&user_data.role)
        .bind(&user_data.password_hash)
        .execute(&self.db_pool)
        .await?;

        let new_user_id = insert_result.last_insert_id() as u32;

        let new_user = self.find_by_id(new_user_id).await?.ok_or(AppError::NotFound("Gagal mengambil user setelah dibuat".to_string()))?;
        Ok(new_user)
    }
}