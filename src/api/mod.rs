// use axum::{
//     Json, Router,
//     body::Body,
//     extract::{Path, State},
//     http::{Request, StatusCode},
//     middleware::{self, Next},
//     response::{IntoResponse, Response},
//     routing::{delete, get, post, put},
// };
// use serde_json::Value;
// use std::sync::Arc;
// use tracing::info;

// use crate::db::{Database, DbError, Document};

// mod auth;
// pub use auth::{AuthConfig, AuthenticatedUser};

// #[derive(Debug, thiserror::Error)]
// pub enum ApiError {
//     #[error("Database error: {0}")]
//     DbError(#[from] DbError),
//     #[error("Invalid JSON: {0}")]
//     JsonError(#[from] serde_json::Error),
//     #[error("Authentication error")]
//     AuthError,
// }

// impl IntoResponse for ApiError {
//     fn into_response(self) -> Response {
//         let status = match self {
//             ApiError::DbError(DbError::NotFound) => StatusCode::NOT_FOUND,
//             ApiError::DbError(DbError::CollectionNotFound) => StatusCode::NOT_FOUND,
//             ApiError::AuthError => StatusCode::UNAUTHORIZED,
//             _ => StatusCode::INTERNAL_SERVER_ERROR,
//         };
//         (status, self.to_string()).into_response()
//     }
// }

// #[derive(Clone)]
// pub struct ApiState {
//     pub db: Arc<Database>,
//     pub auth_config: AuthConfig,
// }

// pub async fn start_server(
//     db: Database,
//     host: &str,
//     port: u16,
//     auth_config: AuthConfig,
// ) -> anyhow::Result<()> {
//     let state = ApiState {
//         db: Arc::new(db),
//         auth_config,
//     };

//     let app = Router::new()
//         .route("/collections/:name", post(create_collection))
//         .route("/collections/:name", delete(delete_collection))
//         .route("/collections/:name/documents", post(insert_document))
//         .route("/collections/:name/documents", get(list_documents))
//         .route("/collections/:name/documents/:id", get(get_document))
//         .route("/collections/:name/documents/:id", put(update_document))
//         .route("/collections/:name/documents/:id", delete(delete_document))
//         .layer(middleware::from_fn(auth_middleware))
//         .with_state(state);

//     let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
//     info!("Server listening on {}:{}", host, port);
//     axum::serve(listener, app).await?;
//     Ok(())
// }

// async fn auth_middleware(request: Request<Body>, next: Next<Body>) -> Result<Response, Response> {
//     // Extract auth config from extensions
//     let auth_config = request
//         .extensions()
//         .get::<AuthConfig>()
//         .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing auth config").into_response())?;

//     // Extract authorization header
//     let auth_header = request
//         .headers()
//         .get(axum::http::header::AUTHORIZATION)
//         .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header").into_response())?;

//     let auth_str = auth_header
//         .to_str()
//         .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid authorization header").into_response())?;

//     if !auth_str.starts_with("Basic ") {
//         return Err((StatusCode::UNAUTHORIZED, "Invalid authorization scheme").into_response());
//     }

//     let credentials = auth_str.trim_start_matches("Basic ");
//     let decoded = base64::decode(credentials)
//         .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid basic auth format").into_response())?;

//     let decoded_str = String::from_utf8(decoded)
//         .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid UTF-8 in credentials").into_response())?;

//     let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
//     if parts.len() != 2 {
//         return Err((StatusCode::UNAUTHORIZED, "Invalid basic auth format").into_response());
//     }

//     let username = parts[0];
//     let password = parts[1];

//     match auth_config.users.get(username) {
//         Some(hashed_password) if verify(password, hashed_password).unwrap_or(false) => {
//             let mut req = request;
//             req.extensions_mut().insert(AuthenticatedUser {
//                 username: username.to_string(),
//             });
//             Ok(next.run(req).await)
//         }
//         _ => Err((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()),
//     }
// }

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

use crate::db::{Database, DbError, Document};

mod auth;
pub use auth::{AuthConfig, AuthenticatedUser};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    DbError(#[from] DbError),
    #[error("Invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Authentication error")]
    AuthError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::DbError(DbError::NotFound) => StatusCode::NOT_FOUND,
            ApiError::DbError(DbError::CollectionNotFound) => StatusCode::NOT_FOUND,
            ApiError::AuthError => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Database>,
    pub auth_config: AuthConfig,
}

