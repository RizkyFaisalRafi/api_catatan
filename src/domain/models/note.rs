use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Ini adalah Entitas Domain.
// Atribut `FromRow` secara teknis adalah pelanggaran kecil terhadap Clean Architecture
// karena bergantung pada `sqlx`, namun ini adalah kompromi pragmatis yang umum di Rust.

// Struct ini mewakili data di tabel `notes`
// `FromRow` -> untuk memetakan hasil query DB ke struct
// `Serialize` -> untuk mengubah struct ke JSON (respons)
#[derive(Debug, FromRow, Serialize)]
pub struct Note {
    pub id: u32,
    pub user_id: u32,
    pub title: String,
    pub content: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

// Struct ini untuk data note baru yang akan disimpan ke DB
// Ini berbeda dari payload karena menyertakan user_id
#[derive(Debug, Deserialize)]
pub struct NewNote {
    pub user_id: u32,
    pub title: String,
    pub content: Option<String>,
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