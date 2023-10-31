use std::{sync::Arc, path::PathBuf};

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State},
    response::Response,
    routing::get,
    Router,
};

use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::structs::communication::RadarData;

#[derive(Clone)]
struct AppState {
    data_lock: Arc<RwLock<RadarData>>
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if msg == Message::Text("requestInfo".to_string()) {
                let data = state.data_lock.read().await;

                let str = {
                    match serde_json::to_string(&*data) {
                        Ok(json) => json,
                        Err(e) => {
                            log::error!("Could not serialize data into json: {}", e.to_string());
                            log::error!("Sending \"error\" instead");
                            "error".to_string()
                        },
                    }
                };

                if socket.send(Message::Text(str)).await.is_err() {
                    // client disconnected
                    return;
                }
            }
        } else {
            // client disconnected
            return;
        }
    }
}

pub async fn run(path: PathBuf, port: u16, data_lock: Arc<RwLock<RadarData>>) -> anyhow::Result<()> {
    let app = Router::new()
        .nest_service("/", ServeDir::new(path))
        .route("/ws", get(ws_handler))
        .with_state(AppState { data_lock });

    let address = format!("0.0.0.0:{}", port);

    axum::Server::bind(&address.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}