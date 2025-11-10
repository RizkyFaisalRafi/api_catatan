use serde::{Deserialize, Serialize};
// use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

// Struct ini mewakili data di tabel `notes`
// `FromRow` -> untuk memetakan hasil query DB ke struct
// `Serialize` -> untuk mengubah struct ke JSON (respons)
#[derive(Debug, FromRow, Serialize)]
pub struct Note {
    pub id: u32,
    pub title: String,
    pub content: Option<String>,
    // pub created_at: Option<NaiveDateTime>,
    pub created_at: Option<DateTime<Utc>>,
}

// Struct ini untuk payload 'Create Note' (data dari user)
// `Deserialize` -> untuk mengubah JSON (request) ke struct
#[derive(Debug, Deserialize)]
pub struct CreateNotePayload {
    pub title: String,
    pub content: Option<String>,
}

// Struct ini untuk payload 'Update Note'
// gunakan Option<> karena user mungkin hanya ingin
// update judulnya saja atau kontennya saja.
#[derive(Debug, Deserialize)]
pub struct UpdateNotePayload {
    pub title: Option<String>,
    pub content: Option<String>,
}