//! 流水线处理模块
//!
//! 实现批量图片的流水线 OCR 处理，提高吞吐量

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ocr_rs::{OcrEngine, OcrResult_};
use walkdir::WalkDir;

/// 图片任务
#[derive(Debug)]
pub struct ImageTask {
    /// 图片路径
    pub path: PathBuf,
    /// 任务索引
    pub index: usize,
}

/// OCR 结果
#[derive(Debug)]
pub struct OcrTaskResult {
    /// 原始任务
    pub task: ImageTask,
    /// OCR 结果
    pub results: Result<Vec<OcrResult_>>,
    /// 处理耗时
    pub duration: Duration,
}

/// 流水线配置
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PipelineConfig {
    /// 图片加载线程数
    pub loader_threads: usize,
    /// 推理引擎数 (并行推理)
    pub engine_count: usize,
    /// 图片缓冲区大小
    pub buffer_size: usize,
    /// 是否显示进度条
    pub show_progress: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        let cpus = num_cpus::get();
        Self {
            loader_threads: (cpus / 4).max(2),
            engine_count: 1, // OCR 引擎通常使用多线程，单引擎即可
            buffer_size: 32,
            show_progress: true,
        }
    }
}

#[allow(dead_code)]
impl PipelineConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置加载线程数
    pub fn with_loader_threads(mut self, threads: usize) -> Self {
        self.loader_threads = threads.max(1);
        self
    }

    /// 设置引擎数量
    pub fn with_engine_count(mut self, count: usize) -> Self {
        self.engine_count = count.max(1);
        self
    }

    /// 设置缓冲区大小
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size.max(1);
        self
    }

    /// 设置是否显示进度
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }
}

/// 收集文件夹中的所有图片
pub fn collect_images(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let supported_extensions = ["jpg", "jpeg", "png", "bmp", "gif", "webp", "tiff", "tif"];

    let mut images = Vec::new();

    if recursive {
        for entry in WalkDir::new(dir).follow_links(true) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if supported_extensions
                        .iter()
                        .any(|&e| ext.eq_ignore_ascii_case(e))
                    {
                        images.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    } else {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if supported_extensions
                        .iter()
                        .any(|&e| ext.eq_ignore_ascii_case(e))
                    {
                        images.push(path);
                    }
                }
            }
        }
    }

    // 按文件名排序
    images.sort();

    Ok(images)
}

/// 图片加载结果
struct LoadedImage {
    task: ImageTask,
    image: Result<image::DynamicImage>,
}

/// 流水线 OCR 处理器
pub struct OcrPipeline {
    engine: Arc<OcrEngine>,
    config: PipelineConfig,
}

#[allow(dead_code)]
impl OcrPipeline {
    /// 创建新的流水线
    pub fn new(engine: OcrEngine, config: PipelineConfig) -> Self {
        Self {
            engine: Arc::new(engine),
            config,
        }
    }

    /// 处理单张图片
    pub fn process_single(&self, path: &Path) -> Result<Vec<OcrResult_>> {
        let image = image::open(path)
            .with_context(|| format!("Failed to open image: {}", path.display()))?;

        self.engine
            .recognize(&image)
            .map_err(|e| anyhow::anyhow!("OCR failed: {}", e))
    }

    /// 处理多张图片 (流水线模式)
    pub fn process_batch(&self, paths: Vec<PathBuf>) -> Vec<OcrTaskResult> {
        let total = paths.len();
        if total == 0 {
            return Vec::new();
        }

        // 创建进度条
        let multi_progress = if self.config.show_progress {
            Some(MultiProgress::new())
        } else {
            None
        };

        let progress_bar = multi_progress.as_ref().map(|mp| {
            let pb = mp.add(ProgressBar::new(total as u64));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                    .unwrap()
                    .progress_chars("█▓▒░ "),
            );
            pb.set_message("Processing...");
            pb
        });

        // 创建通道
        let (task_tx, task_rx): (Sender<ImageTask>, Receiver<ImageTask>) =
            bounded(self.config.buffer_size);
        let (loaded_tx, loaded_rx): (Sender<LoadedImage>, Receiver<LoadedImage>) =
            bounded(self.config.buffer_size);
        let (result_tx, result_rx): (Sender<OcrTaskResult>, Receiver<OcrTaskResult>) =
            bounded(self.config.buffer_size);

        // 启动任务分发线程
        let paths_clone = paths.clone();
        let dispatcher = thread::spawn(move || {
            for (index, path) in paths_clone.into_iter().enumerate() {
                let task = ImageTask { path, index };
                if task_tx.send(task).is_err() {
                    break;
                }
            }
        });

