use axum::{
    async_trait,
    extract::FromRequestParts,
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
    pub users: HashMap<String, String>, // username -> hashed_password
}

#[derive(Clone)] // Added Clone derive
pub struct AuthenticatedUser {
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_config = parts
            .extensions
            .get::<AuthConfig>()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing auth config").into_response())?;

        let auth = parts
            .extract::<TypedHeader<Authorization<Basic>>>()
            .await
            .map_err(|err| err.into_response())?;

        let TypedHeader(Authorization(basic)) = auth;

        match auth_config.users.get(basic.username()) {
            Some(hashed_password) if verify(basic.password(), hashed_password).unwrap_or(false) => {
                Ok(AuthenticatedUser {
                    username: basic.username().to_string(),
                })
            }
            _ => Err((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()),
        }
    }
}

// use axum::{
//     async_trait,
//     extract::FromRequestParts,
//     http::{StatusCode, request::Parts},
//     response::IntoResponse,
// };
// use axum_extra::{
//     TypedHeader,
//     headers::{Authorization, authorization::Basic},
// };
// use bcrypt::verify;
// use std::collections::HashMap;

// #[derive(Clone)]
// pub struct AuthConfig {
//     pub users: HashMap<String, String>, // username -> hashed_password
// }

// pub struct AuthenticatedUser {
//     pub username: String,
// }

// #[async_trait]
// impl<S> FromRequestParts<S> for AuthenticatedUser
// where
//     S: Send + Sync,
// {
//     type Rejection = axum::response::Response;

//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         // 1. Get auth config from extensions
//         let auth_config = parts
//             .extensions
//             .get::<AuthConfig>()
//             .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing auth config").into_response())?;

//         // 2. Extract basic auth header
//         let auth = parts
//             .extract::<TypedHeader<Authorization<Basic>>>()
//             .await
//             .map_err(|err| err.into_response())?;

//         let TypedHeader(Authorization(basic)) = auth;

//         // 3. Verify credentials
//         match auth_config.users.get(basic.username()) {
//             Some(hashed_password) if verify(basic.password(), hashed_password).unwrap_or(false) => {
//                 Ok(AuthenticatedUser {
//                     username: basic.username().to_string(),
//                 })
//             }
//             _ => Err((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()),
//         }
//     }
// }

// use axum::{
//     async_trait,
//     extract::FromRequestParts,
//     http::{StatusCode, request::Parts},
//     response::{IntoResponse, Response},
// };
// use axum_extra::{
//     TypedHeader,
//     headers::{Authorization, authorization::Basic},
// };
// use bcrypt::verify;
// use std::collections::HashMap;

// #[derive(Clone)]
// pub struct AuthConfig {
//     pub users: HashMap<String, String>, // username -> hashed_password
// }

// pub struct AuthenticatedUser {
//     pub username: String,
// }

// #[async_trait]
// impl<S> FromRequestParts<S> for AuthenticatedUser
// where
//     S: Send + Sync,
// {
//     type Rejection = Response;

//     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
//         // 1. Get auth config from extensions
//         let auth_config = parts
//             .extensions
//             .get::<AuthConfig>()
//             .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing auth config").into_response())?;

//         // 2. Extract basic auth header
//         let auth = parts
//             .extract::<TypedHeader<Authorization<Basic>>>()
//             .await
//             .map_err(|err| err.into_response())?;

//         let TypedHeader(Authorization(basic)) = auth;

//         // 3. Verify credentials
//         match auth_config.users.get(basic.username()) {
//             Some(hashed_password) if verify(basic.password(), hashed_password).unwrap_or(false) => {
//                 Ok(AuthenticatedUser {
//                     username: basic.username().to_string(),
//                 })
//             }
//             _ => Err((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()),
//         }
//     }
// }

// // use axum::{
// //     async_trait,
// //     extract::FromRequestParts,
// //     http::{StatusCode, request::Parts},
// //     response::{IntoResponse, Response},
// // };
// // use axum_extra::{
// //     TypedHeader,
// //     headers::{Authorization, authorization::Basic},
// // };
// // use bcrypt::verify;
// // use std::collections::HashMap;

// // #[derive(Clone)]
// // pub struct AuthConfig {
// //     pub users: HashMap<String, String>, // username -> hashed_password
// // }

// // pub struct AuthenticatedUser {
// //     pub username: String,
// // }

// // #[async_trait]
// // impl<S> FromRequestParts<S> for AuthenticatedUser
// // where
// //     S: Send + Sync,
// // {
// //     type Rejection = Response;

// //     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
// //         // Get auth config from extensions
// //         let auth_config = parts
// //             .extensions
// //             .get::<AuthConfig>()
// //             .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing auth config").into_response())?;

// //         // Extract auth header
// //         let auth_header = parts
// //             .extract::<TypedHeader<Authorization<Basic>>>()
// //             .await
// //             .map_err(|err| err.into_response())?;

// //         let TypedHeader(Authorization(basic)) = auth_header;

// //         // Verify credentials
// //         match auth_config.users.get(basic.username()) {
// //             Some(hashed_password) if verify(basic.password(), hashed_password).unwrap_or(false) => {
// //                 Ok(AuthenticatedUser {
// //                     username: basic.username().to_string(),
// //                 })
// //             }
// //             _ => Err((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()),
// //         }
// //     }
// // }
