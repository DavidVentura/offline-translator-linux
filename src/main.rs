use slint::{self, Model, ModelExt, VecModel};
use std::error::Error;
mod translate;
use std::rc::Rc;
use translate::Translator;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let mut translator =
        Translator::new("/home/david/git/firefox-translations-models/models/tiny/enes".to_string());
    translator.load_language_pair("en", "es").unwrap();

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

    ui.set_available_languages(available_languages.clone().into());
    ui.set_installed_languages(installed_languages.clone().into());
    ui.set_installed_language_names(installed_language_names.into());

    if installed_languages.row_count() > 0 {
        ui.set_current_screen(Screen::Translation);
    } else {
        ui.set_current_screen(Screen::NoLanguages);
    }
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

    ui.on_process_text({
        let ui_handle = ui.as_weak();
        move |input| {
            let ui = ui_handle.unwrap();
            let lines: Vec<&str> = input.split("\n").collect();
            let source = ui.get_source_language();
            let target = ui.get_target_language();

            let res = match translator.translate(
                source.code.as_str(),
                target.code.as_str(),
                lines.as_slice(),
            ) {
                Ok(result) => result.join("\n").into(),
                Err(message) => message.into(),
            };
            ui.set_output_text(res);
        }
    });

    ui.on_download_language({
        let installed = installed_languages.clone();
        let available = available_languages.clone();
        move |lang| {
            println!("Download language: {} ({})", lang.name, lang.code);
            installed.push(lang.clone());
            for i in 0..available.row_count() {
                if available.row_data(i).unwrap().code == lang.code {
                    available.remove(i);
                    break;
                }
            }
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

    Ok(())
}