        // 启动图片加载线程池
        let loaders: Vec<_> = (0..self.config.loader_threads)
            .map(|_| {
                let rx = task_rx.clone();
                let tx = loaded_tx.clone();

                thread::spawn(move || {
                    for task in rx {
                        let image = image::open(&task.path)
                            .map_err(|e| anyhow::anyhow!("Failed to load image: {}", e));

                        if tx.send(LoadedImage { task, image }).is_err() {
                            break;
                        }
                    }
                })
            })
            .collect();

        // 关闭发送端，让接收端知道何时结束
        drop(task_rx);
        drop(loaded_tx);

        // 启动推理线程
        let engine = Arc::clone(&self.engine);
        let pb_clone = progress_bar.clone();
        let result_tx_clone = result_tx.clone();

        let processor = thread::spawn(move || {
            for loaded in loaded_rx {
                let start = Instant::now();

                let results = match loaded.image {
                    Ok(img) => engine
                        .recognize(&img)
                        .map_err(|e| anyhow::anyhow!("OCR failed: {}", e)),
                    Err(e) => Err(e),
                };

                let duration = start.elapsed();

                let result = OcrTaskResult {
                    task: loaded.task,
                    results,
                    duration,
                };

                if let Some(ref pb) = pb_clone {
                    pb.inc(1);
                }

                if result_tx_clone.send(result).is_err() {
                    break;
                }
            }
        });

        drop(result_tx);

        // 收集结果
        let mut results: Vec<OcrTaskResult> = result_rx.iter().collect();

        // 等待所有线程完成
        dispatcher.join().ok();
        for loader in loaders {
            loader.join().ok();
        }
        processor.join().ok();

        // 按原始顺序排序
        results.sort_by_key(|r| r.task.index);

        // 完成进度条
        if let Some(pb) = progress_bar {
            pb.finish_with_message("Done!");
        }

        results
    }

    /// 处理文件夹
    pub fn process_directory(&self, dir: &Path, recursive: bool) -> Result<Vec<OcrTaskResult>> {
        let images = collect_images(dir, recursive)?;

        if images.is_empty() {
            anyhow::bail!("No images found in directory: {}", dir.display());
        }

        Ok(self.process_batch(images))
    }
}

/// 统计信息
#[derive(Debug, Default)]
pub struct PipelineStats {
    /// 处理的图片数
    pub total_images: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub error_count: usize,
    /// 识别的文本区域总数
    pub total_text_regions: usize,
    /// 总耗时
    pub total_duration: Duration,
    /// 平均每张图片耗时
    pub avg_duration: Duration,
    /// 最快处理时间
    pub min_duration: Duration,
    /// 最慢处理时间
    pub max_duration: Duration,
}

impl PipelineStats {
    /// 从结果计算统计信息
    pub fn from_results(results: &[OcrTaskResult]) -> Self {
        if results.is_empty() {
            return Self::default();
        }

        let mut stats = Self {
            total_images: results.len(),
            min_duration: Duration::MAX,
            ..Default::default()
        };

        for result in results {
            stats.total_duration += result.duration;

            if result.duration < stats.min_duration {
                stats.min_duration = result.duration;
            }
            if result.duration > stats.max_duration {
                stats.max_duration = result.duration;
            }

            match &result.results {
                Ok(ocr_results) => {
                    stats.success_count += 1;
                    stats.total_text_regions += ocr_results.len();
                }
                Err(_) => {
                    stats.error_count += 1;
                }
            }
        }

        if stats.total_images > 0 {
            stats.avg_duration = stats.total_duration / stats.total_images as u32;
        }

        if stats.min_duration == Duration::MAX {
            stats.min_duration = Duration::ZERO;
        }

        stats
    }

    /// 打印统计信息
    pub fn print(&self) {
        use colored::*;

        println!();
        println!(
            "{}",
            "═══════════════════════════════════════════════════".bright_blue()
        );
        println!(
            "{}",
            "                  Processing Statistics"
                .bright_white()
                .bold()
        );
        println!(
            "{}",
            "═══════════════════════════════════════════════════".bright_blue()
        );
        println!();

        println!("  {} {}", "Total images:".bright_cyan(), self.total_images);
        println!("  {} {}", "Successful:".green(), self.success_count);
        println!("  {} {}", "Failed:".red(), self.error_count);
        println!(
            "  {} {}",
            "Text regions found:".bright_cyan(),
            self.total_text_regions
        );
        println!();

        println!(
            "  {} {:?}",
            "Total time:".bright_cyan(),
            self.total_duration
        );
        println!(
            "  {} {:?}",
            "Average per image:".bright_cyan(),
            self.avg_duration
        );
        println!("  {} {:?}", "Fastest:".green(), self.min_duration);
        println!("  {} {:?}", "Slowest:".yellow(), self.max_duration);

        if self.total_duration.as_secs_f64() > 0.0 {
            let throughput = self.total_images as f64 / self.total_duration.as_secs_f64();
            println!(
                "  {} {:.2} images/sec",
                "Throughput:".bright_cyan(),
                throughput
            );
        }

        println!();
    }
}
