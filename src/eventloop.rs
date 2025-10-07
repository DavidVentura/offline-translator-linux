use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::index::{Index, IndexLanguage};
use crate::translate::Translator;
use crate::{AppWindow, IoEvent};
use crate::{Screen, download};

pub fn run_eventloop(bus_rx: Receiver<IoEvent>, ui_handle: slint::Weak<AppWindow>, index: Index) {
    let mut translator = None::<Translator>;

    while let Ok(msg) = bus_rx.recv() {
        match msg {
            IoEvent::SetDataPath(data_path) => {
                ui_handle
                    .upgrade_in_event_loop(move |ui: AppWindow| {
                        ui.invoke_languages_cleared();
                    })
                    .expect("Failed to update UI");

                let mut has_languages = false;
                let load_start = Instant::now();
                let avail_files = std::fs::read_dir(&data_path).expect("bad data path");
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
                    println!("has langs");
                    translator = Some(Translator::new(data_path));
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
                let lang = index
                    .languages
                    .iter()
                    .filter(|l| l.code == code)
                    .next()
                    .expect("Received illegal code for download");

                download(&lang, ui_handle.clone());
            }
            IoEvent::TranslationRequest { text, from, to } => {
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

fn download(lang: &IndexLanguage, ui_handle: slint::Weak<AppWindow>) {
    let code = lang.code.clone();
    println!("Download language: {} ", code);

    // TODO threaded?
    // TODO file path from settings
    let mut success = true;
    for file in lang.files() {
        let output_path = Path::new("/tmp").join(file.name);

        match download::download_file(&file.url, &output_path, code.clone(), &ui_handle) {
            Ok(_) => {
                println!("Download completed for {}", code);
            }
            Err(e) => {
                success = false;
                eprintln!("Download failed for {}: {}", code, e);
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
