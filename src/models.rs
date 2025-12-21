//! 模型模块
//!
//! 定义内嵌模型和语言支持

use std::fmt;
use std::path::Path;

/// 支持的语言/模型类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecognitionModel {
    /// 中文 (PP-OCRv5, 简体中文、繁体中文、英文)
    Chinese,
    /// 韩语 (Korean, English)
    Korean,
    /// 拉丁语系 (French, German, Italian, Spanish, Portuguese, etc.)
    Latin,
    /// 东斯拉夫语 (Russian, Belarusian, Ukrainian, English)
    EastSlavic,
    /// 泰语 (Thai, English)
    Thai,
    /// 希腊语 (Greek, English)
    Greek,
    /// 英语 (English only)
    English,
    /// 西里尔字母 (Russian, Bulgarian, Mongolian, etc.)
    Cyrillic,
    /// 阿拉伯语系 (Arabic, Persian, Uyghur, Urdu, etc.)
    Arabic,
    /// 天城文 (Hindi, Marathi, Nepali, Sanskrit, etc.)
    Devanagari,
    /// 泰米尔语 (Tamil, English)
    Tamil,
    /// 泰卢固语 (Telugu, English)
    Telugu,
}

impl RecognitionModel {
    /// 获取模型名称
    pub fn name(&self) -> &'static str {
        match self {
            RecognitionModel::Chinese => "chinese",
            RecognitionModel::Korean => "korean",
            RecognitionModel::Latin => "latin",
            RecognitionModel::EastSlavic => "eslav",
            RecognitionModel::Thai => "thai",
            RecognitionModel::Greek => "greek",
            RecognitionModel::English => "english",
            RecognitionModel::Cyrillic => "cyrillic",
            RecognitionModel::Arabic => "arabic",
            RecognitionModel::Devanagari => "devanagari",
            RecognitionModel::Tamil => "tamil",
            RecognitionModel::Telugu => "telugu",
        }
    }

    /// 获取模型文件名
    pub fn model_filename(&self) -> &'static str {
        match self {
            RecognitionModel::Chinese => "PP-OCRv5_mobile_rec.mnn",
            RecognitionModel::Korean => "korean_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::Latin => "latin_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::EastSlavic => "eslav_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::Thai => "th_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::Greek => "el_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::English => "en_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::Cyrillic => "cyrillic_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::Arabic => "arabic_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::Devanagari => "devanagari_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::Tamil => "ta_PP-OCRv5_mobile_rec_infer.mnn",
            RecognitionModel::Telugu => "te_PP-OCRv5_mobile_rec_infer.mnn",
        }
    }

    /// 获取字符集文件名
    pub fn charset_filename(&self) -> &'static str {
        match self {
            RecognitionModel::Chinese => "ppocr_keys_v5.txt",
            RecognitionModel::Korean => "ppocr_keys_korean.txt",
            RecognitionModel::Latin => "ppocr_keys_latin.txt",
            RecognitionModel::EastSlavic => "ppocr_keys_eslav.txt",
            RecognitionModel::Thai => "ppocr_keys_th.txt",
            RecognitionModel::Greek => "ppocr_keys_el.txt",
            RecognitionModel::English => "ppocr_keys_en.txt",
            RecognitionModel::Cyrillic => "ppocr_keys_cyrillic.txt",
            RecognitionModel::Arabic => "ppocr_keys_arabic.txt",
            RecognitionModel::Devanagari => "ppocr_keys_devanagari.txt",
            RecognitionModel::Tamil => "ppocr_keys_ta.txt",
            RecognitionModel::Telugu => "ppocr_keys_te.txt",
        }
    }

    /// 获取支持的语言列表
    pub fn supported_languages(&self) -> &'static [&'static str] {
        match self {
            RecognitionModel::Chinese => {
                &["Chinese (Simplified)", "Chinese (Traditional)", "English"]
            }
            RecognitionModel::Korean => &["Korean", "English"],
            RecognitionModel::Latin => &[
                "French",
                "German",
                "Afrikaans",
                "Italian",
                "Spanish",
                "Bosnian",
                "Portuguese",
                "Czech",
                "Welsh",
                "Danish",
                "Estonian",
                "Irish",
                "Croatian",
                "Uzbek",
                "Hungarian",
                "Serbian (Latin)",
                "Indonesian",
                "Occitan",
                "Icelandic",
                "Lithuanian",
                "Maori",
                "Malay",
                "Dutch",
                "Norwegian",
                "Polish",
                "Slovak",
                "Slovenian",
                "Albanian",
                "Swedish",
                "Swahili",
                "Tagalog",
                "Turkish",
                "Latin",
                "Azerbaijani",
                "Kurdish",
                "Latvian",
                "Maltese",
                "Pali",
                "Romanian",
                "Vietnamese",
                "Finnish",
                "Basque",
                "Galician",
                "Luxembourgish",
                "Romansh",
                "Catalan",
                "Quechua",
            ],
            RecognitionModel::EastSlavic => &["Russian", "Belarusian", "Ukrainian", "English"],
            RecognitionModel::Thai => &["Thai", "English"],
            RecognitionModel::Greek => &["Greek", "English"],
            RecognitionModel::English => &["English"],
            RecognitionModel::Cyrillic => &[
                "Russian",
                "Belarusian",
                "Ukrainian",
                "Serbian (Cyrillic)",
                "Bulgarian",
                "Mongolian",
                "Abkhazian",
                "Adyghe",
                "Kabardian",
                "Avar",
                "Dargin",
                "Ingush",
                "Chechen",
                "Lak",
                "Lezgin",
                "Tabasaran",
                "Kazakh",
                "Kyrgyz",
                "Tajik",
                "Macedonian",
                "Tatar",
                "Chuvash",
                "Bashkir",
                "Malian",
                "Moldovan",
                "Udmurt",
                "Komi",
                "Ossetian",
                "Buryat",
                "Kalmyk",
                "Tuvan",
                "Sakha",
                "Karakalpak",
                "English",
            ],
            RecognitionModel::Arabic => &[
                "Arabic", "Persian", "Uyghur", "Urdu", "Pashto", "Kurdish", "Sindhi", "Balochi",
                "English",
            ],
            RecognitionModel::Devanagari => &[
                "Hindi", "Marathi", "Nepali", "Bihari", "Maithili", "Angika", "Bhojpuri", "Magahi",
                "Santali", "Newari", "Konkani", "Sanskrit", "Haryanvi", "English",
            ],
            RecognitionModel::Tamil => &["Tamil", "English"],
            RecognitionModel::Telugu => &["Telugu", "English"],
        }
    }

    /// 从字符串解析模型类型
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chinese" | "ch" | "cn" | "zh" => Some(RecognitionModel::Chinese),
            "korean" | "ko" | "kr" => Some(RecognitionModel::Korean),
            "latin" | "la" => Some(RecognitionModel::Latin),
            "eslav" | "eastslav" | "east-slavic" => Some(RecognitionModel::EastSlavic),
            "thai" | "th" => Some(RecognitionModel::Thai),
            "greek" | "el" => Some(RecognitionModel::Greek),
            "english" | "en" => Some(RecognitionModel::English),
            "cyrillic" | "cy" => Some(RecognitionModel::Cyrillic),
            "arabic" | "ar" => Some(RecognitionModel::Arabic),
            "devanagari" | "deva" | "hi" | "hindi" => Some(RecognitionModel::Devanagari),
            "tamil" | "ta" => Some(RecognitionModel::Tamil),
            "telugu" | "te" => Some(RecognitionModel::Telugu),
            _ => None,
        }
    }

    /// 获取所有可用模型
    pub fn all() -> &'static [RecognitionModel] {
        &[
            RecognitionModel::Chinese,
            RecognitionModel::Korean,
            RecognitionModel::Latin,
            RecognitionModel::EastSlavic,
            RecognitionModel::Thai,
            RecognitionModel::Greek,
            RecognitionModel::English,
            RecognitionModel::Cyrillic,
            RecognitionModel::Arabic,
            RecognitionModel::Devanagari,
            RecognitionModel::Tamil,
            RecognitionModel::Telugu,
        ]
    }
}

