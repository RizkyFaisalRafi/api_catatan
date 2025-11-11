use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    domain::models::note::{NewNote, Note, UpdateNotePayload},
    utils::error::AppResult,
};

// Trait ini mendefinisikan kontrak untuk operasi data Note.
#[async_trait]
pub trait NoteRepository: Send + Sync {
    async fn create(&self, new_note: &NewNote) -> AppResult<Note>;
    async fn find_all(&self, user_id: u32) -> AppResult<Vec<Note>>;
    async fn find_by_id(&self, id: u32, user_id: u32) -> AppResult<Option<Note>>;
    async fn update(&self, id: u32, user_id: u32, payload: &UpdateNotePayload) -> AppResult<Option<Note>>;
    async fn delete(&self, id: u32, user_id: u32) -> AppResult<u64>;
}

// Tipe alias untuk Arc<dyn NoteRepository> agar lebih mudah digunakan.
pub type DynNoteRepository = Arc<dyn NoteRepository>;