use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use qmetaobject::{QImage, QString};
use translator::{
    BergamotEngine, CatalogSnapshot, DetectedWord, PageSegMode, ReadingOrder, Rect, TextBlock,
    TesseractWrapper, build_text_blocks, translate_texts_in_snapshot,
};

pub struct ImageTranslation {
    pub extracted_text: String,
    pub translated_text: String,
}

struct LoadedImage {
    rgba_bytes: Vec<u8>,
    width: u32,
    height: u32,
}

struct OcrEngineState {
    engine: TesseractWrapper,
    language_spec: String,
    reading_order: ReadingOrder,
    tessdata_path: String,
}

static OCR_ENGINE: OnceLock<Mutex<Option<OcrEngineState>>> = OnceLock::new();

pub fn resolve_local_path(input: &str) -> Option<PathBuf> {
    if input.is_empty() {
        return None;
    }

    if let Some(path) = input.strip_prefix("file://") {
        let decoded = percent_decode(path);
        let local = PathBuf::from(decoded);
        if !local.as_os_str().is_empty() {
            return Some(local);
        }
    }

    let direct = PathBuf::from(percent_decode(input));
    if direct.exists() {
        return Some(direct);
    }
    Some(direct)
}

pub fn translate_image_in_snapshot(
    engine: &mut BergamotEngine,
    snapshot: &CatalogSnapshot,
    image_path: &Path,
    source_code: &str,
    target_code: &str,
    min_confidence: u32,
    max_image_size: u32,
) -> Result<ImageTranslation, String> {
    let loaded = load_image_rgba(image_path, max_image_size)?;
    let reading_order = ReadingOrder::LeftToRight;
    let join_without_spaces = source_code == "ja";
    let relax_single_char_confidence = false;

    let blocks = with_ocr_engine(snapshot, source_code, reading_order, |ocr| {
        let bytes_per_pixel = 4i32;
        let width = loaded.width as i32;
        let height = loaded.height as i32;
        let bytes_per_line = width
            .checked_mul(bytes_per_pixel)
            .ok_or_else(|| "image width overflow".to_string())?;

        ocr.set_page_seg_mode(PageSegMode::PsmAuto);
        ocr.set_frame(
            &loaded.rgba_bytes,
            width,
            height,
            bytes_per_pixel,
            bytes_per_line,
        )
        .map_err(|err| format!("failed to set OCR frame: {err}"))?;

        let words = ocr
            .get_word_boxes()
            .map_err(|err| format!("failed to read OCR words: {err}"))?;

        let detected_words = words
            .into_iter()
            .map(|word| DetectedWord {
                text: word.text,
                confidence: word.confidence,
                bounding_box: Rect {
                    left: word.bounding_rect.left as u32,
                    top: word.bounding_rect.top as u32,
                    right: word.bounding_rect.right as u32,
                    bottom: word.bounding_rect.bottom as u32,
                },
                is_at_beginning_of_para: word.is_at_beginning_of_para,
                end_para: word.end_para,
                end_line: word.end_line,
            })
            .collect::<Vec<_>>();

        Ok(build_text_blocks(
            &detected_words,
            min_confidence,
            join_without_spaces,
            relax_single_char_confidence,
        ))
    })?;

    let source_texts = blocks
        .iter()
        .map(TextBlock::translation_text)
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>();

    if source_texts.is_empty() {
        return Err("No text found in image".to_string());
    }

    let translated_texts = if source_code == target_code {
        source_texts.clone()
    } else {
        match translate_texts_in_snapshot(engine, snapshot, source_code, target_code, &source_texts)
        {
            Some(Ok(values)) => values,
            Some(Err(message)) => return Err(message),
            None => {
                return Err(format!(
                    "Missing installed language pair {source_code}->{target_code}"
                ));
            }
        }
    };

    Ok(ImageTranslation {
        extracted_text: source_texts.join("\n\n"),
        translated_text: translated_texts.join("\n\n"),
    })
}

fn with_ocr_engine<T, F>(
    snapshot: &CatalogSnapshot,
    source_code: &str,
    reading_order: ReadingOrder,
    f: F,
) -> Result<T, String>
where
    F: FnOnce(&mut TesseractWrapper) -> Result<T, String>,
{
    let language = snapshot
        .catalog
        .language_by_code(source_code)
        .ok_or_else(|| format!("unknown source language: {source_code}"))?;

    let tessdata_path = Path::new(&snapshot.base_dir)
        .join("tesseract")
        .join("tessdata");
    let has_japanese_vertical_model =
        source_code == "ja" && tessdata_path.join("jpn_vert.traineddata").exists();
    let language_spec = match (source_code, reading_order, has_japanese_vertical_model) {
        ("ja", ReadingOrder::TopToBottomLeftToRight, true) => "jpn_vert".to_string(),
        _ => format!("{}+eng", language.tess_name),
    };

    let mut slot = OCR_ENGINE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| "OCR engine mutex poisoned".to_string())?;

    let tessdata_path_string = tessdata_path.to_string_lossy().into_owned();
    let needs_reinit = slot.as_ref().is_none_or(|state| {
        state.language_spec != language_spec
            || state.reading_order != reading_order
            || state.tessdata_path != tessdata_path_string
    });

    if needs_reinit {
        let engine = TesseractWrapper::new(
            Some(
                tessdata_path
                    .to_str()
                    .ok_or_else(|| "invalid tessdata path".to_string())?,
            ),
            Some(&language_spec),
        )
        .map_err(|err| format!("failed to initialize tesseract: {err}"))?;
        *slot = Some(OcrEngineState {
            engine,
            language_spec,
            reading_order,
            tessdata_path: tessdata_path_string,
        });
    }

    let state = slot
        .as_mut()
        .ok_or_else(|| "OCR engine unavailable".to_string())?;
    f(&mut state.engine)
}

fn load_image_rgba(path: &Path, max_image_size: u32) -> Result<LoadedImage, String> {
    let image = QImage::load_from_file(QString::from(path.to_string_lossy().into_owned()));
    let source_size = image.size();
    if source_size.width == 0 || source_size.height == 0 {
        return Err(format!("Failed to load image: {}", path.display()));
    }

    let (width, height) = scaled_dimensions(source_size.width, source_size.height, max_image_size);
    let mut rgba_bytes = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        let source_y = y * source_size.height / height;
        for x in 0..width {
            let source_x = x * source_size.width / width;
            let color = image.get_pixel_color(source_x, source_y);
            rgba_bytes.push(color.red() as u8);
            rgba_bytes.push(color.green() as u8);
            rgba_bytes.push(color.blue() as u8);
            rgba_bytes.push(color.alpha() as u8);
        }
    }

    Ok(LoadedImage {
        rgba_bytes,
        width,
        height,
    })
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len()
            && let Ok(hex) = std::str::from_utf8(&bytes[index + 1..index + 3])
            && let Ok(value) = u8::from_str_radix(hex, 16)
        {
            decoded.push(value);
            index += 3;
            continue;
        }
        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

fn scaled_dimensions(width: u32, height: u32, max_image_size: u32) -> (u32, u32) {
    if max_image_size == 0 {
        return (width.max(1), height.max(1));
    }

    let largest = width.max(height);
    if largest <= max_image_size {
        return (width.max(1), height.max(1));
    }

    if width >= height {
        let scaled_height = ((height as u64 * max_image_size as u64) / width as u64).max(1);
        (max_image_size, scaled_height as u32)
    } else {
        let scaled_width = ((width as u64 * max_image_size as u64) / height as u64).max(1);
        (scaled_width as u32, max_image_size)
    }
}
