use anyhow::{Context, Result};
use axum::{
    body::{boxed, Body},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query,
    },
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use base64::Engine as _;
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::{oneshot, Mutex};
use tower::{service_fn, ServiceExt};
use tower_http::services::ServeDir;

#[derive(Serialize)]
struct FileDiff {
    path: String,
    status: String,
    diff: Option<String>,
}

#[derive(Serialize)]
struct ChangeResponse {
    files: Vec<FileDiff>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn get_changed(
    Path(container): Path<String>,
) -> Result<Json<ChangeResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Get git status to find changed files
    let status_output = Command::new("docker")
        .args(["exec", &container, "git", "status", "--porcelain"])
        .output()
        .await;

    match status_output {
        Ok(out) if out.status.success() => {
            let status_lines = String::from_utf8_lossy(&out.stdout);
            let mut files = Vec::new();

            for line in status_lines.lines() {
                if line.len() < 3 {
                    continue;
                }

                let status_chars: Vec<char> = line.chars().collect();
                let index_status = status_chars[0];
                let worktree_status = status_chars[1];
                let path = line[3..].to_string();

                // Determine the overall status
                let status = if index_status != ' ' && index_status != '?' {
                    index_status.to_string()
                } else {
                    worktree_status.to_string()
                };

                // Get the diff for this file
                let diff_text = match (index_status, worktree_status) {
                    ('?', '?') => {
                        // Untracked file - show entire content as added
                        let cat_output = Command::new("docker")
                            .args(["exec", &container, "cat", &path])
                            .output()
                            .await;
                        match cat_output {
                            Ok(cat_out) if cat_out.status.success() => {
                                let content = String::from_utf8_lossy(&cat_out.stdout);
                                Some(format!(
                                    "--- /dev/null\n+++ {}\n@@ -0,0 +1,{} @@\n{}",
                                    path,
                                    content.lines().count(),
                                    content
                                        .lines()
                                        .map(|l| format!("+{}", l))
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                ))
                            }
                            _ => None,
                        }
                    }
                    _ => {
                        // Use git diff for tracked files
                        let diff_output = Command::new("docker")
                            .args(["exec", &container, "git", "diff", "HEAD", "--", &path])
                            .output()
                            .await;
                        match diff_output {
                            Ok(diff_out) if diff_out.status.success() => {
                                let diff_content =
                                    String::from_utf8_lossy(&diff_out.stdout).to_string();
                                if diff_content.is_empty() {
                                    // Try diff against index for staged changes
                                    let staged_diff = Command::new("docker")
                                        .args([
                                            "exec", &container, "git", "diff", "--cached", "--",
                                            &path,
                                        ])
                                        .output()
                                        .await;
                                    match staged_diff {
                                        Ok(staged_out) if staged_out.status.success() => {
                                            let staged_content =
                                                String::from_utf8_lossy(&staged_out.stdout)
                                                    .to_string();
                                            if !staged_content.is_empty() {
                                                Some(staged_content)
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    }
                                } else {
                                    Some(diff_content)
                                }
                            }
                            _ => None,
                        }
                    }
                };

                files.push(FileDiff {
                    path,
                    status,
                    diff: diff_text,
                });
            }

            Ok(Json(ChangeResponse { files }))
        }
        Ok(out) => {
            let msg = String::from_utf8_lossy(&out.stderr).to_string();
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse { error: msg }),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Deserialize)]
pub(crate) struct TerminalParams {
    token: Option<String>,
    run: Option<String>,
    run_b64: Option<String>,
}

pub(crate) async fn terminal_ws(
    ws: WebSocketUpgrade,
    Path(container): Path<String>,
    Query(params): Query<TerminalParams>,
) -> Response {
    let token_matches = params
        .token
        .as_ref()
        .map(|t| t == &container)
        .unwrap_or(true);

    if token_matches {
        ws.on_upgrade(move |socket| handle_terminal(socket, container, params.run, params.run_b64))
    } else {
        (StatusCode::UNAUTHORIZED, "invalid token").into_response()
    }
}

async fn handle_terminal(
    mut socket: WebSocket,
    container: String,
    run: Option<String>,
    run_b64: Option<String>,
) {
    let mut child = match Command::new("docker")
        // Do not request a TTY from Docker here; allocate a PTY inside the
        // container using `script` so it works from non-TTY servers.
        .args([
            "exec",
            "-i",
            &container,
            "/usr/bin/env",
            "TERM=xterm-256color",
            "/usr/bin/script",
            "-q",
            "-f",
            "-c",
            "bash -l",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            let _ = socket
                .send(Message::Text(format!("failed to start shell: {e}")))
                .await;
            return;
        }
    };

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    // If an autorun command was provided, send it immediately
    if let Some(cmd_b64) = run_b64 {
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(cmd_b64.as_bytes()) {
            let mut to_send = bytes;
            to_send.push(b'\n');
            let _ = stdin.write_all(&to_send).await;
            let _ = stdin.flush().await;
        }
    } else if let Some(cmd) = run {
        let _ = stdin.write_all(format!("{}\n", cmd).as_bytes()).await;
        let _ = stdin.flush().await;
    }

    // Forward shell stdout to websocket as text frames
    let mut out_buf = [0u8; 4096];
    let mut err_buf = [0u8; 4096];
    let sender_stdout = Arc::clone(&sender);
    let stdout_task = tokio::spawn(async move {
        loop {
            match stdout.read(&mut out_buf).await {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&out_buf[..n]).to_string();
                    if sender_stdout.lock().await.send(Message::Text(chunk)).await.is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    });

    // Forward shell stderr to websocket as text frames
    let sender_stderr = Arc::clone(&sender);
    let stderr_task = tokio::spawn(async move {
        loop {
            match stderr.read(&mut err_buf).await {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&err_buf[..n]).to_string();
                    if sender_stderr.lock().await.send(Message::Text(chunk)).await.is_err() {
                        break;
                    }
                }
                _ => break,
            }
        }
    });

    // Forward websocket messages to stdin
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(t) => {
                if stdin.write_all(t.as_bytes()).await.is_err() {
                    break;
                }
                let _ = stdin.flush().await;
            }
            Message::Binary(b) => {
                if stdin.write_all(&b).await.is_err() {
                    break;
                }
                let _ = stdin.flush().await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    let _ = stdin.shutdown().await;
    let _ = stdout_task.await;
    let _ = stderr_task.await;
    let _ = child.kill().await;
}

async fn shutdown_handler(
    Extension(tx): Extension<Arc<Mutex<Option<oneshot::Sender<()>>>>>,
) -> StatusCode {
    if let Some(tx) = tx.lock().await.take() {
        let _ = tx.send(());
    }
    StatusCode::OK
}

pub async fn serve() -> Result<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));
    let serve_dir = ServeDir::new("web/dist");
    let static_files = service_fn(move |req: Request<Body>| {
        let serve_dir = serve_dir.clone();
        async move {
            match serve_dir.oneshot(req).await {
                Ok(res) => Ok(res.map(boxed)),
                Err(err) => Ok::<_, std::convert::Infallible>(
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {err}"),
                    )
                        .into_response(),
                ),
            }
        }
    });
    let app = Router::new()
        .route("/api/changed/:container", get(get_changed))
        .route("/terminal/:container", get(terminal_ws))
        .route("/shutdown", get(shutdown_handler))
        .nest_service("/", static_files)
        .layer(Extension(shutdown_tx));
    let addr = SocketAddr::from(([0, 0, 0, 0], 6789));
    println!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await?;
    Ok(())
}

pub async fn stop() -> Result<()> {
    reqwest::get("http://127.0.0.1:6789/shutdown")
        .await
        .context("failed to send shutdown signal")?;
    Ok(())
}
