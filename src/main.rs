use axum::{
  extract::ws::{Message, WebSocket, WebSocketUpgrade},
  response::{Html, IntoResponse},
  routing::get,
  Json, Router,
};
use clap::Parser;
use futures::{sink::SinkExt, stream::StreamExt};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

mod viewer_html;

/// Kitbash Viewer - 3D mesh viewer with live file watching
#[derive(Parser, Debug)]
#[command(name = "kitbash-viewer")]
#[command(version, about, long_about = None)]
struct Cli {
  /// Server port
  #[arg(short, long, default_value = "8080")]
  port: u16,

  /// Bind address
  #[arg(long, default_value = "127.0.0.1")]
  host: String,

  /// Directory to watch for OBJ files
  #[arg(short, long, default_value = "scene")]
  scene_dir: PathBuf,

  /// Auto-open browser on startup
  #[arg(short, long)]
  open: bool,

  /// Show keyboard controls help
  #[arg(long)]
  help_keys: bool,

  /// Show available settings
  #[arg(long)]
  help_settings: bool,
}

#[derive(Serialize, Deserialize)]
struct FileInfo {
  name: String,
}

#[derive(Serialize)]
struct FileListResponse {
  files: Vec<FileInfo>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum FileEvent {
  Added    { filename: String },
  Modified { filename: String },
  Removed  { filename: String },
}

#[derive(Clone)]
struct AppState {
  scene_dir: PathBuf,
  tx: broadcast::Sender<FileEvent>,
}

async fn websocket_handler(
  ws: WebSocketUpgrade,
  axum::extract::State(state): axum::extract::State<AppState>,) 
    -> impl IntoResponse {
  ws.on_upgrade(move |socket| handle_socket(socket, state.tx))
}

async fn handle_socket(
    socket: WebSocket,
    tx: broadcast::Sender<FileEvent>) {
  let (mut sender, mut receiver) = socket.split();
  let mut rx = tx.subscribe();

  // Spawn a task to forward file change events to the WebSocket
  let mut send_task = tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
      let json = serde_json::to_string(&event).unwrap();
      if sender.send(Message::Text(json)).await.is_err() {
        break;
      }
    }
  });

  // Handle incoming messages (for ping/pong if needed)
  let mut recv_task = tokio::spawn(async move {
    while let Some(Ok(_msg)) = receiver.next().await {
      // Handle incoming messages if needed (e.g., ping)
    }
  });

  // Wait for either task to finish
  tokio::select! {
    _ = (&mut send_task) => recv_task.abort(),
    _ = (&mut recv_task) => send_task.abort(),
  };
}

async fn list_files(
  axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<FileListResponse> {
  let scene_dir = &state.scene_dir;
  let mut files = Vec::new();

  if let Ok(entries) = fs::read_dir(scene_dir) {
    for entry in entries.flatten() {
      if let Ok(metadata) = entry.metadata() {
        if metadata.is_file() {
          if let Some(file_name) = entry.file_name().to_str() {
            if file_name.ends_with(".obj") {
              files.push(FileInfo {
                name: file_name.to_string(),
              });
            }
          }
        }
      }
    }
  }

  // Sort files by name for consistent ordering
  files.sort_by(|a, b| a.name.cmp(&b.name));

  Json(FileListResponse { files })
}

async fn serve_html() -> Html<&'static str> {
  Html(viewer_html::HTML)
}

fn print_keyboard_help() {
  println!("Kitbash Viewer - Keyboard Controls\n");
  println!("Navigation:");
  println!("  Mouse drag       Rotate camera");
  println!("  Mouse wheel      Zoom in/out");
  println!("  0                Reset camera to initial position");
  println!("  1-6              Standard view angles (front/back/right/left/top/bottom)");
  println!();
  println!("Selection:");
  println!("  Click object     Select object");
  println!("  Click empty      Deselect");
  println!("  [                Select previous object");
  println!("  ]                Select next object");
  println!();
  println!("View:");
  println!("  f                Frame selected object");
  println!("  F (Shift+f)      Frame all visible objects");
  println!("  Tab              Toggle file list overlay");
  println!("  g                Toggle grid visibility");
  println!("  w                Cycle wireframe mode (solid/solid+wire/wire)");
  println!();
  println!("Object Management:");
  println!("  h                Hide/show selected object");
  println!("  H (Shift+h)      Show all hidden objects");
  println!("  r                Reload all files");
  println!();
}

