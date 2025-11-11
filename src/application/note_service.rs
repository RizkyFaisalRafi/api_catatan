use crate::{
    domain::{
        models::note::{Note, CreateNotePayload, UpdateNotePayload},
        repositories::note_repository::DynNoteRepository,
    },
    utils::error::{AppError, AppResult},
};

pub struct NoteService {
    note_repo: DynNoteRepository,
}

impl NoteService {
    pub fn new(note_repo: DynNoteRepository) -> Self {
        Self { note_repo }
    }

    pub async fn create_note(&self, payload: CreateNotePayload, user_id: u32) -> AppResult<Note> {
        self.note_repo.create(&payload, user_id).await
    }

    pub async fn get_all_notes(&self, user_id: u32) -> AppResult<Vec<Note>> {
        self.note_repo.find_all_by_user_id(user_id).await
    }

    pub async fn get_note_by_id(&self, id: u32, user_id: u32) -> AppResult<Note> {
        self.note_repo
            .find_by_id_and_user_id(id, user_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Catatan dengan id {} tidak ditemukan", id)))
    }

    pub async fn update_note(
        &self,
        id: u32,
        user_id: u32,
        payload: UpdateNotePayload
    ) -> AppResult<Note> {
        self.note_repo
            .update_by_id(id, user_id, &payload).await?
            .ok_or_else(|| AppError::NotFound(format!("Catatan dengan id {} tidak ditemukan", id)))
    }

    pub async fn delete_note(&self, id: u32, user_id: u32) -> AppResult<()> {
        let rows_affected = self.note_repo.delete_by_id_and_user_id(id, user_id).await?;
        if rows_affected == 0 {
            return Err(AppError::NotFound(format!("Catatan dengan id {} tidak ditemukan", id)));
        }
        Ok(())
    }
}