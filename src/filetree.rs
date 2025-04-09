use std::path::Path;
use hyper::body::Bytes;
use hyper::Response;
use hyper::header;
use hyper::StatusCode;
use http_body_util::combinators::BoxBody;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use urlencoding::encode;


use crate::config::{full, empty};

pub async fn serve_files(path: &str, wwwroot: &str) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
{
    let full_path = Path::new(wwwroot).join(path);
    let metadata = match fs::metadata(&full_path).await {
        Ok(meta) => meta,
        Err(_) => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            return Ok(not_found)
        }
    };

    if metadata.is_dir() {
        match directory_page(path, wwwroot).await {
            Ok(html) => return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html")
                .body(full(html))
                .unwrap()),
            Err(_) => return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty())
                .unwrap())
        }
    }
    
    let mime = guess_mime(path);
    match File::open(full_path).await {
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
            *not_found.status_mut() = StatusCode::IM_A_TEAPOT;
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

async fn directory_page(relative_path: &str, wwwroot: &str) -> Result<String, String> {
    let wwwroot_path = Path::new(wwwroot);
    let path = Path::new(wwwroot).join(relative_path);
    
    match fs::read_dir(&path).await {
        Ok(mut entries) => {
            let mut html = String::from("<html><body><h1>Directory listing</h1><ul>");
            eprint!("PATH : {}", relative_path);
            if !relative_path.is_empty() {
                let parent = Path::new(relative_path).parent().unwrap()
                    .parent();
                html.push_str(format!("<li><a href=\"{}\">{}</a></li>",
                    parent.unwrap_or(Path::new("/")).to_string_lossy(),
                    "../"
                ).as_str());
            }
            while let Ok(Some(entry)) = entries.next_entry().await {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let encoded_file_name = encode(&file_name);

                let file_path = path.join(&file_name);
                let relative_path = file_path.strip_prefix(wwwroot_path)
                    .unwrap_or(&file_path);

                // Get the parent part (if any), then append the encoded filename
                let mut href = String::from("");
                if let Some(parent) = relative_path.parent() {
                    href.push_str(&parent.to_string_lossy());
                    href.push('/');
                }
                href.push_str(&encoded_file_name);
                html.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>",
                    href, file_name
                ));
            }

            html.push_str("</ul></body></html>");
            Ok(html)
        }
        Err(_) => Err("Failed to read directory".to_string()),
    }
}