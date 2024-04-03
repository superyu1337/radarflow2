use std::{sync::Arc, path::PathBuf};

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State},
    response::Response,
    routing::get,
    Router,
};

use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::comms::RadarData;

#[derive(Clone)]
struct AppState {
    data_lock: Arc<RwLock<RadarData>>
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let clone = state.clone();
    ws.on_upgrade(|socket| handle_socket(socket, clone))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if msg == Message::Text("requestInfo".to_string()) {
                let str = {
                    let data = state.data_lock.read().await;

                    match serde_json::to_string(&*data) {
                        Ok(json) => json,
                        Err(e) => {
                            log::error!("Could not serialize data into json: {}", e.to_string());
                            log::error!("Sending \"error\" instead");
                            "error".to_string()
                        },
                    }
                };

                //println!("{str}");

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
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app.into_make_service())
        .await?;

    Ok(())
}