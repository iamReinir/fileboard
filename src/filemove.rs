use axum::{extract::Path, response::IntoResponse, Json};
use hyper::StatusCode;
use serde::Deserialize;
use std::{fs, path::Path as FsPath};

use crate::config;
#[derive(Deserialize)]
pub struct MoveRequest {
    destination: String,
}



pub async fn mv(
    Path(path): Path<String>,
    Json(payload): Json<MoveRequest>) -> impl IntoResponse {
    eprintln!("[{}] PATCH : {} {}", chrono::Local::now().to_rfc3339(), path, payload.destination);
    let wwwroot = config::get().unwrap().server.wwwroot;
    let clean_path = path.strip_prefix("/").unwrap_or(&path);
    let src_path = FsPath::new(&wwwroot).join(clean_path);
    let dest_path = FsPath::new(&wwwroot).join(&payload.destination);
    eprintln!("str : {}", src_path.to_string_lossy());
    eprintln!("des : {}", dest_path.to_string_lossy());
    if !src_path.exists() {
        return (StatusCode::NOT_FOUND, "Source not found").into_response();
    }
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            if let Err(err) = fs::create_dir_all(parent) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create parent dirs: {}", err),
                )
                    .into_response();
            }
        }
    }

    if dest_path.exists() {
        return (
            StatusCode::CONFLICT,
            "Destination already exists. Trash that first.",
        )
        .into_response();
    }

    match fs::rename(src_path, dest_path) {
        Ok(_) => (StatusCode::OK, "Move successful").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Move failed: {}", e),
        )
            .into_response(),
    }
}

pub async fn del_root() -> impl IntoResponse 
{
    del(Path("".to_string())).await
}

pub async fn del(
    Path(path): Path<String>) -> impl IntoResponse {
    eprintln!("[{}] DELETE : {}", chrono::Local::now().to_rfc3339(), path);
    let server = config::get().unwrap().server;
    let wwwroot = server.wwwroot;
    let clean_path = path.strip_prefix("/").unwrap_or(&path);
    let filename = FsPath::new(&path).file_name().unwrap();
    let src_path = FsPath::new(&wwwroot).join(clean_path);
    let dest_path = FsPath::new(&server.trash_can).join(filename);
    if !src_path.exists() {
        return (StatusCode::NOT_FOUND, "Source not found").into_response();
    }
    if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            if let Err(err) = fs::create_dir_all(parent) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create parent dirs: {}", err),
                )
                    .into_response();
            }
        }
    }

    match fs::rename(src_path, dest_path) {
        Ok(_) => (StatusCode::OK, "Move successful").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Move failed: {}", e),
        )
            .into_response(),
    }
}