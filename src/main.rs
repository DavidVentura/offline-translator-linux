use slint;
use std::error::Error;
mod translate;
use translate::Translator;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let mut translator =
        Translator::new("/home/david/git/firefox-translations-models/models/tiny/enes".to_string());
    println!("loaded");
    translator.load_language_pair("en", "es").unwrap();

    ui.set_has_languages(false);

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

    ui.on_history_clicked({
        move || {
            println!("History clicked");
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

    ui.run()?;

    Ok(())
}
