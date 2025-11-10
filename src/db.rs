use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use crate::config::Config; // Impor struct Config kita

// fungsi ini menerima 'config'
pub async fn create_pool(config: &Config) -> MySqlPool {
    // Bangun connection string secara manual
    let db_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        config.db_username,
        config.db_password,
        config.db_host,
        config.db_port,
        config.db_name
    );

    // Buat pool koneksi menggunakan string yang baru kita buat
    MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&db_url) // Gunakan db_url hasil build
        .await
        .expect("Gagal membuat database pool")
}