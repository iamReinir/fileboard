use std::net::SocketAddr;
use filetree::serve_files;
use std::env;

use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use urlencoding::decode;

use hyper::{Method, StatusCode};
use http_body_util::combinators::BoxBody;
use config::empty;

pub mod config;
pub mod filetree;


// File server

async fn serve_file(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    eprintln!("[{}] Request: {} {}", chrono::Local::now().to_rfc3339(), req.method(), req.uri());
    match *req.method() {
        Method::GET => {
            let config = config::get().unwrap().server;
            let path = req.uri().path().trim_start_matches('/');
            let decoded_path = decode(path).unwrap().into_owned();
            serve_files(decoded_path.as_str(), &config.wwwroot).await
        },
        Method::POST => {
            Ok(Response::new(empty()))
        },
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
            Ok(not_found)
        }
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