use async_trait::async_trait;
use sqlx::MySqlPool;

use crate::{
    domain::{
        models::note::{NewNote, Note, UpdateNotePayload},
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
    async fn create(&self, new_note: &NewNote) -> AppResult<Note> {
        let insert_result = sqlx::query(
            "INSERT INTO notes (title, content, user_id) VALUES (?, ?, ?)",
        )
        .bind(&new_note.title)
        .bind(&new_note.content)
        .bind(new_note.user_id)
        .execute(&self.db_pool)
        .await?;

        let new_id = insert_result.last_insert_id() as u32;

        let created_note = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?")
            .bind(new_id)
            .fetch_one(&self.db_pool)
            .await?;

        Ok(created_note)
    }

    async fn find_all(&self, user_id: u32) -> AppResult<Vec<Note>> {
        let notes = sqlx::query_as::<_, Note>(
            "SELECT * FROM notes WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?;
        Ok(notes)
    }

    async fn find_by_id(&self, id: u32, user_id: u32) -> AppResult<Option<Note>> {
        let note = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(note)
    }

    async fn update(
        &self,
        id: u32,
        user_id: u32,
        payload: &UpdateNotePayload,
    ) -> AppResult<Option<Note>> {
        // First, retrieve the existing note to ensure correct ownership
        let mut note = match self.find_by_id(id, user_id).await? {
            Some(note) => note,
            None => return Ok(None), // If not found or not owned, return None
        };

        // Update fields if they exist in the payload
        if let Some(title) = &payload.title {
            note.title = title.clone();
        }
        if let Some(content) = &payload.content {
            note.content = Some(content.clone());
        }

        // Execute the UPDATE query
        sqlx::query("UPDATE notes SET title = ?, content = ? WHERE id = ? AND user_id = ?")
            .bind(&note.title)
            .bind(&note.content)
            .bind(id)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;

        Ok(Some(note))
    }

    async fn delete(&self, id: u32, user_id: u32) -> AppResult<u64> {
        let result = sqlx::query("DELETE FROM notes WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected())
    }
}