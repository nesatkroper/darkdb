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
                    (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
                }
            }
            None => (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(),
        }
    }
}
