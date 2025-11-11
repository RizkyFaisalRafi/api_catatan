use async_trait::async_trait;
use std::sync::Arc;

use crate::domain::models::user::{User, UserProfile};
use crate::utils::error::AppResult;

// Trait ini adalah "port" dalam arsitektur heksagonal.
// Lapisan aplikasi akan bergantung pada trait ini, bukan pada implementasi konkret.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn find_by_id(&self, id: u32) -> AppResult<Option<User>>;
    async fn find_profile_by_id(&self, id: u32) -> AppResult<Option<UserProfile>>;
    async fn get_all_profiles(&self) -> AppResult<Vec<UserProfile>>;
    async fn create(&self, payload: &User) -> AppResult<User>;
}

pub type DynUserRepository = Arc<dyn UserRepository>;