use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use cld2::{Format, detect_language};
use image::{ImageBuffer, Rgba};
use translator::{BergamotEngine, CatalogSnapshot, LanguageCatalog, translate_texts_in_snapshot};

use crate::catalog_state::{
    build_snapshot, delete_plan_for_feature, download_plan_for_feature, languages_from_snapshot,
    remove_delete_plan,
};
use crate::download;
use crate::image_ocr;
use crate::model::FeatureKind;
use crate::tts;
use crate::ui::{ImageOverlayListItem, TtsVoiceListItem, UiCallbacks, argb_to_qml_color};
use crate::{AppPaths, IoEvent};

pub fn run_eventloop(bus_rx: Receiver<IoEvent>, ui: UiCallbacks, catalog: LanguageCatalog) {
    let mut engine = BergamotEngine::new();
    let mut app_paths = None::<AppPaths>;
    let mut snapshot = None::<CatalogSnapshot>;

    while let Ok(msg) = bus_rx.recv() {
        match msg {
            IoEvent::SetAppPaths(path) => {
                app_paths = Some(path.clone());

                let load_start = Instant::now();
                std::fs::create_dir_all(&path.data).expect("can't make data dir");
                std::fs::create_dir_all(&path.config).expect("can't make config dir");

                let new_snapshot = build_snapshot(&catalog, &path.data);
                let languages = languages_from_snapshot(&new_snapshot);
                (ui.set_languages)(languages);
                snapshot = Some(new_snapshot);
                println!("Load took {:?}", load_start.elapsed());
            }
            IoEvent::DownloadRequest { code, feature } => {
                let Some(app_paths) = app_paths.clone() else {
                    println!("no app path, cant download");
                    continue;
                };
                let Some(current_snapshot) = snapshot.as_ref() else {
                    println!("no snapshot, cant download");
                    continue;
                };

                if let Some(plan) = download_plan_for_feature(current_snapshot, &code, feature) {
                    if let Err(err) = download_feature(&code, feature, &plan, &app_paths.data, &ui)
                    {
                        eprintln!("Download failed for {code}: {err}");
                    }
                }

                let new_snapshot = build_snapshot(&catalog, &app_paths.data);
                let languages = languages_from_snapshot(&new_snapshot);
                (ui.set_languages)(languages);
                snapshot = Some(new_snapshot);
            }
            IoEvent::DeleteLanguage { code, feature } => {
                let Some(app_paths) = app_paths.clone() else {
                    println!("no app path, cant delete");
                    continue;
                };
                let Some(current_snapshot) = snapshot.as_ref() else {
                    println!("no snapshot, cant delete");
                    continue;
                };

                let delete_plan = delete_plan_for_feature(current_snapshot, &code, feature);
                remove_delete_plan(&app_paths.data, &delete_plan);

                let new_snapshot = build_snapshot(&catalog, &app_paths.data);
                let languages = languages_from_snapshot(&new_snapshot);
                (ui.set_languages)(languages);
                snapshot = Some(new_snapshot);
            }
            IoEvent::TranslationRequest { text, from, to } => {
                send_detection_to_ui(&text, &ui);
                let Some(current_snapshot) = snapshot.as_ref() else {
                    continue;
                };

                let lines = text
                    .split('\n')
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let start = Instant::now();

                let result = if from == to {
                    Ok(lines.join("\n"))
                } else {
                    match translate_texts_in_snapshot(
                        &mut engine,
                        current_snapshot,
                        &from,
                        &to,
                        &lines,
                    ) {
                        Some(Ok(values)) => Ok(values.join("\n")),
                        Some(Err(message)) => Err(message),
                        None => Err(format!("Missing installed language pair {from}->{to}")),
                    }
                };

                let text = match result {
                    Ok(result) => result,
                    Err(message) => message,
                };
                println!("translation took {:?} = '{}'", start.elapsed(), text);
                (ui.set_output_text)(text);
            }
            IoEvent::RefreshTtsVoices {
                language_code,
                selected_voice_name,
            } => {
                let Some(current_snapshot) = snapshot.as_ref() else {
                    continue;
                };

                match tts::load_tts_voices(
                    current_snapshot,
                    &language_code,
                    (!selected_voice_name.is_empty()).then_some(selected_voice_name.as_str()),
                ) {
                    Ok(result) => {
                        let items = result
                            .voices
                            .into_iter()
                            .map(|voice| TtsVoiceListItem {
                                name: voice.name.into(),
                                display_name: voice.display_name.into(),
                            })
                            .collect::<Vec<_>>();
                        (ui.set_tts_voices)(
                            result.available,
                            items,
                            result.selected_voice_name,
                            result.selected_voice_display_name,
                        );
                    }
                    Err(err) => {
                        eprintln!("Failed to load TTS voices: {err}");
                        (ui.set_tts_voices)(false, Vec::new(), String::new(), "Default".to_string());
                    }
                }
            }
            IoEvent::SpeakRequest {
                language_code,
                text,
                speech_speed,
                voice_name,
            } => {
                let Some(current_snapshot) = snapshot.as_ref() else {
                    continue;
                };
                tts::play_text_async(
                    current_snapshot.clone(),
                    language_code,
                    text,
                    speech_speed,
                    (!voice_name.is_empty()).then_some(voice_name),
                    ui.clone(),
                );
            }
            IoEvent::StopTts => {
                tts::stop_playback();
                (ui.set_tts_state)(false, false);
            }
            IoEvent::ImageTranslationRequest {
                image_path,
                from,
                to,
                min_confidence,
                max_image_size,
                background_mode,
            } => {
                let Some(current_snapshot) = snapshot.as_ref() else {
                    continue;
                };

                let start = Instant::now();
                let result = image_ocr::translate_image_in_snapshot(
                    &mut engine,
                    current_snapshot,
                    std::path::Path::new(&image_path),
                    &from,
                    &to,
                    min_confidence,
                    max_image_size,
                    &background_mode,
                );

                match result {
                    Ok(image_translation) => {
                        if let Some(paths) = app_paths.as_ref()
                            && let Ok(image_url) = persist_processed_image(&paths.data, &image_translation)
                        {
                            (ui.set_selected_image_url)(image_url);
                        }
                        send_detection_to_ui(&image_translation.extracted_text, &ui);
                        (ui.set_input_text)(image_translation.extracted_text);
                        (ui.set_output_text)(image_translation.translated_text);
                        let overlay_items = image_translation
                            .overlay_blocks
                            .into_iter()
                            .map(|block| ImageOverlayListItem {
                                block_x: block.x as f32,
                                block_y: block.y as f32,
                                block_width: block.width as f32,
                                block_height: block.height as f32,
                                translated_text: block.translated_text.into(),
                                background_color: argb_to_qml_color(block.background_argb),
                                foreground_color: argb_to_qml_color(block.foreground_argb),
                            })
                            .collect::<Vec<_>>();
                        (ui.set_image_overlay)(
                            overlay_items,
                            image_translation.image_width as f32,
                            image_translation.image_height as f32,
                        );
                    }
                    Err(message) => {
                        (ui.set_input_text)(String::new());
                        (ui.set_output_text)(message);
                        (ui.set_image_overlay)(Vec::new(), 0.0, 0.0);
                    }
                }
                println!("image translation took {:?}", start.elapsed());
            }
            IoEvent::Shutdown => {
                tts::stop_playback();
                println!("shutdown signal, exiting");
                break;
            }
        }
    }
    println!("all senders done, closing");
}