pub async fn start_server(
    db: Database,
    host: &str,
    port: u16,
    auth_config: AuthConfig,
) -> anyhow::Result<()> {
    let state = ApiState {
        db: Arc::new(db),
        auth_config,
    };

    let app = Router::new()
        .route("/collections/:name", post(create_collection))
        .route("/collections/:name", delete(delete_collection))
        .route("/collections/:name/documents", post(insert_document))
        .route("/collections/:name/documents", get(list_documents))
        .route("/collections/:name/documents/:id", get(get_document))
        .route("/collections/:name/documents/:id", put(update_document))
        .route("/collections/:name/documents/:id", delete(delete_document))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    info!("Server listening on {}:{}", host, port);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn auth_middleware(request: Request<Body>, next: Next) -> Result<Response, Response> {
    // Extract auth config from extensions
    let auth_config = request
        .extensions()
        .get::<AuthConfig>()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Missing auth config").into_response())?;

    // Extract authorization header
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header").into_response())?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid authorization header").into_response())?;

    if !auth_str.starts_with("Basic ") {
        return Err((StatusCode::UNAUTHORIZED, "Invalid authorization scheme").into_response());
    }

    let credentials = auth_str.trim_start_matches("Basic ");
    let decoded = base64::decode(credentials)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid basic auth format").into_response())?;

    let decoded_str = String::from_utf8(decoded)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid UTF-8 in credentials").into_response())?;

    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err((StatusCode::UNAUTHORIZED, "Invalid basic auth format").into_response());
    }

    let username = parts[0];
    let password = parts[1];

    match auth_config.users.get(username) {
        Some(hashed_password) if verify(password, hashed_password).unwrap_or(false) => {
            let mut req = request;
            req.extensions_mut().insert(AuthenticatedUser {
                username: username.to_string(),
            });
            Ok(next.run(req).await)
        }
        _ => Err((StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()),
    }
}

async fn create_collection(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.db.create_collection(&name)?;
    Ok(StatusCode::CREATED)
}

#[axum::debug_handler]
async fn delete_collection(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.db.drop_collection(&name)?;
    Ok(StatusCode::NO_CONTENT)
}

