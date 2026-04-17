use std::cmp::min;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::thread;
use std::time::Duration;

use translator::PcmAudio;

const PA_STREAM_PLAYBACK: c_int = 1;
const PA_SAMPLE_S16LE: c_int = 3;
const PLAYBACK_CHUNK_SAMPLES: usize = 2_048;
const PLAYBACK_SLEEP_SLICE_MS: u64 = 20;

#[repr(C)]
struct PaSampleSpec {
    format: c_int,
    rate: u32,
    channels: u8,
}

#[repr(C)]
struct PaSimple {
    _private: [u8; 0],
}

#[link(name = "pulse-simple")]
unsafe extern "C" {
    fn pa_simple_new(
        server: *const c_char,
        name: *const c_char,
        dir: c_int,
        dev: *const c_char,
        stream_name: *const c_char,
        ss: *const PaSampleSpec,
        map: *const c_void,
        attr: *const c_void,
        error: *mut c_int,
    ) -> *mut PaSimple;
    fn pa_simple_free(s: *mut PaSimple);
    fn pa_simple_write(
        s: *mut PaSimple,
        data: *const c_void,
        bytes: usize,
        error: *mut c_int,
    ) -> c_int;
    fn pa_simple_flush(s: *mut PaSimple, error: *mut c_int) -> c_int;
}

#[link(name = "pulse")]
unsafe extern "C" {
    fn pa_strerror(error: c_int) -> *const c_char;
}

struct PulsePlayback {
    handle: *mut PaSimple,
}

pub struct PulsePlaybackStream {
    playback: PulsePlayback,
    sample_rate: i32,
}

impl PulsePlayback {
    fn new(sample_rate: i32) -> Result<Self, String> {
        if sample_rate <= 0 {
            return Err(format!("Invalid sample rate for playback: {sample_rate}"));
        }

        let app_name = CString::new("Offline translator").unwrap();
        let stream_name = CString::new("Text to speech").unwrap();
        let sample_spec = PaSampleSpec {
            format: PA_SAMPLE_S16LE,
            rate: sample_rate as u32,
            channels: 1,
        };
        let mut error = 0;

        let handle = unsafe {
            pa_simple_new(
                std::ptr::null(),
                app_name.as_ptr(),
                PA_STREAM_PLAYBACK,
                std::ptr::null(),
                stream_name.as_ptr(),
                &sample_spec,
                std::ptr::null(),
                std::ptr::null(),
                &mut error,
            )
        };

        if handle.is_null() {
            return Err(format!(
                "Failed to connect to PulseAudio: {}",
                pulse_error_message(error)
            ));
        }

        Ok(Self { handle })
    }

    fn write_samples(&self, samples: &[i16]) -> Result<(), String> {
        let mut bytes = Vec::with_capacity(std::mem::size_of_val(samples));
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }

        let mut error = 0;
        let result = unsafe {
            pa_simple_write(
                self.handle,
                bytes.as_ptr().cast::<c_void>(),
                bytes.len(),
                &mut error,
            )
        };

        if result < 0 {
            return Err(format!(
                "PulseAudio write failed: {}",
                pulse_error_message(error)
            ));
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), String> {
        let mut error = 0;
        let result = unsafe { pa_simple_flush(self.handle, &mut error) };
        if result < 0 {
            return Err(format!(
                "PulseAudio flush failed: {}",
                pulse_error_message(error)
            ));
        }

        Ok(())
    }
}

impl Drop for PulsePlayback {
    fn drop(&mut self) {
        unsafe { pa_simple_free(self.handle) };
    }
}

impl PulsePlaybackStream {
    pub fn new(sample_rate: i32) -> Result<Self, String> {
        Ok(Self {
            playback: PulsePlayback::new(sample_rate)?,
            sample_rate,
        })
    }

    pub fn write_audio<F>(&self, audio: &PcmAudio, should_stop: &mut F) -> Result<(), String>
    where
        F: FnMut() -> bool,
    {
        if audio.sample_rate != self.sample_rate {
            return Err(format!(
                "Mismatched sample rate for playback stream: expected {}, got {}",
                self.sample_rate, audio.sample_rate
            ));
        }

        self.write_samples(&audio.pcm_samples, should_stop)
    }

    pub fn write_pause_ms<F>(&self, duration_ms: i32, should_stop: &mut F) -> Result<(), String>
    where
        F: FnMut() -> bool,
    {
        if duration_ms <= 0 {
            return Ok(());
        }

        let silence = PcmAudio::silence(self.sample_rate, duration_ms);
        self.write_samples(&silence.pcm_samples, should_stop)
    }

    pub fn flush(&self) -> Result<(), String> {
        self.playback.flush()
    }

    fn write_samples<F>(&self, samples: &[i16], should_stop: &mut F) -> Result<(), String>
    where
        F: FnMut() -> bool,
    {
        if should_stop() {
            let _ = self.playback.flush();
            return Ok(());
        }

        for chunk in samples.chunks(PLAYBACK_CHUNK_SAMPLES) {
            if should_stop() {
                let _ = self.playback.flush();
                return Ok(());
            }

            self.playback.write_samples(chunk)?;
            sleep_for_chunk(chunk.len(), self.sample_rate, should_stop, &self.playback)?;
        }

        Ok(())
    }
}

fn sleep_for_chunk<F>(
    chunk_len: usize,
    sample_rate: i32,
    should_stop: &mut F,
    playback: &PulsePlayback,
) -> Result<(), String>
where
    F: FnMut() -> bool,
{
    if sample_rate <= 0 || chunk_len == 0 {
        return Ok(());
    }

    let chunk_ms = ((chunk_len as u128 * 1_000) / sample_rate as u128)
        .max(1)
        .try_into()
        .unwrap_or(u64::MAX);
    sleep_interruptibly(chunk_ms, should_stop);

    if should_stop() {
        let _ = playback.flush();
    }

    Ok(())
}

fn sleep_interruptibly<F>(duration_ms: u64, should_stop: &mut F)
where
    F: FnMut() -> bool,
{
    let mut remaining = duration_ms;
    while remaining > 0 {
        if should_stop() {
            break;
        }

        let step = min(remaining, PLAYBACK_SLEEP_SLICE_MS);
        thread::sleep(Duration::from_millis(step));
        remaining -= step;
    }
}

fn pulse_error_message(error: c_int) -> String {
    let message = unsafe { pa_strerror(error) };
    if message.is_null() {
        return format!("unknown PulseAudio error ({error})");
    }

    unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned()
}
