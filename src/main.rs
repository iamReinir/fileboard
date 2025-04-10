use std::net::SocketAddr;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::extract::Multipart;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
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


#[tokio::main]
async fn main() {

    let args: Vec<String> = env::args().collect();

    let config_file_path = args.get(1)
        .map(|x| x.as_str())
        .unwrap_or("./config.toml");
    let config = config::Config::new(config_file_path);
    eprintln!("Config:\nport: {}\nroot: {}", config.server.port, config.server.wwwroot);
    config::set(config);

    let configx = config::get().unwrap();
    let address = if !configx.server.allow_public {
        [127,0,0,1] 
    } else { 
        [0,0,0,0]
    };

    eprintln!("Starting fileboard at {}:{}",
        if configx.server.allow_public { "0.0.0.0" } else { "localhost" },
        configx.server.port
    );
    eprintln!("Content root: {}", configx.server.wwwroot);
    let post_routes = Router::new()
        .route("/", post(sendfile_root))
        .route("/{*path}", post(sendfile))
        // Disable the default limit
        .layer(DefaultBodyLimit::disable())
        // Set a different limit
        .layer(RequestBodyLimitLayer::new(20 * 1024 * 1024)); // 20 MB

    let get_routes = Router::new()
        .route("/", get(get_root))
        .route("/{*path}", get(getfile));

    let router = post_routes
        .merge(get_routes)
        .layer(TraceLayer::new_for_http());
    let addr = SocketAddr::from((address, configx.server.port));
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

