mod catalog_state;
mod download;
mod eventloop;
mod image_ocr;
mod model;
mod pulse;
mod settings;
mod tts;
mod ui;

use qmetaobject::*;
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::catalog_state::{build_snapshot, bundled_catalog, languages_from_snapshot};
use crate::model::FeatureKind;
use crate::settings::load_settings;
use crate::ui::{AppBridge, create_ui_callbacks};

const APP_NAME: &str = "dev.davidv.translator";

#[derive(Clone, Debug)]
struct AppPaths {
    config: String,
    data: String,
}

enum IoEvent {
    DownloadRequest {
        code: String,
        feature: FeatureKind,
        selected_tts_pack_id: Option<String>,
    },
    DeleteLanguage {
        code: String,
        feature: FeatureKind,
    },
    SetAppPaths(AppPaths),
    TranslationRequest {
        text: String,
        from: String,
        to: String,
    },
    ImageTranslationRequest {
        image_path: String,
        from: String,
        to: String,
        min_confidence: u32,
        max_image_size: u32,
        background_mode: String,
    },
    RefreshTtsVoices {
        language_code: String,
        selected_voice_name: String,
    },
    WarmTtsModel {
        language_code: String,
    },
    SpeakRequest {
        language_code: String,
        text: String,
        speech_speed: f32,
        voice_name: String,
    },
    StopTts,
    Shutdown,
}

fn get_app_paths() -> AppPaths {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/home/{}", whoami::username())));
    let data_root = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"));
    let config_root = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));

    AppPaths {
        data: data_root.join(APP_NAME).display().to_string(),
        config: config_root.join(APP_NAME).display().to_string(),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    configure_onnxruntime_dylib_path()?;
    qmetaobject::log::init_qt_to_rust();

    let (bus_tx, bus_rx) = mpsc::channel::<IoEvent>();
    let app_paths = get_app_paths();
    let catalog = bundled_catalog();
    let initial_snapshot = build_snapshot(&catalog, &app_paths.data);
    let initial_languages = languages_from_snapshot(&initial_snapshot);
    let main_qml = find_main_qml()?;
    let asset_dir = find_asset_dir(&main_qml)?;
    let settings = load_settings(&app_paths.config);
    let mut engine = QmlEngine::new();
    let app = QObjectBox::new(AppBridge::new(
        initial_languages,
        bus_tx.clone(),
        asset_dir,
        app_paths.config.clone(),
        app_paths.data.clone(),
        settings,
    ));

    engine.set_object_property("app".into(), app.pinned());

    let ui_callbacks = create_ui_callbacks(QPointer::from(app.pinned().borrow()));
    let jh = std::thread::spawn(move || eventloop::run_eventloop(bus_rx, ui_callbacks, catalog));

    bus_tx.send(IoEvent::SetAppPaths(app_paths)).unwrap();
    engine.load_file(main_qml.into());
    engine.exec();

    bus_tx.send(IoEvent::Shutdown).unwrap();
    drop(bus_tx);
    jh.join().unwrap();

    Ok(())
}

fn configure_onnxruntime_dylib_path() -> Result<(), Box<dyn Error>> {
    if std::env::var_os("ORT_DYLIB_PATH").is_some() {
        return Ok(());
    }

    let current_exe = std::env::current_exe()?;
    let exe_dir = current_exe
        .parent()
        .map(PathBuf::from)
        .ok_or("current executable has no parent directory")?;
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let target_arch = std::env::consts::ARCH;
    let arch_relative = format!("runtime-lib/{target_arch}/libonnxruntime.so");
    let mut candidates = Vec::new();
    candidates.push(exe_dir.join("libonnxruntime.so"));
    candidates.push(exe_dir.join(&arch_relative));

    for ancestor in current_exe.ancestors().skip(1).take(5) {
        candidates.push(ancestor.join("libonnxruntime.so"));
        candidates.push(ancestor.join(&arch_relative));
    }

    candidates.push(manifest_dir.join(&arch_relative));

    if let Some(path) = candidates.iter().find(|path| path.is_file()) {
        // Set once during process startup before worker threads are spawned.
        unsafe { std::env::set_var("ORT_DYLIB_PATH", path) };
    }

    Ok(())
}

fn find_main_qml() -> Result<String, Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dev_path = manifest_dir.join("qml/Main.qml");
    if std::env::var_os("CLICKABLE_DESKTOP_MODE").is_some() && dev_path.exists() {
        return Ok(dev_path.canonicalize()?.display().to_string());
    }

    let exe = std::env::current_exe()?;
    if let Some(qml_path) = exe
        .parent()
        .and_then(|bin_dir| bin_dir.parent())
        .map(|qml_dir| qml_dir.join("Main.qml"))
        .filter(|path| path.exists())
    {
        return Ok(qml_path.display().to_string());
    }

    if dev_path.exists() {
        return Ok(dev_path.display().to_string());
    }

    Err("Could not locate Main.qml".into())
}

fn find_asset_dir(main_qml: &str) -> Result<String, Box<dyn Error>> {
    let main_qml = PathBuf::from(main_qml);
    let candidates = [
        main_qml.parent().map(|dir| dir.join("../assets")),
        main_qml.parent().map(|dir| dir.join("../../assets")),
    ];

    for candidate in candidates.into_iter().flatten() {
        let candidate = candidate.canonicalize().unwrap_or(candidate);
        if candidate.exists() {
            return Ok(candidate.display().to_string());
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dev_path = manifest_dir.join("assets");
    if dev_path.exists() {
        return Ok(dev_path.canonicalize()?.display().to_string());
    }

    Err("Could not locate assets directory".into())
}
