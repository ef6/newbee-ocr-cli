//! OCR HTTP 服务模块
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer};
use futures_util::StreamExt;
use ocr_rs::OcrEngine;
use serde::Serialize;

use crate::models::{
    is_language_supported_by_detection_model, unsupported_language_message, DetectionModel,
    RecognitionModel,
};
use crate::{create_engine, GpuBackend, PrecisionModeArg};

#[derive(Serialize)]
struct OcrJsonResponse {
    results: Vec<OcrTextRegion>,
    time_ms: u64,
}

#[derive(Serialize)]
struct OcrTextRegion {
    text: String,
    confidence: f32,
    bbox: OcrBBox,
}

#[derive(Serialize)]
struct OcrBBox {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

struct AppState {
    engine: Arc<OcrEngine>,
    det_model: String,
    language: String,
}

pub fn run_service(
    host: &str,
    port: u16,
    language: &str,
    det_model_str: &str,
    models_dir: Option<&Path>,
    precision: PrecisionModeArg,
    threads: i32,
    gpu: Option<GpuBackend>,
    verbose: bool,
) -> anyhow::Result<()> {
    let rec_model = RecognitionModel::from_str(language)
        .ok_or_else(|| anyhow::anyhow!("Unknown language/model: {}", language))?;
    let det_model = DetectionModel::from_str(det_model_str)
        .ok_or_else(|| anyhow::anyhow!("Unknown detection model: {}", det_model_str))?;
    if !is_language_supported_by_detection_model(language, det_model) {
        anyhow::bail!(unsupported_language_message(language, det_model));
    }

    if verbose {
        log::info!("Loading OCR engine (language={}, det={})...", language, det_model);
    }
    let start = Instant::now();
    let engine = create_engine(
        rec_model, det_model, models_dir, precision, threads, gpu, verbose,
    )?;
    log::info!("OCR engine loaded in {:?}", start.elapsed());

    let bind_addr = format!("{}:{}", host, port);
    log::info!("Starting OCR service on http://{}", bind_addr);
    log::info!("  POST /ocr       -> JSON with timing");
    log::info!("  POST /ocr-text  -> plain text");

    let state = web::Data::new(AppState {
        engine: Arc::new(engine),
        det_model: det_model_str.to_string(),
        language: language.to_string(),
    });

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .route("/", web::get().to(index_handler))
                .route("/ocr", web::post().to(ocr_handler))
                .route("/ocr-text", web::post().to(ocr_text_handler))
        })
        .bind(&bind_addr)?
        .run()
        .await
    })?;

    Ok(())
}

async fn index_handler(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(format!(
            "Newbee OCR Service is running\nDetection model: {}\nLanguage: {}\n",
            state.det_model, state.language
        ))
}

async fn extract_image(mut payload: Multipart) -> Result<Vec<u8>, HttpResponse> {
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| HttpResponse::BadRequest().body(e.to_string()))?;
        if field.content_disposition().get_name() == Some("image_file") {
            let mut buf = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| HttpResponse::BadRequest().body(e.to_string()))?;
                buf.extend_from_slice(&data);
            }
            return Ok(buf);
        }
    }
    Err(HttpResponse::BadRequest().body("Missing 'image_file' field"))
}

fn do_ocr(engine: &OcrEngine, data: &[u8]) -> Result<Vec<OcrTextRegion>, String> {
    let img = image::load_from_memory(data).map_err(|e| format!("Image decode failed: {}", e))?;
    let results = engine.recognize(&img).map_err(|e| format!("OCR failed: {}", e))?;
    Ok(results
        .iter()
        .map(|r| OcrTextRegion {
            text: r.text.clone(),
            confidence: r.confidence,
            bbox: OcrBBox {
                x: r.bbox.rect.left(),
                y: r.bbox.rect.top(),
                width: r.bbox.rect.width(),
                height: r.bbox.rect.height(),
            },
        })
        .collect())
}

async fn ocr_handler(state: web::Data<AppState>, payload: Multipart) -> HttpResponse {
    let data = match extract_image(payload).await {
        Ok(d) => d,
        Err(r) => return r,
    };
    let start = Instant::now();
    let result = do_ocr(&state.engine, &data);
    let time_ms = start.elapsed().as_millis() as u64;
    log::info!("POST /ocr completed in {}ms", time_ms);

    match result {
        Ok(regions) => HttpResponse::Ok().json(OcrJsonResponse { results: regions, time_ms }),
        Err(e) => {
            log::error!("OCR error: {}", e);
            HttpResponse::InternalServerError().body(e)
        }
    }
}

async fn ocr_text_handler(state: web::Data<AppState>, payload: Multipart) -> HttpResponse {
    let data = match extract_image(payload).await {
        Ok(d) => d,
        Err(r) => return r,
    };
    let start = Instant::now();
    let result = do_ocr(&state.engine, &data);
    let time_ms = start.elapsed().as_millis() as u64;
    log::info!("POST /ocr-text completed in {}ms", time_ms);

    match result {
        Ok(regions) => {
            let text = regions.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join("\n");
            HttpResponse::Ok().content_type("text/plain; charset=utf-8").body(text)
        }
        Err(e) => {
            log::error!("OCR error: {}", e);
            HttpResponse::InternalServerError().body(e)
        }
    }
}