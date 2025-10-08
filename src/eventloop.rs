use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use cld2::{Format, detect_language};
use rayon::prelude::*;

use crate::index::{Index, IndexLanguage};
use crate::translate::Translator;
use crate::{AppPaths, AppWindow, IoEvent};
use crate::{Screen, download};

pub fn run_eventloop(bus_rx: Receiver<IoEvent>, ui_handle: slint::Weak<AppWindow>, index: Index) {
    let mut translator = None::<Translator>;
    let mut app_paths = None::<AppPaths>;

    while let Ok(msg) = bus_rx.recv() {
        match msg {
            IoEvent::SetAppPaths(path) => {
                app_paths = Some(path.clone());
                ui_handle
                    .upgrade_in_event_loop(move |ui: AppWindow| {
                        ui.invoke_languages_cleared();
                    })
                    .expect("Failed to update UI");

                let mut has_languages = false;
                let load_start = Instant::now();
                std::fs::create_dir_all(&path.data).expect("can't make data dir");
                std::fs::create_dir_all(&path.config).expect("can't make data dir");
                let avail_files = std::fs::read_dir(&path.data).expect("bad data path");
                let avail_files: HashSet<String> = HashSet::from_iter(
                    avail_files
                        .into_iter()
                        .map(|f| f.unwrap().file_name().into_string().unwrap()),
                );

                for lang in &index.languages {
                    let lang_files: HashSet<String> =
                        HashSet::from_iter(lang.files().iter().map(|f| f.name.clone()));
                    if avail_files.is_superset(&lang_files) {
                        let code = lang.code.clone();
                        if code != "en" {
                            has_languages = true;
                        }
                        ui_handle
                            .upgrade_in_event_loop(move |ui: AppWindow| {
                                ui.invoke_language_downloaded(code.into());
                            })
                            .expect("Failed to update UI");
                    }
                }

                if has_languages {
                    println!("has langs");
                    translator = Some(Translator::new(path.data.clone()));
                    ui_handle
                        .upgrade_in_event_loop(move |ui: AppWindow| {
                            ui.set_current_screen(Screen::Translation);
                        })
                        .expect("Failed to update UI");
                } else {
                    println!("has NO langs");
                }

                println!("Load took {:?}", load_start.elapsed());
            }
            IoEvent::DownloadRequest(code) => {
                if app_paths.is_none() {
                    println!("no app path, cant download");
                    continue;
                }
                let app_paths = app_paths.clone().unwrap();
                let lang = index
                    .languages
                    .iter()
                    .filter(|l| l.code == code)
                    .next()
                    .expect("Received illegal code for download")
                    .clone();

                let ui_handle_clone = ui_handle.clone();
                let data_path_clone = app_paths.data.clone();
                let jh = std::thread::spawn(move || {
                    download(&lang, ui_handle_clone, &data_path_clone);
                });
                jh.join().expect("thread panicked");
                if translator.is_none() {
                    translator = Some(Translator::new(app_paths.data.clone()));
                    println!("init translator on download");
                }
            }
            IoEvent::DeleteLanguage(code) => {
                if app_paths.is_none() {
                    println!("no app path, cant download");
                    continue;
                }
                let app_paths = app_paths.clone().unwrap();
                println!("Deleting language: {}", code);

                if let Some(lang) = index.languages.iter().find(|l| l.code == code) {
                    let mut files = lang.files();
                    files.sort_by(|a, b| a.name.cmp(&b.name));
                    files.dedup_by_key(|f| f.name.clone());

                    for file in files {
                        let file_path = Path::new(&app_paths.data).join(&file.name);
                        match std::fs::remove_file(&file_path) {
                            Ok(_) => println!("Deleted file: {}", file.name),
                            Err(e) => eprintln!("Failed to delete {}: {}", file.name, e),
                        }
                    }
                } else {
                    eprintln!("Language not found in index: {}", code);
                }
            }
            IoEvent::TranslationRequest { text, from, to } => {
                send_detection_to_ui(&text, &ui_handle);
                if let Some(ref mut translator) = translator {
                    let lines: Vec<&str> = text.split("\n").collect();
                    let start = Instant::now();

                    if let Err(e) = translator.load_language_pair(&from, &to) {
                        ui_handle
                            .upgrade_in_event_loop(move |ui: AppWindow| {
                                ui.set_output_text(
                                    format!("Couldn't load language pair {from}->{to}: {e}").into(),
                                );
                            })
                            .unwrap();
                        continue;
                    }
                    let result = match translator.translate(&from, &to, lines.as_slice()) {
                        Ok(result) => result.join("\n"),
                        Err(message) => message,
                    };
                    println!("translation took {:?} = '{}'", start.elapsed(), result);
                    ui_handle
                        .upgrade_in_event_loop(move |ui: AppWindow| {
                            ui.set_output_text(result.into());
                        })
                        .unwrap();
                } else {
                    println!("no translator, idk");
                }
            }
            IoEvent::Shutdown => {
                println!("shutdown signal, exiting");
                break;
            }
        }
    }
    println!("all senders done, closing");
}