impl fmt::Display for RecognitionModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// 检测模型类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetectionModel {
    /// PP-OCRv5 检测模型
    #[default]
    V5,
    /// PP-OCRv5 FP16 检测模型 (更快)
    V5Fp16,
    /// PP-OCRv4 检测模型
    V4,
}

impl DetectionModel {
    /// 获取模型名称
    pub fn name(&self) -> &'static str {
        match self {
            DetectionModel::V5 => "v5",
            DetectionModel::V5Fp16 => "v5-fp16",
            DetectionModel::V4 => "v4",
        }
    }

    /// 获取模型文件名
    pub fn model_filename(&self) -> &'static str {
        match self {
            DetectionModel::V5 => "PP-OCRv5_mobile_det.mnn",
            DetectionModel::V5Fp16 => "PP-OCRv5_mobile_det_fp16.mnn",
            DetectionModel::V4 => "ch_PP-OCRv4_det_infer.mnn",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "v5" | "ppocr-v5" => Some(DetectionModel::V5),
            "v5-fp16" | "v5fp16" | "ppocr-v5-fp16" => Some(DetectionModel::V5Fp16),
            "v4" | "ppocr-v4" => Some(DetectionModel::V4),
            _ => None,
        }
    }

    /// 获取所有可用模型
    pub fn all() -> &'static [DetectionModel] {
        &[
            DetectionModel::V5,
            DetectionModel::V5Fp16,
            DetectionModel::V4,
        ]
    }
}

