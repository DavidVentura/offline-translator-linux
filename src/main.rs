use slint::{self, Model, VecModel};
use std::error::Error;
mod translate;
use std::rc::Rc;
use translate::Translator;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let mut translator =
        Translator::new("/home/david/git/firefox-translations-models/models/tiny/enes".to_string());
    println!("loaded");
    translator.load_language_pair("en", "es").unwrap();

    let available_languages = Rc::new(VecModel::from(vec![
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
    ui.set_available_languages(available_languages.clone().into());
    ui.set_installed_languages(installed_languages.clone().into());

    ui.on_swap_languages({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let source = ui.get_source_language();
            let target = ui.get_target_language();
            ui.set_source_language(target.into());
            ui.set_target_language(source.into());
        }
    });

    ui.on_camera_clicked({
        move || {
            println!("Camera clicked");
        }
    });

    ui.on_process_text(move |input| {
        let lines: Vec<&str> = input.split("\n").collect();
        translator
            .translate("en", "es", lines.as_slice())
            .unwrap()
            .join("\n")
            .into()
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

    ui.run()?;

    Ok(())
}
