// src/auth/mod.rs
use axum::{
    RequestPartsExt, async_trait,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Basic},
};
use bcrypt::verify;
use std::collections::HashMap;
use tracing::warn;

#[derive(Clone)]
pub struct AuthConfig {
    pub users: HashMap<String, String>, // username -> bcrypt hash
}

pub struct AuthenticatedUser {
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(auth_config) = parts
            .extract_with_state::<State<AuthConfig>, _>(state)
            .await
            .map_err(|err| err.into_response())?;

        let TypedHeader(Authorization(basic)) = parts
            .extract::<TypedHeader<Authorization<Basic>>>()
            .await
            .map_err(|err| err.into_response())?;

        match auth_config.users.get(basic.username()) {
            Some(hash) => {
                if verify(basic.password(), hash).unwrap_or(false) {
                    Ok(AuthenticatedUser {
                        username: basic.username().to_string(),
                    })
                } else {
                    warn!("Invalid password for user: {}", basic.username());
                    Err((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response())
                }
            }
            None => {
                warn!("Unknown user: {}", basic.username());
                Err((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response())
            }
        }
    }
}
