use std::path::{Path, PathBuf};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};
use translator::{
    CatalogSnapshot, PcmAudio, ResolvedTtsVoiceFiles, TtsVoiceOption, list_voices,
    plan_speech_chunks_for_text, resolve_tts_voice_files_in_snapshot, synthesize_pcm,
};

use crate::ui::UiCallbacks;

pub struct TtsVoiceRefresh {
    pub available: bool,
    pub voices: Vec<TtsVoiceOption>,
    pub selected_voice_name: String,
    pub selected_voice_display_name: String,
}

struct ActivePlayback {
    _stream: OutputStream,
    sink: Sink,
}

static PLAYBACK_GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn stop_playback() {
    PLAYBACK_GENERATION.fetch_add(1, Ordering::SeqCst);
}

pub fn load_tts_voices(
    snapshot: &CatalogSnapshot,
    language_code: &str,
    selected_voice_name: Option<&str>,
) -> Result<TtsVoiceRefresh, String> {
    let Some(files) = resolve_tts_files(snapshot, language_code) else {
        return Ok(TtsVoiceRefresh {
            available: false,
            voices: Vec::new(),
            selected_voice_name: String::new(),
            selected_voice_display_name: "Default".to_string(),
        });
    };

    let voices = catch_tts_panic(|| {
        list_voices(
            &files.engine,
            &absolute_install_path(snapshot, &files.model_install_path),
            &absolute_install_path(snapshot, &files.aux_install_path),
            support_data_root(snapshot, &files).as_deref(),
            language_code,
        )
    })?;

    let selected_voice_name = selected_voice_name
        .filter(|value| voices.iter().any(|voice| voice.name == **value))
        .map(ToOwned::to_owned)
        .or_else(|| voices.first().map(|voice| voice.name.clone()))
        .unwrap_or_default();

    let selected_voice_display_name = if voices.len() <= 1 {
        "Default".to_string()
    } else {
        voices
            .iter()
            .find(|voice| voice.name == selected_voice_name)
            .map(|voice| voice.display_name.clone())
            .unwrap_or_else(|| "Default".to_string())
    };

    Ok(TtsVoiceRefresh {
        available: true,
        voices,
        selected_voice_name,
        selected_voice_display_name,
    })
}