fn print_settings_help() {
  println!("Kitbash Viewer - Available Settings\n");
  println!("Basic Options:");
  println!("  -p, --port <PORT>         Server port (default: 8080)");
  println!("      --host <HOST>         Bind address (default: 127.0.0.1)");
  println!("  -s, --scene-dir <PATH>    Directory to watch for OBJ files (default: scene)");
  println!("  -o, --open                Auto-open browser on startup");
  println!();
  println!("Help:");
  println!("  -h, --help                Show this help message");
  println!("  -V, --version             Show version");
  println!("      --help-keys           Show keyboard controls");
  println!("      --help-settings       Show this settings help");
  println!();
}

#[tokio::main]
async fn main() {
  // Parse CLI arguments
  let cli = Cli::parse();

  // Handle help flags
  if cli.help_keys {
    print_keyboard_help();
    return;
  }

  if cli.help_settings {
    print_settings_help();
    return;
  }

  // Create broadcast channel for file change events
  let (tx, _rx) = broadcast::channel::<FileEvent>(100);
  let tx_clone = tx.clone();

  // Clone scene_dir before moving into async block
  let scene_dir_for_watcher = cli.scene_dir.clone();

  // Set up file watcher in a separate task
  tokio::spawn(async move {
    let (watch_tx, mut watch_rx) = tokio::sync::mpsc::channel(100);

    let mut watcher = notify::recommended_watcher(
      move |res: Result<Event, notify::Error>| {
      if let Ok(event) = res {
        let _ = watch_tx.blocking_send(event);
      }
    })
    .expect("Failed to create file watcher");

    watcher
      .watch(&scene_dir_for_watcher, RecursiveMode::NonRecursive)
      .expect("Failed to watch scene directory");

    println!("File watcher started for {:?}", scene_dir_for_watcher);

    // Debounce map: filename -> (last_event_kind, last_time)
    let mut last_events = HashMap::new();
    let debounce_duration = Duration::from_millis(100);

    while let Some(event) = watch_rx.recv().await {
      for path in event.paths {
        //if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        if let Some(file_name) = path.file_name()
                                     .and_then(|n| n.to_str()) {
          if file_name.ends_with(".obj") {
            // Check if file actually exists
            let file_exists = path.exists();

            let event_kind_str = match event.kind {
              EventKind::Create(_) => "create",
              EventKind::Modify(_) => "modify",
              EventKind::Remove(_) => "remove",
              _ => continue,
            };

            // Verify file state matches event type
            // If we get a create/modify event but file doesn't exist, 
            //   treat as remove
            // If we get a remove event but file exists, ignore it
            let actual_event_kind = if !file_exists {
              "remove"
            } else {
              event_kind_str
            };

            let now = Instant::now();
            let should_send = if 
              let Some((lk, lt)) = last_events.get(file_name) {
              // Only send if different type or enough time passed
              lk != actual_event_kind || 
              now.duration_since(*lt) > debounce_duration
            } else {
              true
            };

            if should_send {
              let change_event = if actual_event_kind == "remove" {
                println!("File removed: {}", file_name);
                // Keep remove in debounce map to prevent duplicates
                last_events.insert(
                  file_name.to_string(), 
                  (actual_event_kind.to_string(), now));
                Some(FileEvent::Removed { filename: file_name.to_string() })
              } else if actual_event_kind == "create" && file_exists {
                println!("File created: {}", file_name);
                last_events.insert(
                  file_name.to_string(), 
                  (actual_event_kind.to_string(), now));
                Some(FileEvent::Added { filename: file_name.to_string() })
              } else if actual_event_kind == "modify" && file_exists {
                println!("File modified: {}", file_name);
                last_events.insert(
                  file_name.to_string(), 
                  (actual_event_kind.to_string(), now));
                Some(FileEvent::Modified { filename: file_name.to_string() })
              } else {
                None
              };

              if let Some(evt) = change_event {
                let _ = tx_clone.send(evt);
              }
            }
          }
        }
      }
    }

    // Keep watcher alive
    drop(watcher);
  });

  let state = AppState {
    scene_dir: cli.scene_dir.clone(),
    tx,
  };

  let app = Router::new()
    .route("/", get(serve_html))
    .route("/api/files", get(list_files))
    .route("/ws", get(websocket_handler))
    .nest_service("/scene", ServeDir::new(&cli.scene_dir))
    .with_state(state);

  let addr = format!("{}:{}", cli.host, cli.port);
  let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

  println!("Kitbash Viewer running at http://{}", addr);
  println!("Scene directory: {:?}", cli.scene_dir);
  println!("WebSocket enabled for live file updates");

  if cli.open {
    println!("Opening browser...");
    let url = format!("http://{}", addr);
    let _ = open::that(&url);
  } else {
    println!("Open your browser to http://{}", addr);
  }

  axum::serve(listener, app).await.unwrap();
}
