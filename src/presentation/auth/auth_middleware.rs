use axum::{
    extract::State,
    http::Request,
    middleware::Next,
    response::{ Response, IntoResponse },
    body::Body,
};
use axum::http::header;
use std::sync::Arc;
use chrono::Utc;
use jsonwebtoken::{ decode, Validation, DecodingKey };
use redis::AsyncCommands; // <-- Penting untuk .exists() dan .set_ex()

use crate::{
    domain::models::user::TokenClaims,
    utils::error::AppError,
    AppState,
};

// Ini adalah middleware kita
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next
) -> Response {
    // 1. Ekstrak token dari header
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") { Some(auth_value[7..].to_string()) } else { None }
        });

    let token_str = match token {
        Some(token) => token,
        None => {
            return AppError::MissingToken.into_response();
        }
    };

    // 2. Cek apakah token ada di blacklist (di Redis)
    let mut redis_conn = match state.redis_client.get_multiplexed_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("Gagal konek Redis: {}", e);
            //     return auth_error(AppError::InvalidToken); // Anggap invalid jika Redis down
            return AppError::InvalidToken.into_response();
        }
    };

    let is_blacklisted: bool = match redis_conn.exists(&token_str).await {
        Ok(exists) => exists,
        Err(e) => {
            tracing::error!("Gagal cek Redis: {}", e);
            //     return auth_error(AppError::InvalidToken);
            return AppError::InvalidToken.into_response();
        }
    };

    if is_blacklisted {
        // return auth_error(AppError::InvalidToken); // Token sudah di-logout
        return AppError::InvalidToken.into_response();
    }

    // 3. Validasi token (signature & expiry)
    let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    let claims = match
        decode::<TokenClaims>(
            &token_str,
            &DecodingKey::from_secret(state.config.jwt_secret_key.as_ref()),
            &validation
        )
    {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            // Cek apakah error karena kedaluwarsa
            if let jsonwebtoken::errors::ErrorKind::ExpiredSignature = e.kind() {
                // return auth_error(AppError::TokenExpired);
                return AppError::TokenExpired.into_response();
            }
            //     return auth_error(AppError::InvalidToken);
            return AppError::InvalidToken.into_response();
        }
    };

    // 4. Cek expiry sekali lagi (double check)
    let now = Utc::now().timestamp();
    if claims.exp < now {
        // return auth_error(AppError::TokenExpired);
        return AppError::TokenExpired.into_response();
    }

    // 5. Simpan data token di request 'extensions' agar bisa
    //    diambil oleh handler (seperti handler /logout)
    req.extensions_mut().insert(claims);
    req.extensions_mut().insert(token_str); // Simpan juga raw token-nya

    // Lanjutkan ke handler
    next.run(req).await
}
