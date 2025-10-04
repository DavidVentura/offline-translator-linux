use slint::{self, Model, ModelExt, VecModel};
use std::error::Error;
mod translate;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use translate::Translator;

slint::include_modules!();

enum IoEvent {
    DownloadRequest(String),
    TranslationRequest {
        text: String,
        from: String,
        to: String,
    },
    Shutdown,
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let available_languages = Rc::new(VecModel::from(vec![
        Language {
            code: "en".into(),
            name: "English".into(),
            size: "45 MB".into(),
        },
        Language {
            code: "es".into(),
            name: "Spanish".into(),
            size: "45 MB".into(),
        },
        Language {
            code: "fr".into(),
            name: "French".into(),
            size: "50 MB".into(),
        },
        Language {
            code: "de".into(),
            name: "German".into(),
            size: "48 MB".into(),
        },
        Language {
            code: "nl".into(),
            name: "Dutch".into(),
            size: "42 MB".into(),
        },
    ]));

    let installed_languages = Rc::new(VecModel::from(vec![]));
    let installed_language_names = Rc::new(
        installed_languages
            .clone()
            .map(|lang: Language| lang.name.clone()),
    );

    let (bus_tx, bus_rx) = mpsc::channel::<IoEvent>();

    let ui_handle = ui.as_weak();
    let jh = std::thread::spawn(move || {
        let mut translator =
            Translator::new("/home/david/git/offline-translator-linux/lang-data/".to_string());
        translator
            .load_language_pair("en", "es")
            .expect("Couldn't load lang");

        while let Ok(msg) = bus_rx.recv() {
            match msg {
                IoEvent::DownloadRequest(code) => {
                    println!("Download language: {} ", code);
                    std::thread::sleep(Duration::from_millis(500));
                    ui_handle
                        .upgrade_in_event_loop(|ui: AppWindow| {
                            ui.invoke_language_downloaded(code.into());
                        })
                        .unwrap();
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
    });

    ui.set_available_languages(available_languages.clone().into());
    ui.set_installed_languages(installed_languages.clone().into());
    ui.set_installed_language_names(installed_language_names.into());

    if installed_languages.row_count() > 0 {
        ui.set_current_screen(Screen::Translation);
    } else {
        ui.set_current_screen(Screen::NoLanguages);
    }

    // event loop -> UI
    ui.on_language_downloaded({
        let available = available_languages.clone();
        let installed = installed_languages.clone();
        move |code| {
            println!("lang downloaded ui {code:?}");
            for i in 0..available.row_count() {
                let lang = available.row_data(i).unwrap();
                if lang.code == code {
                    available.remove(i);
                    installed.push(lang);
                    break;
                }
            }
        }
    });

    // UI -> backend
    ui.on_swap_languages({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let source = ui.get_source_language();
            let target = ui.get_target_language();

            println!("flip {source:?} {target:?}");
            ui.set_source_language(target);
            ui.set_target_language(source);
            let source = ui.get_source_language();
            let target = ui.get_target_language();
            println!("got {source:?} {target:?}");
        }
    });

    ui.on_camera_clicked({
        move || {
            println!("Camera clicked");
        }
    });

    let translate_tx = bus_tx.clone();
    ui.on_process_text({
        let ui_handle = ui.as_weak();
        move |input| {
            let ui = ui_handle.unwrap();
            let source = ui.get_source_language();
            let target = ui.get_target_language();

            translate_tx
                .send(IoEvent::TranslationRequest {
                    text: input.to_string(),
                    from: source.code.to_string(),
                    to: target.code.to_string(),
                })
                .unwrap();
        }
    });

    let dl_tx = bus_tx.clone();
    ui.on_download_language({
        move |lang| {
            dl_tx
                .send(IoEvent::DownloadRequest(lang.code.to_string()))
                .unwrap();
        }
    });

    ui.on_delete_language({
        let installed = installed_languages.clone();
        let available = available_languages.clone();
        move |lang| {
            println!("Delete language: {} ({})", lang.name, lang.code);
            for i in 0..installed.row_count() {
                if installed.row_data(i).unwrap().code == lang.code {
                    installed.remove(i);
                    break;
                }
            }
            available.push(lang);
        }
    });

    ui.on_set_from({
        let ui_handle = ui.as_weak();
        let installed = installed_languages.clone();
        move |name| {
            let ui = ui_handle.unwrap();
            for i in 0..installed.row_count() {
                if let Some(lang) = installed.row_data(i) {
                    if lang.name == name {
                        println!("set from {lang:?}");
                        ui.set_source_language(lang);
                        break;
                    }
                }
            }
        }
    });

    ui.on_set_to({
        let ui_handle = ui.as_weak();
        let installed = installed_languages.clone();
        move |name| {
            let ui = ui_handle.unwrap();
            for i in 0..installed.row_count() {
                if let Some(lang) = installed.row_data(i) {
                    if lang.name == name {
                        println!("set to {lang:?}");
                        ui.set_target_language(lang);
                        break;
                    }
                }
            }
        }
    });

    ui.run()?;
    bus_tx.send(IoEvent::Shutdown).unwrap();
    drop(bus_tx);
    drop(ui);
    jh.join().unwrap();

    Ok(())
}
