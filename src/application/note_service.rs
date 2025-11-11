use crate::{
    domain::{
        models::note::{CreateNotePayload, NewNote, Note, UpdateNotePayload},
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
        let new_note = NewNote {
            user_id,
            title: payload.title,
            content: payload.content,
        };
        self.note_repo.create(&new_note).await
    }

    pub async fn get_all_notes(&self, user_id: u32) -> AppResult<Vec<Note>> {
        self.note_repo.find_all(user_id).await
    }

    pub async fn get_note_by_id(&self, id: u32, user_id: u32) -> AppResult<Note> {
        self.note_repo
            .find_by_id(id, user_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Catatan dengan id {} tidak ditemukan atau bukan milik anda",
                    id
                ))
            })
    }

    pub async fn update_note(
        &self,
        id: u32,
        user_id: u32,
        payload: UpdateNotePayload,
    ) -> AppResult<Note> {
        self.note_repo
            .update(id, user_id, &payload)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Catatan dengan id {} tidak ditemukan atau bukan milik anda",
                    id
                ))
            })
    }

    pub async fn delete_note(&self, id: u32, user_id: u32) -> AppResult<()> {
        let rows_affected = self.note_repo.delete(id, user_id).await?;
        if rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Catatan dengan id {} tidak ditemukan atau bukan milik anda",
                id
            )));
        }
        Ok(())
    }
}