impl fmt::Display for DetectionModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// 内嵌模型数据
pub struct EmbeddedModels;

impl EmbeddedModels {
    /// 获取内嵌的检测模型字节 (如果有)
    #[allow(unused_variables)]
    pub fn get_det_model(model: DetectionModel) -> Option<&'static [u8]> {
        match model {
            #[cfg(feature = "embed-det-v5")]
            DetectionModel::V5 => Some(include_bytes!("../models/PP-OCRv5_mobile_det.mnn")),

            #[cfg(feature = "embed-det-v5-fp16")]
            DetectionModel::V5Fp16 => {
                Some(include_bytes!("../models/PP-OCRv5_mobile_det_fp16.mnn"))
            }

            #[cfg(feature = "embed-det-v4")]
            DetectionModel::V4 => Some(include_bytes!("../models/ch_PP-OCRv4_det_infer.mnn")),

            #[allow(unreachable_patterns)]
            _ => None,
        }
    }

    /// 获取内嵌的识别模型字节 (如果有)
    #[allow(unused_variables)]
    pub fn get_rec_model(model: RecognitionModel) -> Option<&'static [u8]> {
        match model {
            #[cfg(feature = "embed-rec-chinese")]
            RecognitionModel::Chinese => Some(include_bytes!("../models/PP-OCRv5_mobile_rec.mnn")),

            #[cfg(feature = "embed-rec-korean")]
            RecognitionModel::Korean => Some(include_bytes!(
                "../models/korean_PP-OCRv5_mobile_rec_infer.mnn"
            )),

            #[cfg(feature = "embed-rec-latin")]
            RecognitionModel::Latin => Some(include_bytes!(
                "../models/latin_PP-OCRv5_mobile_rec_infer.mnn"
            )),

            #[cfg(feature = "embed-rec-eslav")]
            RecognitionModel::EastSlavic => Some(include_bytes!(
                "../models/eslav_PP-OCRv5_mobile_rec_infer.mnn"
            )),

            #[cfg(feature = "embed-rec-thai")]
            RecognitionModel::Thai => {
                Some(include_bytes!("../models/th_PP-OCRv5_mobile_rec_infer.mnn"))
            }

            #[cfg(feature = "embed-rec-greek")]
            RecognitionModel::Greek => {
                Some(include_bytes!("../models/el_PP-OCRv5_mobile_rec_infer.mnn"))
            }

            #[cfg(feature = "embed-rec-english")]
            RecognitionModel::English => {
                Some(include_bytes!("../models/en_PP-OCRv5_mobile_rec_infer.mnn"))
            }

            #[cfg(feature = "embed-rec-cyrillic")]
            RecognitionModel::Cyrillic => Some(include_bytes!(
                "../models/cyrillic_PP-OCRv5_mobile_rec_infer.mnn"
            )),

            #[cfg(feature = "embed-rec-arabic")]
            RecognitionModel::Arabic => Some(include_bytes!(
                "../models/arabic_PP-OCRv5_mobile_rec_infer.mnn"
            )),

            #[cfg(feature = "embed-rec-devanagari")]
            RecognitionModel::Devanagari => Some(include_bytes!(
                "../models/devanagari_PP-OCRv5_mobile_rec_infer.mnn"
            )),

            #[cfg(feature = "embed-rec-tamil")]
            RecognitionModel::Tamil => {
                Some(include_bytes!("../models/ta_PP-OCRv5_mobile_rec_infer.mnn"))
            }

            #[cfg(feature = "embed-rec-telugu")]
            RecognitionModel::Telugu => {
                Some(include_bytes!("../models/te_PP-OCRv5_mobile_rec_infer.mnn"))
            }

            #[allow(unreachable_patterns)]
            _ => None,
        }
    }

    /// 获取内嵌的字符集字节 (如果有)
    #[allow(unused_variables)]
    pub fn get_charset(model: RecognitionModel) -> Option<&'static [u8]> {
        match model {
            #[cfg(feature = "embed-rec-chinese")]
            RecognitionModel::Chinese => Some(include_bytes!("../models/ppocr_keys_v5.txt")),

            #[cfg(feature = "embed-rec-korean")]
            RecognitionModel::Korean => Some(include_bytes!("../models/ppocr_keys_korean.txt")),

            #[cfg(feature = "embed-rec-latin")]
            RecognitionModel::Latin => Some(include_bytes!("../models/ppocr_keys_latin.txt")),

            #[cfg(feature = "embed-rec-eslav")]
            RecognitionModel::EastSlavic => Some(include_bytes!("../models/ppocr_keys_eslav.txt")),

            #[cfg(feature = "embed-rec-thai")]
            RecognitionModel::Thai => Some(include_bytes!("../models/ppocr_keys_th.txt")),

            #[cfg(feature = "embed-rec-greek")]
            RecognitionModel::Greek => Some(include_bytes!("../models/ppocr_keys_el.txt")),

            #[cfg(feature = "embed-rec-english")]
            RecognitionModel::English => Some(include_bytes!("../models/ppocr_keys_en.txt")),

            #[cfg(feature = "embed-rec-cyrillic")]
            RecognitionModel::Cyrillic => Some(include_bytes!("../models/ppocr_keys_cyrillic.txt")),

            #[cfg(feature = "embed-rec-arabic")]
            RecognitionModel::Arabic => Some(include_bytes!("../models/ppocr_keys_arabic.txt")),

            #[cfg(feature = "embed-rec-devanagari")]
            RecognitionModel::Devanagari => {
                Some(include_bytes!("../models/ppocr_keys_devanagari.txt"))
            }

            #[cfg(feature = "embed-rec-tamil")]
            RecognitionModel::Tamil => Some(include_bytes!("../models/ppocr_keys_ta.txt")),

            #[cfg(feature = "embed-rec-telugu")]
            RecognitionModel::Telugu => Some(include_bytes!("../models/ppocr_keys_te.txt")),

            #[allow(unreachable_patterns)]
            _ => None,
        }
    }

    /// 检查是否有内嵌的检测模型
    #[allow(dead_code)]
    pub fn has_embedded_det() -> bool {
        cfg!(feature = "embed-det-v5")
            || cfg!(feature = "embed-det-v5-fp16")
            || cfg!(feature = "embed-det-v4")
    }

    /// 检查是否有内嵌的识别模型
    #[allow(dead_code)]
    pub fn has_embedded_rec() -> bool {
        cfg!(feature = "embed-rec-chinese")
            || cfg!(feature = "embed-rec-korean")
            || cfg!(feature = "embed-rec-latin")
            || cfg!(feature = "embed-rec-eslav")
            || cfg!(feature = "embed-rec-thai")
            || cfg!(feature = "embed-rec-greek")
            || cfg!(feature = "embed-rec-english")
            || cfg!(feature = "embed-rec-cyrillic")
            || cfg!(feature = "embed-rec-arabic")
            || cfg!(feature = "embed-rec-devanagari")
            || cfg!(feature = "embed-rec-tamil")
            || cfg!(feature = "embed-rec-telugu")
    }

    /// 获取所有内嵌的识别模型
    pub fn embedded_rec_models() -> Vec<RecognitionModel> {
        let mut models = Vec::new();

        #[cfg(feature = "embed-rec-chinese")]
        models.push(RecognitionModel::Chinese);

        #[cfg(feature = "embed-rec-korean")]
        models.push(RecognitionModel::Korean);

        #[cfg(feature = "embed-rec-latin")]
        models.push(RecognitionModel::Latin);

        #[cfg(feature = "embed-rec-eslav")]
        models.push(RecognitionModel::EastSlavic);

        #[cfg(feature = "embed-rec-thai")]
        models.push(RecognitionModel::Thai);

        #[cfg(feature = "embed-rec-greek")]
        models.push(RecognitionModel::Greek);

        #[cfg(feature = "embed-rec-english")]
        models.push(RecognitionModel::English);

        #[cfg(feature = "embed-rec-cyrillic")]
        models.push(RecognitionModel::Cyrillic);

        #[cfg(feature = "embed-rec-arabic")]
        models.push(RecognitionModel::Arabic);

        #[cfg(feature = "embed-rec-devanagari")]
        models.push(RecognitionModel::Devanagari);

        #[cfg(feature = "embed-rec-tamil")]
        models.push(RecognitionModel::Tamil);

        #[cfg(feature = "embed-rec-telugu")]
        models.push(RecognitionModel::Telugu);

        models
    }

    /// 获取所有内嵌的检测模型
    pub fn embedded_det_models() -> Vec<DetectionModel> {
        let mut models = Vec::new();

        #[cfg(feature = "embed-det-v5")]
        models.push(DetectionModel::V5);

        #[cfg(feature = "embed-det-v5-fp16")]
        models.push(DetectionModel::V5Fp16);

        #[cfg(feature = "embed-det-v4")]
        models.push(DetectionModel::V4);

        models
    }
}