fn download(lang: &IndexLanguage, ui_handle: slint::Weak<AppWindow>, data_path: &str) {
    let code = lang.code.clone();
    println!("Download language: {} ", code);

    let mut files = lang.files();
    files.sort_by(|a, b| a.name.cmp(&b.name));
    files.dedup_by_key(|f| f.name.clone());

    let total_size: usize = files.iter().map(|f| f.size_bytes as usize).sum();
    println!("total size {total_size}");
    let total_downloaded = Arc::new(AtomicUsize::new(0));
    let download_complete = Arc::new(AtomicBool::new(false));

    let progress_total_downloaded = total_downloaded.clone();
    let progress_download_complete = download_complete.clone();
    let progress_ui_handle = ui_handle.clone();
    let progress_code = code.clone();

    let startup_code = progress_code.clone();
    let _ = progress_ui_handle.upgrade_in_event_loop(move |ui: AppWindow| {
        ui.invoke_download_progress(startup_code.into(), 0.00001);
    });

    let progress_thread = thread::spawn(move || {
        const UPDATE_THRESHOLD: usize = 512 * 1024;
        let mut last_update = 0;

        while !progress_download_complete.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(33));

            let current = progress_total_downloaded.load(Ordering::Relaxed);
            if current - last_update >= UPDATE_THRESHOLD {
                let percent = current as f32 / total_size as f32;
                let code = progress_code.clone();
                let _ = progress_ui_handle.upgrade_in_event_loop(move |ui: AppWindow| {
                    ui.invoke_download_progress(code.into(), percent);
                });
                last_update = current;
            }
        }
    });

    let results: Vec<Result<(), String>> = files
        .par_iter()
        .map(|file| {
            let output_path = Path::new(data_path).join(&file.name);
            download::download_file(&file.url, &output_path, total_downloaded.clone())
        })
        .collect();

    download_complete.store(true, Ordering::Relaxed);
    progress_thread.join().expect("Progress thread panicked");

    let success = results.iter().all(|r| r.is_ok());

    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(_) => {
                println!("Download completed for {} file {}", code, files[i].name);
            }
            Err(e) => {
                eprintln!("Download failed for {} file {}: {}", code, files[i].name, e);
            }
        }
    }

    if success {
        let lang_code = lang.code.clone();
        ui_handle
            .upgrade_in_event_loop(|ui: AppWindow| {
                ui.invoke_language_downloaded(lang_code.into());
            })
            .expect("Failed to update UI");
    }
}

fn send_detection_to_ui(text: &str, ui_handle: &slint::Weak<AppWindow>) {
    // TODO detect in another thread?
    let (detected, reliable) = detect_language(&text, Format::Text);

    let code = match (detected, reliable) {
        (Some(c), cld2::Reliable) => c.0,
        _ => "",
    };
    ui_handle
        .upgrade_in_event_loop(move |ui: AppWindow| {
            ui.invoke_set_detected_language_code(code.to_string().into());
        })
        .unwrap();
}
