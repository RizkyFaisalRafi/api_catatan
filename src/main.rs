mod db;
mod models;
mod handlers;
mod routes;
mod config; // Daftarkan modul config baru
mod error; // Untuk AppError dan AppResult
mod auth; // Untuk middleware autentikasi
mod extractor; // Untuk extractor Json<T>

use sqlx::MySqlPool;
use std::sync::Arc;
use crate::routes::create_router;
use redis::Client as RedisClient;

// Impor fungsi config
use crate::config::{ Config, load_config };

#[derive(Clone)]
pub struct AppState {
    pub db_pool: MySqlPool,
    pub config: Arc<Config>, // Simpan juga config di AppState (opsional, tapi bagus)
    pub redis_client: RedisClient, // Simpan client Redis di AppState
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    // Muat semua config dari .env ke struct
    let config = load_config();

    // Kirim config ke fungsi 'create_pool'
    let pool = db::create_pool(&config).await;

    // Bangun Redis connection string
    // Format: redis://[username:password@]host[:port][/database]
    let redis_url = format!(
        "redis://:{}@{}:{}/{}",
        config.redis_password, // Asumsi password ada, walau kosong
        config.redis_host,
        config.redis_port,
        config.redis_db
    );

    // Buat client Redis menggunakan URL yang baru dibuat
    let redis_client = RedisClient::open(redis_url) // Gunakan 'redis_url'
        .expect("Gagal terhubung ke Redis");

    // Buat AppState
    let app_state = Arc::new(AppState {
        db_pool: pool,
        config: Arc::new(config),
        redis_client: redis_client,
    });

    // Buat router dengan state
    let app = create_router(app_state);

    // Jalankan server di port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Server berjalan di http://0.0.0.0:3000");

    // Jalankan server dengan listener yang sudah dibuat
    axum::serve(listener, app).await.unwrap();
}
