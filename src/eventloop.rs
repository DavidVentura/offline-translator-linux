use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::index::Index;
use crate::translate::Translator;
use crate::{AppWindow, IoEvent};
use crate::{Screen, download};

pub fn run_eventloop(
    bus_rx: Receiver<IoEvent>,
    ui_handle: slint::Weak<AppWindow>,
    mut translator: Translator,
    index: Index,
    data_path: &str,
) {
    while let Ok(msg) = bus_rx.recv() {
        match msg {
            IoEvent::StartupLoadLanguages => {
                let mut has_languages = false;
                let load_start = Instant::now();
                let avail_files = std::fs::read_dir(data_path).expect("bad data path");
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
                        has_languages = true;
                        ui_handle
                            .upgrade_in_event_loop(move |ui: AppWindow| {
                                ui.invoke_language_downloaded(code.into());
                            })
                            .expect("Failed to update UI");
                    }
                }

                ui_handle
                    .upgrade_in_event_loop(move |ui: AppWindow| {
                        ui.invoke_language_downloaded("en".into());
                    })
                    .expect("Failed to update UI");

                if has_languages {
                    ui_handle
                        .upgrade_in_event_loop(move |ui: AppWindow| {
                            ui.set_current_screen(Screen::Translation);
                        })
                        .expect("Failed to update UI");
                }

                println!("Load took {:?}", load_start.elapsed());
            }
            IoEvent::DownloadRequest(code) => {
                println!("Download language: {} ", code);

                let lang = index
                    .languages
                    .iter()
                    .filter(|l| l.code == code)
                    .next()
                    .expect("Received illegal code for download");

                // TODO threaded?
                for file in lang.files() {
                    let output_path = Path::new("/tmp").join(format!("{}.zip", code));

                    match download::download_file(&file.url, &output_path, code.clone(), &ui_handle)
                    {
                        Ok(_) => {
                            println!("Download completed for {}", code);
                        }
                        Err(e) => {
                            eprintln!("Download failed for {}: {}", code, e);
                        }
                    }
                }
            }
            IoEvent::TranslationRequest { text, from, to } => {
                let lines: Vec<&str> = text.split("\n").collect();
                let start = Instant::now();

                translator
                    .load_language_pair(&from, &to)
                    .expect("Couldn't load lang");
                let result = match translator.translate(&from, &to, lines.as_slice()) {
                    Ok(result) => result.join("\n"),
                    Err(message) => message,
                };
                println!("translation took {:?}", start.elapsed());
                ui_handle
                    .upgrade_in_event_loop(move |ui: AppWindow| {
                        ui.set_output_text(result.into());
                    })
                    .unwrap();
            }
            IoEvent::Shutdown => {
                println!("shutdown signal, exiting");
                break;
            }
        }
    }
    println!("all senders done, closing");
}
