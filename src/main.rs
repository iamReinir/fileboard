use std::fs;
use std::net::SocketAddr;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::routing::put;
use axum::Router;
use filetree::format_size;
use filetree::serve_files;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use upload::upload;
use std::env;
use hyper::{Request, Response};
use tokio::net::TcpListener;

use hyper::{Method, StatusCode};
use config::{empty, full};

pub mod config;
pub mod filetree;
pub mod upload;
pub mod jstemplate;


// File server
async fn getfile(Path(path): Path<String>)
-> impl IntoResponse {
    eprintln!("[{}] GET : {}", chrono::Local::now().to_rfc3339(), path);
    let config = config::get().unwrap().server;
    match serve_files(path.as_str(), &config.wwwroot).await {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Error serving file: {:?}", err);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full("Internal server error"))
                .unwrap()
        }
    }
}

async fn get_root() -> impl IntoResponse {
    getfile(Path("".to_string())).await
}


async fn sendfile(
    Path(path): Path<String>,
    multipart: Multipart,
)
-> impl IntoResponse {
    let folder = path;
    let config = config::get().unwrap().server;
    eprintln!("[{}] POST: {}", chrono::Local::now().to_rfc3339(), folder);
    match upload(multipart, &folder, &config.wwwroot).await {
        Ok(msg) => (StatusCode::OK, msg),
        Err(msg) => (StatusCode::BAD_REQUEST, msg)
    }
}

async fn sendfile_root(multipart: Multipart)
-> impl IntoResponse {
    sendfile(Path("".to_string()), multipart).await
}

async fn create_dir(Path(path): Path<String>) 
-> impl IntoResponse {
    let root = config::get().unwrap_or_default().server.wwwroot;
    let dir = std::path::Path::new(&root).join(path);
    eprintln!("Create directory:{}", dir.to_string_lossy());
    match fs::create_dir(dir) {
        Err(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
        Ok(_) => (StatusCode::OK, "Directory created".to_string())
    }
}




#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();

    let config_file_path = args.get(1)
        .map(|x| x.as_str())
        .unwrap_or("./config.toml");
    let config = config::Config::new(config_file_path);
    eprintln!("========================================================");
    eprintln!("Config:\nport: {}\nroot: {}\n host:{}\n max upload size:{}\n",
        config.server.port,
        config.server.wwwroot,
        config.server.host,
        format_size(config.server.max_file_size.try_into().unwrap())
    );
    config::set(config);

    let configx = config::get().unwrap();
    let address = if !configx.server.allow_public {
        [127,0,0,1] 
    } else { 
        [0,0,0,0]
    };
    eprintln!("========================================================");
    eprintln!("Starting fileboard at {}:{}",
        if configx.server.allow_public { "0.0.0.0" } else { "localhost" },
        configx.server.port
    );
    eprintln!("Content root: {}", configx.server.wwwroot);
    eprintln!("========================================================");
    let post_routes = Router::new()
        .route("/", post(sendfile_root))
        .route("/{*path}", post(sendfile))
        // Disable the default limit
        .layer(DefaultBodyLimit::disable())
        // Set a different limit
        .layer(RequestBodyLimitLayer::new(configx.server.max_file_size));

    let get_routes = Router::new()
        .route("/", get(get_root))
        .route("/{*path}", get(getfile))
        .route("/{*path}", put(create_dir));

    let router = post_routes
        .merge(get_routes)
        .layer(TraceLayer::new_for_http());
    let addr = SocketAddr::from((address, configx.server.port));
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