#[axum::debug_handler]
async fn insert_document(
    State(state): State<ApiState>,
    Path(collection): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Json<Document>, ApiError> {
    let col = state.db.collection(&collection)?;
    let doc = col.insert(payload, None)?;
    Ok(Json(doc))
}

#[axum::debug_handler]
async fn list_documents(
    State(state): State<ApiState>,
    Path(collection): Path<String>,
) -> Result<Json<Vec<Document>>, ApiError> {
    let col = state.db.collection(&collection)?;
    let docs = col.find_all()?;
    Ok(Json(docs))
}

#[axum::debug_handler]
async fn get_document(
    State(state): State<ApiState>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<Json<Document>, ApiError> {
    let col = state.db.collection(&collection)?;
    let doc = col.find(&id)?.ok_or(DbError::NotFound)?;
    Ok(Json(doc))
}

#[axum::debug_handler]
async fn update_document(
    State(state): State<ApiState>,
    Path((collection, id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Result<Json<Document>, ApiError> {
    let col = state.db.collection(&collection)?;
    let doc = col.update(&id, payload)?;
    Ok(Json(doc))
}

#[axum::debug_handler]
async fn delete_document(
    State(state): State<ApiState>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let col = state.db.collection(&collection)?;
    col.delete(&id)?;
    Ok(StatusCode::NO_CONTENT)
}

// use axum::{
//     Json, Router,
//     body::Body,
//     extract::{Path, State},
//     http::{Request, StatusCode},
//     middleware::{self, Next},
//     response::{IntoResponse, Response},
//     routing::{delete, get, post, put},
// };
// use serde_json::Value;
// use std::sync::Arc;
// use tracing::info;

// use crate::db::{Database, DbError, Document};

// mod auth;
// pub use auth::{AuthConfig, AuthenticatedUser};

// #[derive(Debug, thiserror::Error)]
// pub enum ApiError {
//     #[error("Database error: {0}")]
//     DbError(#[from] DbError),
//     #[error("Invalid JSON: {0}")]
//     JsonError(#[from] serde_json::Error),
//     #[error("Authentication error")]
//     AuthError,
// }

// impl IntoResponse for ApiError {
//     fn into_response(self) -> axum::response::Response {
//         let status = match self {
//             ApiError::DbError(DbError::NotFound) => StatusCode::NOT_FOUND,
//             ApiError::DbError(DbError::CollectionNotFound) => StatusCode::NOT_FOUND,
//             ApiError::AuthError => StatusCode::UNAUTHORIZED,
//             _ => StatusCode::INTERNAL_SERVER_ERROR,
//         };
//         (status, self.to_string()).into_response()
//     }
// }

// #[derive(Clone)]
// pub struct ApiState {
//     pub db: Arc<Database>,
//     pub auth_config: AuthConfig,
// }

// pub async fn start_server(
//     db: Database,
//     host: &str,
//     port: u16,
//     auth_config: AuthConfig,
// ) -> anyhow::Result<()> {
//     let state = ApiState {
//         db: Arc::new(db),
//         auth_config,
//     };

//     // Create the router with all routes
//     let app = Router::new()
//         .route("/collections", post(create_collection))
//         .route("/collections/:name", delete(delete_collection))
//         .route("/collections/:name/documents", post(insert_document))
//         .route("/collections/:name/documents", get(list_documents))
//         .route("/collections/:name/documents/:id", get(get_document))
//         .route("/collections/:name/documents/:id", put(update_document))
//         .route("/collections/:name/documents/:id", delete(delete_document))
//         // Add the auth layer middleware
//         .layer(middleware::from_fn_with_state(
//             state.clone(),
//             |state: ApiState, request: Request<Body>, next: Next<Body>| async move {
//                 let (mut parts, body) = request.into_parts();
//                 parts.extensions.insert(state.auth_config);
//                 next.run(Request::from_parts(parts, body)).await
//             },
//         ))
//         .with_state(state);

//     // Start the server
//     let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
//     info!("Server listening on {}:{}", host, port);
//     axum::serve(listener, app).await?;
//     Ok(())
// }

// // pub async fn start_server(
// //     db: Database,
// //     host: &str,
// //     port: u16,
// //     auth_config: AuthConfig,
// // ) -> anyhow::Result<()> {
// //     let state = ApiState {
// //         db: Arc::new(db),
// //         auth_config,
// //     };

// //     let app = Router::new()
// //         .route("/collections", post(create_collection))
// //         .route("/collections/:name", delete(delete_collection))
// //         .route("/collections/:name/documents", post(insert_document))
// //         .route("/collections/:name/documents", get(list_documents))
// //         .route("/collections/:name/documents/:id", get(get_document))
// //         .route("/collections/:name/documents/:id", put(update_document))
// //         .route("/collections/:name/documents/:id", delete(delete_document))
// //         .layer(middleware::from_fn(auth_layer))
// //         .with_state(state);

// //     let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
// //     info!("Server listening on {}:{}", host, port);
// //     axum::serve(listener, app).await?;
// //     Ok(())
// // }

// // async fn auth_layer(
// //     State(state): State<ApiState>,
// //     request: Request<Body>,
// //     next: Next<Body>,
// // ) -> Response {
// //     let (mut parts, body) = request.into_parts();
// //     parts.extensions.insert(state.auth_config);
// //     next.run(Request::from_parts(parts, body)).await
// // }

// async fn create_collection(
//     State(state): State<ApiState>,
//     Path(name): Path<String>,
// ) -> Result<StatusCode, ApiError> {
//     state.db.create_collection(&name)?;
//     Ok(StatusCode::CREATED)
// }

// // ... other handler functions remain exactly the same ...

// async fn delete_collection(
//     State(state): State<ApiState>,
//     Path(name): Path<String>,
// ) -> Result<StatusCode, ApiError> {
//     state.db.drop_collection(&name)?;
//     Ok(StatusCode::NO_CONTENT)
// }

// async fn insert_document(
//     State(state): State<ApiState>,
//     Path(collection): Path<String>,
//     Json(payload): Json<Value>,
// ) -> Result<Json<Document>, ApiError> {
//     let col = state.db.collection(&collection)?;
//     let doc = col.insert(payload, None)?;
//     Ok(Json(doc))
// }

// async fn list_documents(
//     State(state): State<ApiState>,
//     Path(collection): Path<String>,
// ) -> Result<Json<Vec<Document>>, ApiError> {
//     let col = state.db.collection(&collection)?;
//     let docs = col.find_all()?;
//     Ok(Json(docs))
// }

// async fn get_document(
//     State(state): State<ApiState>,
//     Path((collection, id)): Path<(String, String)>,
// ) -> Result<Json<Document>, ApiError> {
//     let col = state.db.collection(&collection)?;
//     let doc = col.find(&id)?.ok_or(DbError::NotFound)?;
//     Ok(Json(doc))
// }

// async fn update_document(
//     State(state): State<ApiState>,
//     Path((collection, id)): Path<(String, String)>,
//     Json(payload): Json<Value>,
// ) -> Result<Json<Document>, ApiError> {
//     let col = state.db.collection(&collection)?;
//     let doc = col.update(&id, payload)?;
//     Ok(Json(doc))
// }

// async fn delete_document(
//     State(state): State<ApiState>,
//     Path((collection, id)): Path<(String, String)>,
// ) -> Result<StatusCode, ApiError> {
//     let col = state.db.collection(&collection)?;
//     col.delete(&id)?;
//     Ok(StatusCode::NO_CONTENT)
// }
