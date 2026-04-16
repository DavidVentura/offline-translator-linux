use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use image::{GenericImageView, ImageReader, imageops::FilterType};
use translator::{
    BackgroundMode, BergamotEngine, CatalogSnapshot, DetectedWord, PageSegMode, ReadingOrder, Rect,
    TesseractWrapper, TextBlock, build_text_blocks, prepare_overlay_image,
    translate_texts_in_snapshot,
};

#[derive(Debug, Clone)]
pub struct ImageOverlayLine {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub foreground_argb: u32,
}

#[derive(Debug, Clone)]
pub struct ImageOverlayBlock {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub avg_line_height: f32,
    pub lines: Vec<ImageOverlayLine>,
    pub translated_text: String,
    pub background_argb: u32,
    pub foreground_argb: u32,
}

pub struct ImageTranslation {
    pub extracted_text: String,
    pub translated_text: String,
    pub image_width: u32,
    pub image_height: u32,
    pub cleaned_rgba_bytes: Vec<u8>,
    pub overlay_blocks: Vec<ImageOverlayBlock>,
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
    background_mode_label: &str,
) -> Result<ImageTranslation, String> {
    let total_start = Instant::now();
    let load_start = Instant::now();
    let loaded = load_image_rgba(image_path, max_image_size)?;
    let load_elapsed = load_start.elapsed();
    let reading_order = ReadingOrder::LeftToRight;
    let join_without_spaces = source_code == "ja";
    let relax_single_char_confidence = false;
    let background_mode = map_background_mode(background_mode_label);

    let ocr_start = Instant::now();
    let mut set_frame_elapsed = None;
    let mut word_boxes_elapsed = None;
    let mut build_blocks_elapsed = None;
    let blocks = with_ocr_engine(snapshot, source_code, reading_order, |ocr| {
        let bytes_per_pixel = 4i32;
        let width = loaded.width as i32;
        let height = loaded.height as i32;
        let bytes_per_line = width
            .checked_mul(bytes_per_pixel)
            .ok_or_else(|| "image width overflow".to_string())?;

        ocr.set_page_seg_mode(PageSegMode::PsmAuto);
        let set_frame_start = Instant::now();
        ocr.set_frame(
            &loaded.rgba_bytes,
            width,
            height,
            bytes_per_pixel,
            bytes_per_line,
        )
        .map_err(|err| format!("failed to set OCR frame: {err}"))?;
        set_frame_elapsed = Some(set_frame_start.elapsed());

        let word_boxes_start = Instant::now();
        let words = ocr
            .get_word_boxes()
            .map_err(|err| format!("failed to read OCR words: {err}"))?;
        word_boxes_elapsed = Some(word_boxes_start.elapsed());

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

        let build_blocks_start = Instant::now();
        let blocks = build_text_blocks(
            &detected_words,
            min_confidence,
            join_without_spaces,
            relax_single_char_confidence,
        );
        build_blocks_elapsed = Some(build_blocks_start.elapsed());

        Ok(blocks)
    })?;
    let ocr_elapsed = ocr_start.elapsed();

    let source_texts = blocks
        .iter()
        .map(TextBlock::translation_text)
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>();

    if source_texts.is_empty() {
        return Err("No text found in image".to_string());
    }

    let translate_start = Instant::now();
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
    let translate_elapsed = translate_start.elapsed();

    let overlay_start = Instant::now();
    let prepared = prepare_overlay_image(
        &loaded.rgba_bytes,
        loaded.width,
        loaded.height,
        &blocks,
        &translated_texts,
        background_mode,
        reading_order,
    )?;
    let overlay_elapsed = overlay_start.elapsed();

    let overlay_blocks = prepared
        .blocks
        .into_iter()
        .map(|block| ImageOverlayBlock {
            avg_line_height: if block.lines.is_empty() {
                block.bounding_box.height() as f32
            } else {
                block
                    .lines
                    .iter()
                    .map(|line| line.bounding_box.height() as f32)
                    .sum::<f32>()
                    / block.lines.len() as f32
            },
            lines: block
                .lines
                .iter()
                .map(|line| ImageOverlayLine {
                    x: line.bounding_box.left,
                    y: line.bounding_box.top,
                    width: line.bounding_box.width(),
                    height: line.bounding_box.height(),
                    foreground_argb: line.foreground_argb,
                })
                .collect(),
            x: block.bounding_box.left,
            y: block.bounding_box.top,
            width: block.bounding_box.width(),
            height: block.bounding_box.height(),
            translated_text: block.translated_text,
            background_argb: block.background_argb,
            foreground_argb: block.foreground_argb,
        })
        .collect::<Vec<_>>();

    println!(
        "image_ocr timings load={:?} ocr={:?} ocr_set_frame={:?} ocr_word_boxes={:?} ocr_build_blocks={:?} translate={:?} overlay={:?} total={:?}",
        load_elapsed,
        ocr_elapsed,
        set_frame_elapsed,
        word_boxes_elapsed,
        build_blocks_elapsed,
        translate_elapsed,
        overlay_elapsed,
        total_start.elapsed()
    );

    Ok(ImageTranslation {
        extracted_text: source_texts.join("\n\n"),
        translated_text: translated_texts.join("\n\n"),
        image_width: loaded.width,
        image_height: loaded.height,
        cleaned_rgba_bytes: prepared.rgba_bytes,
        overlay_blocks,
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
    let image = ImageReader::open(path)
        .map_err(|err| format!("Failed to open image {}: {err}", path.display()))?
        .decode()
        .map_err(|err| format!("Failed to decode image {}: {err}", path.display()))?;
    let (source_width, source_height) = image.dimensions();
    if source_width == 0 || source_height == 0 {
        return Err(format!("Failed to load image: {}", path.display()));
    }

    let (width, height) = scaled_dimensions(source_width, source_height, max_image_size);
    let rgba = if width == source_width && height == source_height {
        image.to_rgba8()
    } else {
        image
            .resize_exact(width, height, FilterType::Triangle)
            .to_rgba8()
    };

    Ok(LoadedImage {
        rgba_bytes: rgba.into_raw(),
        width,
        height,
    })
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
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

fn map_background_mode(label: &str) -> BackgroundMode {
    match label {
        "Light Background" => BackgroundMode::BlackOnWhite,
        "Dark Background" => BackgroundMode::WhiteOnBlack,
        _ => BackgroundMode::AutoDetect,
    }
}
