use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    domain::models::note::{Note, CreateNotePayload, UpdateNotePayload},
    utils::error::AppResult,
};

// Trait ini mendefinisikan kontrak untuk operasi data Note.
#[async_trait]
pub trait NoteRepository: Send + Sync {
    async fn create(&self, payload: &CreateNotePayload, user_id: u32) -> AppResult<Note>;
    async fn find_all_by_user_id(&self, user_id: u32) -> AppResult<Vec<Note>>;
    async fn find_by_id_and_user_id(&self, id: u32, user_id: u32) -> AppResult<Option<Note>>;
    async fn update_by_id(&self, id: u32, user_id: u32, payload: &UpdateNotePayload) -> AppResult<Option<Note>>;
    async fn delete_by_id_and_user_id(&self, id: u32, user_id: u32) -> AppResult<u64>;
}

// Tipe alias untuk Arc<dyn NoteRepository> agar lebih mudah digunakan.
pub type DynNoteRepository = Arc<dyn NoteRepository>;