use serde::Deserialize;

// #[derive(Deserialize)] akan memberi tahu 'envy' cara
// mengisi struct ini dari variabel .env
// 'envy' otomatis mengubah DB_HOST menjadi db_host
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    // MySQL Configuration
    pub db_host: String,
    pub db_port: u16,
    pub db_username: String,
    pub db_password: String,
    pub db_name: String,

//     // Redis Configuration
//     pub redis_host: String,
//     pub redis_port: u16,
//     pub redis_password: String,
//     pub redis_db: i64,
//     // pub redis_timeout: String, // envy belum bisa parse "5s", load sbg String

//     // SMTP Configuration
//     pub smtp_host: String,
//     pub smtp_port: u16,
//     pub smtp_email: String,
//     pub smtp_password: String,
//     pub smtp_name: String,
//     // pub smtp_timeout: u16,

//     // JWT Configuration
//     pub jwt_secret_key: String,
//     pub jwt_access_token_duration: String, // Load sbg String, parse nanti
//     pub jwt_refresh_token_duration: String, // Load sbg String, parse nanti
//     pub jwt_temp_token_duration: String, // Load sbg String, parse nanti
}

// Fungsi helper untuk memuat config
pub fn load_config() -> Config {
    dotenvy::dotenv().ok(); // Memuat file .env

    match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(e) => panic!("Gagal memuat konfigurasi .env: {e}"),
    }
}