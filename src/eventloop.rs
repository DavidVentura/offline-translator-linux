use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::download;
use crate::translate::Translator;
use crate::{AppWindow, IoEvent};

pub fn run_eventloop(
    bus_rx: Receiver<IoEvent>,
    ui_handle: slint::Weak<AppWindow>,
    translator: Translator,
) {
    while let Ok(msg) = bus_rx.recv() {
        match msg {
            IoEvent::DownloadRequest(code) => {
                println!("Download language: {} ", code);

                let url = "https://translator.davidv.dev/dictionaries/1/en.dict";
                let output_path = Path::new("/tmp").join(format!("{}.zip", code));

                match download::download_file(&url, &output_path, code.clone(), &ui_handle) {
                    Ok(_) => {
                        println!("Download completed for {}", code);
                    }
                    Err(e) => {
                        eprintln!("Download failed for {}: {}", code, e);
                    }
                }
            }
            IoEvent::TranslationRequest { text, from, to } => {
                let lines: Vec<&str> = text.split("\n").collect();
                let start = Instant::now();
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
