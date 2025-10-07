mod data;
mod download;
mod eventloop;
mod index;
mod translate;

use flate2::read::GzDecoder;
use slint::{self, ComponentHandle, FilterModel, MapModel, Model, VecModel};
use std::error::Error;
use std::io::Read;
use std::rc::Rc;
use std::sync::mpsc::{self, Sender};

use crate::data::INDEX_JSON;
use crate::index::{Index, IndexLanguage};

slint::include_modules!();

enum IoEvent {
    DownloadRequest(String),
    SetDataPath(String),
    TranslationRequest {
        text: String,
        from: String,
        to: String,
    },
    Shutdown,
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let (bus_tx, bus_rx) = mpsc::channel::<IoEvent>();
    let default_index = read_default_index();

    setup_language_models(&ui, &default_index, bus_tx.clone());

    let ui_handle = ui.as_weak();
    let jh = std::thread::spawn(move || eventloop::run_eventloop(bus_rx, ui_handle, default_index));

    ui.set_current_screen(Screen::NoLanguages);
    let data_path = "/home/david/git/offline-translator-linux/lang-data/".to_string();

    bus_tx.send(IoEvent::SetDataPath(data_path)).unwrap();
    ui.run()?;
    bus_tx.send(IoEvent::Shutdown).unwrap();
    drop(bus_tx);
    drop(ui);
    jh.join().unwrap();

    Ok(())
}

fn read_default_index() -> Index {
    let mut decoder = GzDecoder::new(INDEX_JSON);
    let mut index_json = String::new();
    decoder
        .read_to_string(&mut index_json)
        .expect("Failed to decompress gzip data");

    let mut default_index: Index =
        miniserde::json::from_str(&index_json).expect("Failed to deserialize Index");

    default_index.languages.push(IndexLanguage {
        code: "en".to_string(),
        name: "English".to_string(),
        script: "Latin".to_string(),
        from: None,
        to: None,
        extra_files: vec![],
    });
    default_index
}

fn setup_language_models(ui: &AppWindow, default_index: &Index, bus_tx: Sender<IoEvent>) {
    let all_languages: Vec<IndexLanguage> = default_index.languages.iter().cloned().collect();

    let from_languages = all_languages
        .iter()
        .filter(|il| il.from.is_some() || il.code == "en")
        .map(|il| Language::from(il))
        .collect::<Vec<Language>>();

    let to_languages = all_languages
        .iter()
        .filter(|il| il.to.is_some() || il.code == "en")
        .map(|il| Language::from(il))
        .collect::<Vec<Language>>();

    let mut all_languages_m: Vec<Language> =
        all_languages.iter().map(|il| Language::from(il)).collect();
    all_languages_m.sort_by(|a, b| a.name.cmp(&b.name));

    let ui_from_languages = Rc::new(VecModel::from(from_languages));
    let ui_to_languages = Rc::new(VecModel::from(to_languages));
    let ui_all_languages = Rc::new(VecModel::from(all_languages_m));

    let all_installed_languages = Rc::new(FilterModel::new(
        ui_all_languages.clone(),
        |l: &Language| l.installed || l.code == "en",
    ));
    let not_installed_languages = Rc::new(FilterModel::new(
        ui_all_languages.clone(),
        |l: &Language| !l.installed,
    ));

    let installed_from_languages = Rc::new(FilterModel::new(
        all_installed_languages.clone(),
        |l: &Language| match l.direction {
            Direction::ToOnly => false,
            Direction::FromOnly | Direction::Both => true,
        },
    ));
    let installed_to_languages = Rc::new(FilterModel::new(
        all_installed_languages.clone(),
        |l: &Language| match l.direction {
            Direction::FromOnly => false,
            Direction::ToOnly | Direction::Both => true,
        },
    ));

    let installed_from_language_names = Rc::new(MapModel::new(
        installed_from_languages.clone(),
        |lang: Language| lang.name,
    ));
    let installed_to_language_names = Rc::new(MapModel::new(
        installed_to_languages.clone(),
        |lang: Language| lang.name,
    ));

    ui.set_all_languages(ui_all_languages.clone().into());
    ui.set_installed_from_languages(ui_from_languages.clone().into());
    ui.set_installed_to_languages(ui_to_languages.clone().into());

    ui.set_all_installed_languages(all_installed_languages.clone().into());
    ui.set_not_installed_languages(not_installed_languages.clone().into());

    ui.set_installed_from_languages(installed_from_languages.clone().into());
    ui.set_installed_from_language_names(installed_from_language_names.clone().into());

    ui.set_installed_to_languages(installed_to_languages.clone().into());
    ui.set_installed_to_language_names(installed_to_language_names.clone().into());

    // setup callbacks
    setup_eventloop_callbacks(&ui, ui_all_languages.clone());
    setup_ui_callbacks(&ui, bus_tx, ui_all_languages.clone());
}

fn setup_eventloop_callbacks(ui: &AppWindow, all_languages: Rc<VecModel<Language>>) {
    // event loop -> UI
    let clear = all_languages.clone();
    ui.on_languages_cleared({
        move || {
            for i in 0..clear.row_count() {
                let mut lang = clear.row_data(i).unwrap();
                lang.installed = false;
                clear.set_row_data(i, lang);
            }
            println!("cleared");
        }
    });

    ui.on_language_downloaded({
        move |code| {
            println!("lang downloaded ui {code:?}");
            for i in 0..all_languages.row_count() {
                let mut lang = all_languages.row_data(i).unwrap();
                if lang.code == code {
                    lang.installed = true;
                    all_languages.set_row_data(i, lang);
                    break;
                }
            }
        }
    });

    ui.on_download_progress({
        move |code, percent| {
            println!("Download progress for {}: {:.1}%", code, percent);
        }
    });
}

fn setup_ui_callbacks(
    ui: &AppWindow,
    bus_tx: Sender<IoEvent>,
    all_languages: Rc<VecModel<Language>>,
) {
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
        let all_languages = all_languages.clone();
        move |lang| {
            println!("Delete language: {} ({})", lang.name, lang.code);
            for i in 0..all_languages.row_count() {
                let mut row_lang = all_languages.row_data(i).unwrap();
                if row_lang.code == lang.code {
                    row_lang.installed = false;
                    all_languages.set_row_data(i, row_lang);
                    break;
                }
            }
        }
    });

    ui.on_set_from({
        let ui_handle = ui.as_weak();
        let all_languages = all_languages.clone();
        move |name| {
            let ui = ui_handle.unwrap();
            for i in 0..all_languages.row_count() {
                if let Some(lang) = all_languages.row_data(i) {
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
        let all_languages = all_languages.clone();
        move |name| {
            let ui = ui_handle.unwrap();
            for i in 0..all_languages.row_count() {
                if let Some(lang) = all_languages.row_data(i) {
                    if lang.name == name {
                        println!("set to {lang:?}");
                        ui.set_target_language(lang);
                        break;
                    }
                }
            }
        }
    });
}
