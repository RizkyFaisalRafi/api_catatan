use axum::{
    extract::FromRequest,
    http::Request,
    Json,
};
use serde::de::DeserializeOwned;
use crate::utils::error::AppError;
use async_trait::async_trait;
use axum::body::Body;
// Buat struct "newtype" yang membungkus Axum::Json
// Beri tahu Axum untuk menggunakan AppError sebagai tipe Rejection
// Ini akan OTOMATIS memanggil `impl From<JsonRejection> for AppError`
// yang sudah Anda tulis di 'error.rs'
pub struct ApiJson<T>(
    // Ekstraktor yang sebenarnya kita bungkus
    pub Json<T>
);
#[async_trait]
impl<S, T> FromRequest<S, Body> for ApiJson<T> // Gunakan Body secara spesifik
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        Ok(Self(Json(value)))
    }
}