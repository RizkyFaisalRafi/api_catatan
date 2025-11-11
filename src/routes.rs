use axum::{
    routing::{
        get,
        post,
        // , put, delete
    },
    Router,
    middleware,
};
use std::sync::Arc;
use crate::AppState;
use crate::handlers::{ note_handler, user_handler };
use crate::auth::auth_middleware;

pub fn create_router(state: Arc<AppState>) -> Router {
    // 1. Definisikan rute publik (tidak perlu login)
    let public_routes = Router::new()
        .route("/auth/register", post(user_handler::register))
        .route("/auth/login", post(user_handler::login));

    // 2. Definisikan rute terproteksi (perlu login)
    let protected_routes = Router::new()
        // --- Endpoint Notes ---
        .route("/notes", post(note_handler::create_note).get(note_handler::get_all_notes))
        .route(
            "/notes/{id}",
            get(note_handler::get_note_by_id)
                .put(note_handler::update_note)
                .delete(note_handler::delete_note)
        )
        // --- Endpoint Logout ---
        .route("/auth/logout", post(user_handler::logout)) // <-- TAMBAHKAN INI
        // --- Terapkan Middleware ---
        .route_layer(
            middleware::from_fn_with_state(
                state.clone(),
                auth_middleware // Pasang middleware di sini
            )
        );

    // 3. Gabungkan semua router
    Router::new().merge(public_routes).merge(protected_routes).with_state(state)

    // Router::new()
    //     // Gabungkan 'create' (POST) dan 'get_all' (GET)
    //     .route("/notes", post(note_handler::create_note).get(note_handler::get_all_notes))
    //     // Gabungkan 'get_by_id' (GET), 'update' (PUT), dan 'delete' (DELETE)
    //     .route(
    //         "/notes/{id}",
    //         get(note_handler::get_note_by_id)
    //             .put(note_handler::update_note)
    //             .delete(note_handler::delete_note)
    //     )
    //     // --- RUTE UNTUK AUTENTIKASI ---
    //     .route("/auth/register", post(user_handler::register))
    //     .route("/auth/login", post(user_handler::login))
    //     // --- AKHIR RUTE BARU ---
    //     .with_state(state) // Injeksikan state (db pool)
}
