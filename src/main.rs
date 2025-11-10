mod db;
mod models;
mod handlers;
mod routes;
mod config; // Daftarkan modul config baru
mod error; // Untuk AppError dan AppResult

use sqlx::MySqlPool;
use std::sync::Arc;
use crate::routes::create_router;

// Impor fungsi config
use crate::config::{
    // Config,
    load_config,
};

#[derive(Clone)]
pub struct AppState {
    db_pool: MySqlPool,
    // config: Arc<Config>, // Simpan juga config di AppState (opsional, tapi bagus)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    // Muat semua config dari .env ke struct
    let config = load_config();

    // Kirim config ke fungsi 'create_pool'
    let pool = db::create_pool(&config).await;

    // Buat AppState
    let app_state = Arc::new(AppState {
        db_pool: pool,
        // config: Arc::new(config), // Simpan config di state
    });

    // Buat router dengan state
    let app = create_router(app_state);

    // Jalankan server di port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Server berjalan di http://0.0.0.0:3000");

    // Jalankan server dengan listener yang sudah dibuat
    axum::serve(listener, app).await.unwrap();
}
