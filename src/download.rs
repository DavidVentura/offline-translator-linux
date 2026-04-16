use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use translator::{DownloadPlan, DownloadTask};
use zip::ZipArchive;

const USER_AGENT: &str = concat!("offline-translator-linux/", env!("CARGO_PKG_VERSION"));

struct ProgressReader<R> {
    inner: R,
    total_downloaded: Arc<AtomicUsize>,
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.total_downloaded.fetch_add(n, Ordering::Relaxed);
        Ok(n)
    }
}

pub fn execute_download_plan(
    base_dir: &str,
    plan: &DownloadPlan,
    total_downloaded: Arc<AtomicUsize>,
) -> Result<(), String> {
    for task in &plan.tasks {
        download_task(base_dir, task, total_downloaded.clone())?;
    }
    Ok(())
}

fn download_task(
    base_dir: &str,
    task: &DownloadTask,
    total_downloaded: Arc<AtomicUsize>,
) -> Result<(), String> {
    let output_path = Path::new(base_dir).join(&task.install_path);
    if task.archive_format.as_deref() == Some("zip") && task.extract_to.is_some() {
        let archive_path = output_path.clone();
        download_to_path(&task.url, &archive_path, false, total_downloaded)?;
        extract_zip(
            base_dir,
            &archive_path,
            task.extract_to.as_deref().unwrap_or(""),
            task.delete_after_extract,
            task.install_marker_path.as_deref(),
            task.install_marker_version,
        )
    } else {
        download_to_path(&task.url, &output_path, task.decompress, total_downloaded)
    }
}

fn download_to_path(
    url: &str,
    output_path: &Path,
    decompress: bool,
    total_downloaded: Arc<AtomicUsize>,
) -> Result<(), String> {
    let mut response = ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("Request failed: {e}"))?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent dir: {e}"))?;
    }

    let tmp_output_path = output_path.with_extension("tmp");
    let mut file =
        File::create(&tmp_output_path).map_err(|e| format!("Failed to create file: {e}"))?;

    let reader = response.body_mut().as_reader();
    let mut buffer = vec![0u8; 32 * 1024];

    let progress_reader = ProgressReader {
        inner: reader,
        total_downloaded,
    };

    let mut decoder: Box<dyn Read> = if decompress {
        Box::new(GzDecoder::new(progress_reader))
    } else {
        Box::new(progress_reader)
    };

    loop {
        let bytes_read = decoder
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read/decompress from response: {e}"))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to write to file: {e}"))?;
    }

    fs::rename(&tmp_output_path, output_path)
        .map_err(|e| format!("Failed to move tmp file: {e}"))?;

    Ok(())
}

fn extract_zip(
    base_dir: &str,
    archive_path: &Path,
    extract_to: &str,
    delete_after_extract: bool,
    install_marker_path: Option<&str>,
    install_marker_version: Option<i32>,
) -> Result<(), String> {
    let extract_root = Path::new(base_dir).join(extract_to);
    let install_root_name = install_marker_path
        .map(|path| Path::new(path).parent().and_then(Path::file_name))
        .flatten()
        .map(|value| value.to_string_lossy().to_string());

    {
        let file = File::open(archive_path).map_err(|e| format!("Failed to open zip: {e}"))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {e}"))?;
        let mut managed_paths = Vec::new();

        for i in 0..archive.len() {
            let entry = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {e}"))?;
            let normalized = normalized_entry_name(entry.name(), install_root_name.as_deref());
            let parts = normalized
                .split('/')
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>();
            if parts.len() >= 2 {
                managed_paths.push(extract_root.join(parts[0]).join(parts[1]));
            } else if let Some(first) = parts.first() {
                managed_paths.push(extract_root.join(first));
            }
        }

        managed_paths.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
        managed_paths.dedup();
        for path in managed_paths {
            if path.exists() {
                let _ = fs::remove_dir_all(&path);
                let _ = fs::remove_file(&path);
            }
        }
    }

    {
        let file = File::open(archive_path).map_err(|e| format!("Failed to reopen zip: {e}"))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {e}"))?;
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {e}"))?;
            let normalized = normalized_entry_name(entry.name(), install_root_name.as_deref());
            if normalized.is_empty() {
                continue;
            }
            let output = extract_root.join(&normalized);
            if entry.is_dir() {
                fs::create_dir_all(&output)
                    .map_err(|e| format!("Failed to create zip dir {}: {e}", output.display()))?;
                continue;
            }
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create parent dir {}: {e}", parent.display())
                })?;
            }
            let mut out = File::create(&output).map_err(|e| {
                format!("Failed to create extracted file {}: {e}", output.display())
            })?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| format!("Failed to extract {}: {e}", output.display()))?;
        }
    }

    if let (Some(marker_path), Some(version)) = (install_marker_path, install_marker_version) {
        let marker_file = Path::new(base_dir).join(marker_path);
        if let Some(parent) = marker_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create marker dir {}: {e}", parent.display()))?;
        }
        let json = format!("{{\"version\":{version}}}\n");
        fs::write(&marker_file, json)
            .map_err(|e| format!("Failed to write marker {}: {e}", marker_file.display()))?;
    }

    if delete_after_extract && archive_path.exists() {
        let _ = fs::remove_file(archive_path);
    }

    Ok(())
}

fn normalized_entry_name(entry_name: &str, install_root_name: Option<&str>) -> String {
    let trimmed = entry_name.trim_start_matches('/').trim_start_matches("./");
    if trimmed.is_empty() {
        return String::new();
    }

    match install_root_name {
        Some(root_name)
            if trimmed != root_name && !trimmed.starts_with(&format!("{root_name}/")) =>
        {
            format!("{root_name}/{trimmed}")
        }
        _ => trimmed.to_string(),
    }
}