pub fn play_text_async(
    snapshot: CatalogSnapshot,
    language_code: String,
    text: String,
    speech_speed: f32,
    voice_name: Option<String>,
    ui: UiCallbacks,
) {
    let generation = PLAYBACK_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    (ui.set_tts_state)(true, false);

    thread::spawn(move || {
        let result = synthesize_full_audio(
            &snapshot,
            &language_code,
            &text,
            speech_speed,
            voice_name.as_deref(),
        );

        if PLAYBACK_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }

        match result {
            Ok(audio) => {
                let playback = start_playback(audio, generation);
                match playback {
                    Ok(playback) => {
                        (ui.set_tts_state)(false, true);
                        while !playback.sink.empty() {
                            if PLAYBACK_GENERATION.load(Ordering::SeqCst) != generation {
                                playback.sink.stop();
                                break;
                            }
                            thread::sleep(Duration::from_millis(50));
                        }
                        if PLAYBACK_GENERATION.load(Ordering::SeqCst) == generation {
                            (ui.set_tts_state)(false, false);
                        }
                    }
                    Err(err) => {
                        eprintln!("TTS playback failed: {err}");
                        if PLAYBACK_GENERATION.load(Ordering::SeqCst) == generation {
                            (ui.set_tts_state)(false, false);
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("TTS synthesis failed: {err}");
                if PLAYBACK_GENERATION.load(Ordering::SeqCst) == generation {
                    (ui.set_tts_state)(false, false);
                }
            }
        }
    });
}

fn resolve_tts_files(
    snapshot: &CatalogSnapshot,
    language_code: &str,
) -> Option<ResolvedTtsVoiceFiles> {
    resolve_tts_voice_files_in_snapshot(snapshot, language_code)
}

fn absolute_install_path(snapshot: &CatalogSnapshot, relative_path: &str) -> String {
    Path::new(&snapshot.base_dir)
        .join(relative_path)
        .display()
        .to_string()
}

fn support_data_root(snapshot: &CatalogSnapshot, files: &ResolvedTtsVoiceFiles) -> Option<String> {
    let model_path = Path::new(&snapshot.base_dir).join(&files.model_install_path);
    let mut candidates = Vec::<PathBuf>::new();

    candidates.push(Path::new(&snapshot.base_dir).join("bin"));
    candidates.push(PathBuf::from(&snapshot.base_dir));

    for ancestor in model_path.ancestors() {
        candidates.push(ancestor.to_path_buf());
    }

    candidates
        .into_iter()
        .find(|candidate| candidate.join("espeak-ng-data").is_dir())
        .map(|path| path.display().to_string())
}

fn synthesize_full_audio(
    snapshot: &CatalogSnapshot,
    language_code: &str,
    text: &str,
    speech_speed: f32,
    voice_name: Option<&str>,
) -> Result<PcmAudio, String> {
    let files = resolve_tts_files(snapshot, language_code)
        .ok_or_else(|| format!("No TTS voice installed for {language_code}"))?;
    let model_path = absolute_install_path(snapshot, &files.model_install_path);
    let aux_path = absolute_install_path(snapshot, &files.aux_install_path);
    let support_data_root = support_data_root(snapshot, &files);
    let speaker_id = files.speaker_id.map(i64::from);
    let planned_chunks = catch_tts_panic(|| {
        plan_speech_chunks_for_text(
            &files.engine,
            &model_path,
            &aux_path,
            support_data_root.as_deref(),
            language_code,
            text,
        )
    })?;

    if planned_chunks.is_empty() {
        return Err("Nothing to speak".to_string());
    }

    let mut sample_rate = None::<i32>;
    let mut combined_samples = Vec::<i16>::new();

    for chunk in planned_chunks {
        let pcm = catch_tts_panic(|| {
            synthesize_pcm(
                &files.engine,
                &model_path,
                &aux_path,
                support_data_root.as_deref(),
                language_code,
                &chunk.content,
                speech_speed,
                voice_name,
                speaker_id,
                chunk.is_phonemes,
            )
        })?;

        let current_sample_rate = sample_rate.get_or_insert(pcm.sample_rate);
        if *current_sample_rate != pcm.sample_rate {
            return Err("Mismatched sample rates across synthesized speech chunks".to_string());
        }

        combined_samples.extend(pcm.pcm_samples);

        if let Some(pause_after_ms) = chunk.pause_after_ms {
            let silence = PcmAudio::silence(*current_sample_rate, pause_after_ms);
            combined_samples.extend(silence.pcm_samples);
        }
    }

    Ok(PcmAudio {
        sample_rate: sample_rate.unwrap_or(22_050),
        pcm_samples: combined_samples,
    })
}

fn start_playback(audio: PcmAudio, generation: u64) -> Result<ActivePlayback, String> {
    let (stream, handle) =
        OutputStream::try_default().map_err(|err| format!("No audio output available: {err}"))?;
    if PLAYBACK_GENERATION.load(Ordering::SeqCst) != generation {
        return Err("Playback superseded".to_string());
    }

    let sink = Sink::try_new(&handle).map_err(|err| format!("Failed to create sink: {err}"))?;
    let source = SamplesBuffer::new(1, audio.sample_rate as u32, audio.pcm_samples);
    sink.append(source);
    sink.play();
    Ok(ActivePlayback {
        _stream: stream,
        sink,
    })
}

fn catch_tts_panic<T, F>(f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    catch_unwind(AssertUnwindSafe(f)).map_err(|payload| {
        if let Some(message) = payload.downcast_ref::<&str>() {
            format!("TTS runtime panicked: {message}")
        } else if let Some(message) = payload.downcast_ref::<String>() {
            format!("TTS runtime panicked: {message}")
        } else {
            "TTS runtime panicked".to_string()
        }
    })?
}
