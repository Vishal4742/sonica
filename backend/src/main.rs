use actix_web::{web, App, HttpServer, Result, middleware::Logger, HttpResponse};
use actix_cors::Cors;
use actix_multipart::Multipart;
use std::env;
use std::io::Write;
use futures_util::TryStreamExt;

mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize logger
    env_logger::init();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", host, port);

    log::info!("Starting SONICA Audio Recognition Server on {}", bind_address);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .route("/health", web::get().to(health_check))
            .route("/", web::get().to(index))
            .route("/api/audio/upload", web::post().to(upload_audio))
            .route("/api/audio/recognize", web::post().to(recognize_audio))
            .route("/api/audio/status/{id}", web::get().to(get_recognition_status))
    })
    .bind(&bind_address)?
    .run()
    .await
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "sonica-audio-recognition",
        "version": "0.1.0"
    })))
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Welcome to SONICA Audio Recognition API",
        "version": "0.1.0",
        "endpoints": {
            "health": "/health",
            "upload_audio": "/api/audio/upload",
            "recognize_audio": "/api/audio/recognize",
            "recognition_status": "/api/audio/status/{id}"
        }
    })))
}

async fn upload_audio(mut payload: Multipart) -> Result<HttpResponse> {
    let mut audio_data = Vec::new();
    
    while let Some(mut field) = payload.try_next().await? {
        if field.content_disposition().get_filename().is_some() {
            while let Some(chunk) = field.try_next().await? {
                audio_data.write_all(&chunk)?;
            }
        }
    }

    if audio_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No audio file provided"
        })));
    }

    // Generate a unique ID for this audio file
    let audio_id = uuid::Uuid::new_v4().to_string();
    
    // TODO: Save audio file and process it
    log::info!("Received audio file with ID: {}, size: {} bytes", audio_id, audio_data.len());

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Audio file uploaded successfully",
        "audio_id": audio_id,
        "file_size": audio_data.len()
    })))
}

async fn recognize_audio(mut payload: Multipart) -> Result<HttpResponse> {
    let mut audio_data = Vec::new();
    
    while let Some(mut field) = payload.try_next().await? {
        if field.content_disposition().get_filename().is_some() {
            while let Some(chunk) = field.try_next().await? {
                audio_data.write_all(&chunk)?;
            }
        }
    }

    if audio_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "No audio file provided"
        })));
    }

    // Generate a unique ID for this recognition task
    let recognition_id = uuid::Uuid::new_v4().to_string();
    
    // TODO: Implement actual audio recognition logic
    log::info!("Starting audio recognition for ID: {}, size: {} bytes", recognition_id, audio_data.len());

    // Simulate recognition result
    let recognition_result = serde_json::json!({
        "recognition_id": recognition_id,
        "status": "completed",
        "results": [
            {
                "text": "Hello, this is a sample recognition result",
                "confidence": 0.95,
                "start_time": 0.0,
                "end_time": 3.5
            }
        ],
        "language": "en-US",
        "processing_time_ms": 1500
    });

    Ok(HttpResponse::Ok().json(recognition_result))
}

async fn get_recognition_status(path: web::Path<String>) -> Result<HttpResponse> {
    let recognition_id = path.into_inner();
    
    // TODO: Query database for actual recognition status
    log::info!("Checking status for recognition ID: {}", recognition_id);

    // Simulate status response
    let status_response = serde_json::json!({
        "recognition_id": recognition_id,
        "status": "completed",
        "progress": 100,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "completed_at": chrono::Utc::now().to_rfc3339()
    });

    Ok(HttpResponse::Ok().json(status_response))
}
