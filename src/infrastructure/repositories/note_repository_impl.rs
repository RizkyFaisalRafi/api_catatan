use async_trait::async_trait;
use sqlx::MySqlPool;

use crate::{
    domain::{
        models::note::{CreateNotePayload, Note, UpdateNotePayload},
        repositories::note_repository::NoteRepository,
    },
    utils::error::AppResult,
};

pub struct NoteRepositoryImpl {
    db_pool: MySqlPool,
}

impl NoteRepositoryImpl {
    pub fn new(db_pool: MySqlPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl NoteRepository for NoteRepositoryImpl {
    async fn create(&self, payload: &CreateNotePayload, user_id: u32) -> AppResult<Note> {
        let insert_result = sqlx
            ::query("INSERT INTO notes (title, content, user_id) VALUES (?, ?, ?)")
            .bind(&payload.title)
            .bind(&payload.content)
            .bind(user_id)
            .execute(&self.db_pool).await?;

        let new_id = insert_result.last_insert_id() as u32;

        let new_note = sqlx
            ::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
            .bind(new_id)
            .fetch_one(&self.db_pool).await?;

        Ok(new_note)
    }

    async fn find_all_by_user_id(&self, user_id: u32) -> AppResult<Vec<Note>> {
        let notes = sqlx
            ::query_as::<_, Note>("SELECT * FROM notes WHERE user_id = ? ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.db_pool).await?;
        Ok(notes)
    }

    async fn find_by_id_and_user_id(&self, id: u32, user_id: u32) -> AppResult<Option<Note>> {
        let note = sqlx
            ::query_as::<_, Note>("SELECT * FROM notes WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.db_pool).await?;
        Ok(note)
    }

    async fn update_by_id(&self, id: u32, user_id: u32, payload: &UpdateNotePayload) -> AppResult<Option<Note>> {
        // Pertama, ambil catatan yang ada untuk memastikan pemiliknya benar
        let mut note = match self.find_by_id_and_user_id(id, user_id).await? {
            Some(note) => note,
            None => return Ok(None), // Jika tidak ditemukan, kembalikan None
        };

        // Perbarui field jika ada di payload
        if let Some(title) = &payload.title {
            note.title = title.clone();
        }
        if let Some(content) = &payload.content {
            note.content = Some(content.clone());
        }

        // Jalankan query UPDATE
        sqlx
            ::query("UPDATE notes SET title = ?, content = ? WHERE id = ? AND user_id = ?")
            .bind(&note.title)
            .bind(&note.content)
            .bind(id)
            .bind(user_id)
            .execute(&self.db_pool).await?;

        Ok(Some(note))
    }

    async fn delete_by_id_and_user_id(&self, id: u32, user_id: u32) -> AppResult<u64> {
        let result = sqlx
            ::query("DELETE FROM notes WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.db_pool).await?;
        Ok(result.rows_affected())
    }
}