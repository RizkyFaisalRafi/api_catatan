use axum::{
    routing::{
        get,
        post,
        // , put, delete
    },
    Router,
};
use std::sync::Arc;
use crate::AppState;
use crate::handlers::note_handler;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Gabungkan 'create' (POST) dan 'get_all' (GET)
        .route("/notes", post(note_handler::create_note).get(note_handler::get_all_notes))
        // Gabungkan 'get_by_id' (GET), 'update' (PUT), dan 'delete' (DELETE)
        .route(
            "/notes/{id}",
            get(note_handler::get_note_by_id)
                .put(note_handler::update_note)
                .delete(note_handler::delete_note)
        )
        .with_state(state) // Injeksikan state (db pool)
}
