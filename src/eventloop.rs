use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use cld2::{Format, detect_language};
use translator::{BergamotEngine, CatalogSnapshot, LanguageCatalog, translate_texts_in_snapshot};

use crate::catalog_state::{
    build_snapshot, delete_plan_for_feature, download_plan_for_feature, languages_from_snapshot,
    remove_delete_plan,
};
use crate::download;
use crate::model::{FeatureKind, Screen};
use crate::ui::UiCallbacks;
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
                let has_languages = languages
                    .iter()
                    .any(|language| !language.built_in && language.core_installed);

                (ui.set_languages)(languages);
                if has_languages {
                    (ui.set_current_screen)(Screen::Translation);
                }
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
                let has_languages = languages
                    .iter()
                    .any(|language| !language.built_in && language.core_installed);
                (ui.set_languages)(languages);
                if has_languages {
                    (ui.set_current_screen)(Screen::Translation);
                }
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
            IoEvent::Shutdown => {
                println!("shutdown signal, exiting");
                break;
            }
        }
    }
    println!("all senders done, closing");
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
