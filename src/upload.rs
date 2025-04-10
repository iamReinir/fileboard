use std::path::Path;

use axum::extract::Multipart;
use tokio::fs::{self,File};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use futures::{StreamExt, TryStreamExt};

use crate::config;

pub async fn upload(mut multipart: Multipart, path:&str, wwwroot:&str) -> Result<String, String> {
    let mut result_string = String::new();
    let config = config::get().unwrap_or_default().server;
    let mut upload_count = 0;

    while let Some(field_result) = multipart.next_field().await.transpose() {
        let field = match field_result {
            Err(msg) => return Err(msg.to_string()),
            Ok(field) => field
        };
        let file_name = field.file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let file_path = format!("{}/{}", path, file_name);
        // Check if file already exists
        if fs::try_exists(&file_path).await.map_err(|e| e.to_string())? {
            return Err(format!("File '{}' already exists", file_name));
        }
        // Save file to disk
        let mut stream = field.into_stream();
        let file_path = Path::new(wwwroot).join(path).join(file_name.clone());
        let mut file = File::create(file_path).await
            .map_err(|e| format!("Failed to create file: {}", e))?;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| e.to_string())?;
            file.write_all(&chunk).await
                .map_err(|e| format!("Failed to create file: {}", e))?;
        }
        
        upload_count += 1;
        if upload_count == 1 {
            result_string = format!("{{\"image_url\":\"{}/{}\"}}",
                config.host,
                file_name
            );
        }
    }

    if upload_count == 0 {
        Err("No file upload".to_string())
    } else if upload_count == 1 {
        Ok(result_string)
    } else {
        Ok("Files uploaded sussecfully".to_string())
    }
}