fn persist_processed_image(
    data_dir: &str,
    image_translation: &image_ocr::ImageTranslation,
) -> Result<String, String> {
    let render_dir = std::path::Path::new(data_dir).join("image-renders");
    std::fs::create_dir_all(&render_dir)
        .map_err(|err| format!("failed to create image render dir: {err}"))?;

    let render_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system clock error: {err}"))?
        .as_millis();
    let image_path = render_dir.join(format!("ocr-render-{render_id}.png"));

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
        image_translation.image_width,
        image_translation.image_height,
        image_translation.cleaned_rgba_bytes.clone(),
    )
    .ok_or_else(|| "failed to build rendered image buffer".to_string())?;

    image
        .save(&image_path)
        .map_err(|err| format!("failed to save rendered image: {err}"))?;

    Ok(format!("file://{}", image_path.display()))
}

fn download_feature(
    code: &str,
    feature: FeatureKind,
    plan: &translator::DownloadPlan,
    data_path: &str,
    ui: &UiCallbacks,
) -> Result<(), String> {
    let total_size = plan.total_size.max(1) as usize;
    let total_downloaded = Arc::new(AtomicUsize::new(0));
    let download_complete = Arc::new(AtomicBool::new(false));

    (ui.set_feature_progress)(code.to_string(), feature, 0.00001);

    let progress_total_downloaded = total_downloaded.clone();
    let progress_download_complete = download_complete.clone();
    let progress_ui = ui.clone();
    let progress_code = code.to_string();

    let progress_thread = thread::spawn(move || {
        const UPDATE_THRESHOLD: usize = 1024 * 1024;
        const UPDATE_INTERVAL: Duration = Duration::from_millis(120);
        let mut last_update = 0;

        while !progress_download_complete.load(Ordering::Relaxed) {
            thread::sleep(UPDATE_INTERVAL);

            let current = progress_total_downloaded.load(Ordering::Relaxed);
            if current.saturating_sub(last_update) >= UPDATE_THRESHOLD {
                let percent = current as f32 / total_size as f32;
                (progress_ui.set_feature_progress)(progress_code.clone(), feature, percent);
                last_update = current;
            }
        }
    });

    let result = download::execute_download_plan(data_path, plan, total_downloaded);
    download_complete.store(true, Ordering::Relaxed);
    progress_thread.join().expect("Progress thread panicked");

    result
}

fn send_detection_to_ui(text: &str, ui: &UiCallbacks) {
    let (detected, reliable) = detect_language(text, Format::Text);

    let code = match (detected, reliable) {
        (Some(c), cld2::Reliable) => c.0,
        _ => "",
    };
    (ui.set_detected_language_code)(code.to_string());
}
