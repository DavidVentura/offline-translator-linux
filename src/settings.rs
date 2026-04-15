use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct Settings {
    pub default_from: String,
    pub default_to: String,
    pub font_size: i32,
    pub ocr_background_mode: String,
    pub ocr_min_confidence: i32,
    pub ocr_max_image_size: i32,
    pub catalog_index_url: String,
    pub disable_ocr: bool,
    pub disable_auto_detect: bool,
    pub show_transliteration_output: bool,
    pub show_transliteration_input: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_from: "English".to_string(),
            default_to: "English".to_string(),
            font_size: 16,
            ocr_background_mode: "Auto-detect Colors".to_string(),
            ocr_min_confidence: 75,
            ocr_max_image_size: 1500,
            catalog_index_url: "https://offline-translator.davidv.dev/index".to_string(),
            disable_ocr: false,
            disable_auto_detect: false,
            show_transliteration_output: false,
            show_transliteration_input: false,
        }
    }
}

pub fn load_settings(config_dir: &str) -> Settings {
    let path = Path::new(config_dir).join("settings.json");
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };

    eprintln!("settings: loading from {}", path.display());
    let mut settings = Settings::default();
    if let Some(v) = json_string(&contents, "default_from") { settings.default_from = v; }
    if let Some(v) = json_string(&contents, "default_to") { settings.default_to = v; }
    if let Some(v) = json_i32(&contents, "font_size") { settings.font_size = v; }
    if let Some(v) = json_string(&contents, "ocr_background_mode") { settings.ocr_background_mode = v; }
    if let Some(v) = json_i32(&contents, "ocr_min_confidence") { settings.ocr_min_confidence = v; }
    if let Some(v) = json_i32(&contents, "ocr_max_image_size") { settings.ocr_max_image_size = v; }
    if let Some(v) = json_string(&contents, "catalog_index_url") { settings.catalog_index_url = v; }
    if let Some(v) = json_bool(&contents, "disable_ocr") { settings.disable_ocr = v; }
    if let Some(v) = json_bool(&contents, "disable_auto_detect") { settings.disable_auto_detect = v; }
    if let Some(v) = json_bool(&contents, "show_transliteration_output") { settings.show_transliteration_output = v; }
    if let Some(v) = json_bool(&contents, "show_transliteration_input") { settings.show_transliteration_input = v; }
    eprintln!("settings: loaded font_size={} ocr_min_confidence={} ocr_max_image_size={} ocr_bg={}", settings.font_size, settings.ocr_min_confidence, settings.ocr_max_image_size, settings.ocr_background_mode);
    settings
}

pub fn save_settings(config_dir: &str, settings: &Settings) {
    let path = Path::new(config_dir).join("settings.json");
    let json = format!(
        r#"{{
  "default_from": "{}",
  "default_to": "{}",
  "font_size": {},
  "ocr_background_mode": "{}",
  "ocr_min_confidence": {},
  "ocr_max_image_size": {},
  "catalog_index_url": "{}",
  "disable_ocr": {},
  "disable_auto_detect": {},
  "show_transliteration_output": {},
  "show_transliteration_input": {}
}}"#,
        escape_json(&settings.default_from),
        escape_json(&settings.default_to),
        settings.font_size,
        escape_json(&settings.ocr_background_mode),
        settings.ocr_min_confidence,
        settings.ocr_max_image_size,
        escape_json(&settings.catalog_index_url),
        settings.disable_ocr,
        settings.disable_auto_detect,
        settings.show_transliteration_output,
        settings.show_transliteration_input,
    );
    let _ = fs::write(path, json);
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let pos = json.find(&pattern)?;
    let after = &json[pos + pattern.len()..];
    let after = after.trim_start().strip_prefix(':')?;
    let after = after.trim_start().strip_prefix('"')?;
    let end = after.find('"')?;
    Some(after[..end].replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn json_i32(json: &str, key: &str) -> Option<i32> {
    let pattern = format!("\"{}\"", key);
    let pos = json.find(&pattern)?;
    let after = &json[pos + pattern.len()..];
    let after = after.trim_start().strip_prefix(':')?;
    let after = after.trim_start();
    let end = after.find(|c: char| !c.is_ascii_digit() && c != '-')?;
    after[..end].parse().ok()
}

fn json_bool(json: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\"", key);
    let pos = json.find(&pattern)?;
    let after = &json[pos + pattern.len()..];
    let after = after.trim_start().strip_prefix(':')?;
    let after = after.trim_start();
    if after.starts_with("true") {
        Some(true)
    } else if after.starts_with("false") {
        Some(false)
    } else {
        None
    }
}
