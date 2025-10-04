use crate::AppWindow;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub fn download_file(
    url: &str,
    output_path: &Path,
    language_code: String,
    ui_handle: &slint::Weak<AppWindow>,
) -> Result<(), String> {
    let mut response = ureq::get(url)
        .call()
        .map_err(|e| format!("Request failed: {}", e))?;

    let content_length = response
        .headers()
        .get("Content-Length")
        .ok_or("Missing Content-Length header")?
        .to_str()
        .map_err(|e| format!("Invalid Content-Length: {}", e))?
        .parse::<usize>()
        .map_err(|e| format!("Failed to parse Content-Length: {}", e))?;

    let mut file =
        File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;

    let mut reader = response.body_mut().as_reader();
    let mut buffer = vec![0u8; 512 * 1024];
    let mut downloaded = 0usize;
    let mut last_update = 0usize;
    const UPDATE_THRESHOLD: usize = 512 * 1024;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read from response: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        downloaded += bytes_read;

        if downloaded - last_update >= UPDATE_THRESHOLD {
            let percent = (downloaded as f32 / content_length as f32) * 100.0;
            let code = language_code.clone();
            ui_handle
                .upgrade_in_event_loop(move |ui: AppWindow| {
                    ui.invoke_download_progress(code.into(), percent);
                })
                .map_err(|_| "Failed to update UI")?;
            last_update = downloaded;
        }
    }

    ui_handle
        .upgrade_in_event_loop(|ui: AppWindow| {
            ui.invoke_language_downloaded(language_code.into());
        })
        .map_err(|_| "Failed to update UI")?;

    Ok(())
}
