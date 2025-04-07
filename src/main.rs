// use std::convert::Infallible;
use std::net::SocketAddr;
use config::Config;
use http_body_util::Empty;
use std::path::Path;
use std::env;
use std::sync::Mutex;
use once_cell::sync::Lazy;


use http_body_util::Full;
use hyper::body::Bytes;
// use hyper::body::Body;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use hyper::header;
// use lazy_static::lazy_static;

// use hyper::body::Frame;
use hyper::{Method, StatusCode};
use http_body_util::{combinators::BoxBody, BodyExt};


use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
// use serde::Deserialize;
// use std::fs::File as StdFile;

pub mod config;




// Echo function that worked
async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full(
            "Try POSTing data to /echo",
        ))),
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body().boxed())),

        // Return 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

// File server

async fn serve_file(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // Get the requested file path, default to "index.html"
    let config = config::get().unwrap().server;
    let path = req.uri().path().trim_start_matches('/');
    let file_path = if path.is_empty() {
        "index.html"
    } else {
        path
    };
    let full_path = Path::new(&config.wwwroot).join(file_path);
    let mime = guess_mime(path);
    let path = PathBuf::from(full_path);
    match File::open(path).await {
        Ok(mut file) => {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await.unwrap();
            let bytes = Bytes::from(contents);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .body(full(bytes))
                .unwrap())
        }
        Err(_) => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn guess_mime(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",  // JSON MIME type (structured text)
        Some("toml") => "text/plain",         // TOML MIME type (text format)
        Some("md") => "text/markdown",       // Markdown MIME type (text format)
        Some("txt") => "text/plain",         // Plain text
        Some("png") => "image/png",          // Image MIME type
        Some("jpg") | Some("jpeg") => "image/jpeg", // Image MIME type
        Some("gif") => "image/gif",          // Image MIME type
        Some("svg") => "image/svg+xml",      // Image MIME type
        Some("bmp") => "image/bmp",          // Image MIME type
        Some("tiff") | Some("tif") => "image/tiff", // Image MIME type
        Some("mp3") => "audio/mpeg",         // Audio MIME type
        Some("wav") => "audio/wav",          // Audio MIME type
        Some("ogg") => "audio/ogg",          // Audio MIME type
        Some("flac") => "audio/flac",        // Audio MIME type
        Some("mp4") => "video/mp4",          // Video MIME type
        Some("webm") => "video/webm",        // Video MIME type
        Some("avi") => "video/x-msvideo",    // Video MIME type
        Some("mov") => "video/quicktime",    // Video MIME type
        Some("mkv") => "video/x-matroska",   // Video MIME type
        _ => "application/octet-stream",     // Default (download)
    }
}




#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let args: Vec<String> = env::args().collect();

    let config_file_path = args.get(1)
        .map(|x| x.as_str())
        .unwrap_or("./config.toml");
    let config = config::Config::new(config_file_path);
    eprintln!("Config:\nport: {}\nroot: {}", config.server.port, config.server.wwwroot);
    config::set(config);

    let configx = config::get().unwrap();
    let address = if configx.server.allow_public == false {
        [127,0,0,1] 
    } else { 
        [0,0,0,0]
    };

    eprintln!("Starting fileboard at {}:{}",
        if configx.server.allow_public { "0.0.0.0" } else { "localhost" },
        3000
    );
    eprintln!("Content root: {}", configx.server.wwwroot);

    let addr = SocketAddr::from((address, configx.server.port));
    // We create a TcpListener and bind it
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(serve_file))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}



// End of config things