/// 模型路径解析器
pub struct ModelResolver {
    models_dir: Option<std::path::PathBuf>,
}

impl ModelResolver {
    /// 创建新的解析器
    pub fn new(models_dir: Option<&Path>) -> Self {
        Self {
            models_dir: models_dir.map(|p| p.to_path_buf()),
        }
    }

    /// 解析检测模型路径
    pub fn resolve_det_model(&self, model: DetectionModel) -> Option<std::path::PathBuf> {
        if let Some(ref dir) = self.models_dir {
            let path = dir.join(model.model_filename());
            if path.exists() {
                return Some(path);
            }
        }

        // 尝试当前目录下的 models 文件夹
        let local_path = std::path::PathBuf::from("models").join(model.model_filename());
        if local_path.exists() {
            return Some(local_path);
        }

        None
    }

    /// 解析识别模型路径
    pub fn resolve_rec_model(&self, model: RecognitionModel) -> Option<std::path::PathBuf> {
        if let Some(ref dir) = self.models_dir {
            let path = dir.join(model.model_filename());
            if path.exists() {
                return Some(path);
            }
        }

        let local_path = std::path::PathBuf::from("models").join(model.model_filename());
        if local_path.exists() {
            return Some(local_path);
        }

        None
    }

    /// 解析字符集路径
    pub fn resolve_charset(&self, model: RecognitionModel) -> Option<std::path::PathBuf> {
        if let Some(ref dir) = self.models_dir {
            let path = dir.join(model.charset_filename());
            if path.exists() {
                return Some(path);
            }
        }

        let local_path = std::path::PathBuf::from("models").join(model.charset_filename());
        if local_path.exists() {
            return Some(local_path);
        }

        None
    }
}

