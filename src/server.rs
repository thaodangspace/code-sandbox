use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;
use std::io::Write;
use std::net::SocketAddr;
use std::process::Command;
use tempfile::NamedTempFile;

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
    let output = Command::new("docker").args(["diff", &container]).output();
    match output {
        Ok(out) if out.status.success() => {
            let diff_output = String::from_utf8_lossy(&out.stdout);

            // Determine base image of the container
            let image = Command::new("docker")
                .args(["inspect", "--format", "{{.Image}}", &container])
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                });

            let mut files = Vec::new();
            for line in diff_output.lines() {
                if line.len() < 2 {
                    continue;
                }
                let status = line.chars().next().unwrap();
                let path = line[2..].to_string();

                let diff_text = match status {
                    'A' | 'C' => {
                        let new_out = Command::new("docker")
                            .args(["exec", &container, "cat", &path])
                            .output();
                        match new_out {
                            Ok(no) if no.status.success() => {
                                let new_bytes = no.stdout;
                                let mut new_file = NamedTempFile::new().map_err(|e| {
                                    (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Json(ErrorResponse {
                                            error: e.to_string(),
                                        }),
                                    )
                                })?;
                                new_file.write_all(&new_bytes).map_err(|e| {
                                    (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        Json(ErrorResponse {
                                            error: e.to_string(),
                                        }),
                                    )
                                })?;

                                let base_bytes = image.as_ref().and_then(|img| {
                                    let base_out = Command::new("docker")
                                        .args(["run", "--rm", img, "cat", &path])
                                        .output()
                                        .ok()?;
                                    if base_out.status.success() {
                                        Some(base_out.stdout)
                                    } else {
                                        None
                                    }
                                });

                                let diff_out = if let Some(base) = base_bytes {
                                    let mut base_file = NamedTempFile::new().map_err(|e| {
                                        (
                                            StatusCode::INTERNAL_SERVER_ERROR,
                                            Json(ErrorResponse {
                                                error: e.to_string(),
                                            }),
                                        )
                                    })?;
                                    base_file.write_all(&base).map_err(|e| {
                                        (
                                            StatusCode::INTERNAL_SERVER_ERROR,
                                            Json(ErrorResponse {
                                                error: e.to_string(),
                                            }),
                                        )
                                    })?;
                                    Command::new("diff")
                                        .args([
                                            "-u",
                                            base_file.path().to_str().unwrap(),
                                            new_file.path().to_str().unwrap(),
                                        ])
                                        .output()
                                        .ok()
                                } else {
                                    Command::new("diff")
                                        .args([
                                            "-u",
                                            "/dev/null",
                                            new_file.path().to_str().unwrap(),
                                        ])
                                        .output()
                                        .ok()
                                };

                                diff_out.map(|d| String::from_utf8_lossy(&d.stdout).to_string())
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                };

                files.push(FileDiff {
                    path,
                    status: status.to_string(),
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

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/changed/:container", get(get_changed));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
