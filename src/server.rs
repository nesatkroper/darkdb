// use crate::{Database, db::DbError};
// use axum::{
//     Router,
//     extract::{Json, State},
//     http::StatusCode,
//     routing::post,
// };
// use serde::Deserialize;
// use std::{net::SocketAddr, sync::Arc};
// use tower_http::auth::AsyncRequireAuthorizationLayer;

// #[derive(Clone)]
// struct AppState {
//     db: Arc<Database>,
// }

// #[derive(Debug, Deserialize)]
// struct CreateRequest {
//     collection: String,
//     data: serde_json::Value,
//     ttl: Option<i64>,
// }

// async fn insert(
//     State(state): State<AppState>,
//     Json(payload): Json<CreateRequest>,
// ) -> Result<Json<String>, StatusCode> {
//     let col = state
//         .db
//         .collection(&payload.collection)
//         .map_err(|_| StatusCode::NOT_FOUND)?;
//     let doc = col
//         .insert(payload.data, payload.ttl)
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//     Ok(Json(doc.id))
// }

// pub async fn start_server(db: Database) {
//     let state = AppState { db: Arc::new(db) };

//     let app = Router::new()
//         .route("/insert", post(insert))
//         .layer(AsyncRequireAuthorizationLayer::basic("admin", "secret"))
//         .with_state(state);

//     let addr = SocketAddr::from(([0, 0, 0, 0], 4141));
//     tracing::info!("Server listening on {}", addr);
//     axum::Server::bind(&addr)
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// }
