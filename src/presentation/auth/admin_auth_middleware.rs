use axum::{
    body::Body,
    extract::Request,
    http::Response,
    middleware::Next,
    response::IntoResponse,
    Extension,
};

use crate::{
    domain::models::user::{Role, TokenClaims},
    utils::error::AppError,
};

/// Middleware untuk memvalidasi bahwa pengguna memiliki peran 'Admin'.
/// Middleware ini harus dijalankan SETELAH `auth_middleware` karena
/// bergantung pada `TokenClaims` yang diekstrak olehnya.
pub async fn admin_auth_middleware(
    Extension(claims): Extension<TokenClaims>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    // Cek apakah peran pengguna adalah Admin
    if claims.role != Role::Admin.as_ref() {
        return AppError::Forbidden.into_response();
    }

    // Jika admin, lanjutkan ke handler berikutnya
    next.run(req).await
}