// Layering
mod domain;
mod application;
mod infrastructure;
mod presentation;
mod utils; // <-- DEKLARASIKAN MODUL UTILS

// Impor dependensi yang dibutuhkan
use crate::domain::repositories::{note_repository::DynNoteRepository, user_repository::DynUserRepository};
use crate::infrastructure::repositories::{note_repository_impl::NoteRepositoryImpl, user_repository_impl::UserRepositoryImpl};
use crate::presentation::routes::create_router;
use crate::utils::{config::{Config, load_config}, db};
use sqlx::MySqlPool;
use std::sync::Arc;
use redis::Client as RedisClient;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: MySqlPool,
    pub config: Arc<Config>, // Simpan juga config di AppState (opsional, tapi bagus)
    pub redis_client: RedisClient, // Simpan client Redis di AppState
    pub user_repo: DynUserRepository,
    pub note_repo: DynNoteRepository,
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

    // Inisialisasi Repositories
    let user_repo = Arc::new(UserRepositoryImpl::new(pool.clone())) as DynUserRepository;
    let note_repo = Arc::new(NoteRepositoryImpl::new(pool.clone())) as DynNoteRepository;

    // Buat AppState
    let app_state = Arc::new(AppState {
        db_pool: pool,
        config: Arc::new(config),
        redis_client: redis_client,
        user_repo,
        note_repo,
    });

    // Buat router dengan state
    let app = create_router(app_state);

    // Jalankan server di port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Server berjalan di http://0.0.0.0:3000");

    // Jalankan server dengan listener yang sudah dibuat
    axum::serve(listener, app).await.unwrap();
}