/// 打印模型信息表格
pub fn print_models_table() {
    use colored::*;

    println!();
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════════════════════════"
            .bright_blue()
    );
    println!(
        "{}",
        "                        Available Recognition Models"
            .bright_white()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════════════════════════"
            .bright_blue()
    );
    println!();

    for model in RecognitionModel::all() {
        let embedded = if EmbeddedModels::get_rec_model(*model).is_some() {
            "[embedded]".green().to_string()
        } else {
            "[external]".yellow().to_string()
        };

        println!(
            "{} {} {}",
            format!("{:<12}", model.name()).bright_cyan().bold(),
            embedded,
            format!("- {}", model.model_filename()).dimmed()
        );

        let languages = model.supported_languages();
        let lang_str = if languages.len() > 5 {
            format!(
                "{}, ... ({} languages total)",
                languages[..5].join(", "),
                languages.len()
            )
        } else {
            languages.join(", ")
        };
        println!("             {}", lang_str.dimmed());
        println!();
    }

    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════════════════════════"
            .bright_blue()
    );
    println!(
        "{}",
        "                        Available Detection Models"
            .bright_white()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════════════════════════"
            .bright_blue()
    );
    println!();

    for model in DetectionModel::all() {
        let embedded = if EmbeddedModels::get_det_model(*model).is_some() {
            "[embedded]".green().to_string()
        } else {
            "[external]".yellow().to_string()
        };

        let desc = match model {
            DetectionModel::V5 => "PP-OCRv5 detection model",
            DetectionModel::V5Fp16 => "PP-OCRv5 FP16 detection model (faster)",
            DetectionModel::V4 => "PP-OCRv4 detection model",
        };

        println!(
            "{} {} {} - {}",
            format!("{:<10}", model.name()).bright_cyan().bold(),
            embedded,
            format!("- {}", model.model_filename()).dimmed(),
            desc.dimmed()
        );
    }

    println!();
}
