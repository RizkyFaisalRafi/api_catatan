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
use crate::presentation::{
    // Akses presentation/auth/mod.rs
    // auth_middleware::auth_middleware,
    auth::{
        admin_auth_middleware::admin_auth_middleware,
        auth_middleware::auth_middleware,
    },
    // middleware::auth_middleware,
    handlers::{note_handler, user_handler},
};

pub fn create_router(state: Arc<AppState>) -> Router {
    // 1. Definisikan rute publik (tidak perlu login)
    let public_routes = Router::new()
        .route("/auth/register", post(user_handler::register))
        .route("/auth/login", post(user_handler::login));

    // 2. Definisikan rute admin (perlu login + peran admin)
    let admin_routes = Router::new()
        .route("/users", get(user_handler::get_all_users))
        .route_layer(middleware::from_fn(admin_auth_middleware));

    // 3. Definisikan rute terproteksi (hanya perlu login)
    let protected_routes = Router::new()
        .merge(admin_routes) // Gabungkan rute admin di sini
        // --- Endpoint Notes ---
        .route("/notes", post(note_handler::create_note).get(note_handler::get_all_notes))
        .route(
            "/notes/:id",
            get(note_handler::get_note_by_id).put(note_handler::update_note).delete(note_handler::delete_note)
        )
        // --- Endpoint Users ---
        .route("/auth/profile", get(user_handler::get_profile))
        .route("/auth/logout", post(user_handler::logout))
        // --- Terapkan Middleware ---
        .route_layer(
            middleware::from_fn_with_state(
                state.clone(),
                auth_middleware // Middleware utama untuk semua rute terproteksi
            )
        );

    // 4. Gabungkan semua router
    Router::new().merge(public_routes).merge(protected_routes).with_state(state)

}
