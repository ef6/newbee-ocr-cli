//! HTTP 服务模块
//!
//! 提供基于 axum 的 OCR 服务接口

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use log::info; 

use crate::{
    create_engine, models::{DetectionModel, RecognitionModel, is_language_supported_by_detection_model, unsupported_language_message},
    GpuBackend, PrecisionModeArg,
};

/// 启动 HTTP 服务（同步入口，内部创建 tokio 运行时）
pub fn run_service(
    host: String,
    port: u16,
    models_dir: Option<&Path>,
    language: &str,
    det_model_str: &str,
    gpu: Option<GpuBackend>,
    precision: PrecisionModeArg,
    threads: i32,
) -> Result<()> {
    // 解析模型
    let rec_model = RecognitionModel::from_str(language)
        .ok_or_else(|| anyhow::anyhow!("Unknown language/model: {}", language))?;

    let det_model = DetectionModel::from_str(det_model_str)
        .ok_or_else(|| anyhow::anyhow!("Unknown detection model: {}", det_model_str))?;

    if !is_language_supported_by_detection_model(language, det_model) {
        anyhow::bail!(unsupported_language_message(language, det_model));
    }

    // 同步创建 OCR 引擎（内部可能包含耗时加载）
    let engine = Arc::new(create_engine(
        rec_model,
        det_model,
        models_dir,
        precision,
        threads,
        gpu,
        true, // 服务模式默认 verbose 以便日志
    )?);

    // 在异步运行时中启动服务
    let rt = Runtime::new()?;
    rt.block_on(async {
        let app = Router::new()
            .route("/", get(root_handler))
            .route("/ocr", post(ocr_handler))
            .route("/ocr-text", post(ocr_text_handler))
            .with_state(engine);

        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).await?;
        log::info!("Newbee OCR service listening on http://{}", addr);
        log::info!("  - GET  /           status");
        log::info!("  - POST /ocr        JSON result");
        log::info!("  - POST /ocr-text   plain text lines");

        axum::serve(listener, app).await?;
        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}

// ---- Handlers ----

async fn root_handler() -> &'static str {
    "Newbee OCR Service is running.\nUse POST /ocr or /ocr-text with multipart field 'image_file'."
}

async fn ocr_handler(
    State(engine): State<Arc<ocr_rs::OcrEngine>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut image_data = None;
    
    while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        if field.name() == Some("image_file") {
            let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read image: {}", e)))?;
            image_data = Some(data);
            break;
        }
    }
    
    let data = image_data
    .ok_or_else(|| AppError::BadRequest("Missing field 'image_file'".to_string()))?;
    
    let img = image::load_from_memory(&data)
    .map_err(|e| AppError::BadRequest(format!("Invalid image: {}", e)))?;
    
    
    let start = Instant::now();
    let results = engine
    .recognize(&img)
    .map_err(|e| AppError::Internal(format!("OCR failed: {}", e)))?;
    let duration = start.elapsed();
    
    info!("/ocr completed in {:?}", duration);  // 记录日志

    let output = json!({
        "results": results.iter().map(|r| {
            json!({
                "text": r.text,
                "confidence": r.confidence,
                "bbox": {
                    "x": r.bbox.rect.left(),
                    "y": r.bbox.rect.top(),
                    "width": r.bbox.rect.width(),
                    "height": r.bbox.rect.height(),
                }
            })
        }).collect::<Vec<_>>(),
        "time_ms": duration.as_millis() as u64,
    });

    Ok(Json(output))
}

async fn ocr_text_handler(
    State(engine): State<Arc<ocr_rs::OcrEngine>>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    let mut image_data = None;
    
    while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        if field.name() == Some("image_file") {
            let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read image: {}", e)))?;
            image_data = Some(data);
            break;
        }
    }
    
    let data = image_data
    .ok_or_else(|| AppError::BadRequest("Missing field 'image_file'".to_string()))?;
    
    let img = image::load_from_memory(&data)
    .map_err(|e| AppError::BadRequest(format!("Invalid image: {}", e)))?;
    
    let start = Instant::now();
    let results = engine
        .recognize(&img)
        .map_err(|e| AppError::Internal(format!("OCR failed: {}", e)))?;
    let duration = start.elapsed();

    info!("/ocr-text completed in {:?}", duration);   // 记录日志

    let text_lines: Vec<String> = results.iter().map(|r| r.text.clone()).collect();
    let body = text_lines.join("\n");

    // 返回纯文本，并附加上耗时（在响应头或正文？根据需求"逐行返回解析出的纯文本内容"，不要求耗时，但可加注释）
    // 按描述只返回纯文本，不加额外信息。
    Ok((StatusCode::OK, body).into_response())
}

// ---- 错误类型 ----

enum AppError {
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, msg).into_response()
    }
}