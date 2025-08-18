use axum::{extract::Path, http::StatusCode, routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::process::Command;

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
        .output();

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
                            .output();
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
                            .output();
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
                                        .output();
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

#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/changed/:container", get(get_changed));
    let addr = SocketAddr::from(([0, 0, 0, 0], 6789));
    println!("Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
