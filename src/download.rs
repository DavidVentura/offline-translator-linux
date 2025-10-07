use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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

pub fn download_file(
    url: &str,
    output_path: &Path,
    total_downloaded: Arc<AtomicUsize>,
) -> Result<(), String> {
    let mut response = ureq::get(url)
        .call()
        .map_err(|e| format!("Request failed: {}", e))?;

    let is_gzip = url.ends_with(".gz");
    let final_output_path: PathBuf = output_path.to_path_buf();
    let tmp_output_path = final_output_path.with_extension("tmp");

    let mut file =
        File::create(&tmp_output_path).map_err(|e| format!("Failed to create file: {}", e))?;

    let reader = response.body_mut().as_reader();
    let mut buffer = vec![0u8; 32 * 1024];

    let mut progress_reader = ProgressReader {
        inner: reader,
        total_downloaded: total_downloaded.clone(),
    };

    let decoder: &mut dyn Read = if is_gzip {
        &mut GzDecoder::new(progress_reader)
    } else {
        &mut progress_reader
    };

    loop {
        let bytes_read = decoder
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read/decompress from response: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Failed to write to file: {}", e))?;
    }

    std::fs::rename(tmp_output_path, final_output_path)
        .map_err(|e| format!("Failed to move tmp file: {e}"))?;

    Ok(())
}
