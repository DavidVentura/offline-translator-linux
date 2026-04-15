use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::mpsc::{RecvTimeoutError, SyncSender, sync_channel};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use translator::{
    CatalogSnapshot, PcmAudio, ResolvedTtsVoiceFiles, TtsVoiceOption, list_voices,
    plan_speech_chunks_for_text, resolve_tts_voice_files_in_snapshot, synthesize_pcm,
};

use crate::pulse::PulsePlaybackStream;
use crate::ui::UiCallbacks;

pub struct TtsVoiceRefresh {
    pub available: bool,
    pub voices: Vec<TtsVoiceOption>,
    pub selected_voice_name: String,
    pub selected_voice_display_name: String,
}

static PLAYBACK_GENERATION: AtomicU64 = AtomicU64::new(0);
const SYNTHESIS_QUEUE_DEPTH: usize = 2;
const STREAM_POLL_INTERVAL_MS: u64 = 50;

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

    let model_path = absolute_install_path(snapshot, &files.model_install_path);
    let aux_path = absolute_install_path(snapshot, &files.aux_install_path);
    let support_data_root = support_data_root(snapshot, &files);

    eprintln!(
        "tts.load_tts_voices: language={} engine={} model={} aux={} support_root={}",
        language_code,
        files.engine,
        model_path,
        aux_path,
        support_data_root.as_deref().unwrap_or("<none>")
    );

    let voices = catch_tts_panic(|| {
        list_voices(
            &files.engine,
            &model_path,
            &aux_path,
            support_data_root.as_deref(),
            language_code,
        )
    })?;

    eprintln!(
        "tts.load_tts_voices: language={} returned {} voice(s): {:?}",
        language_code,
        voices.len(),
        voices
            .iter()
            .map(|voice| format!("{}={}", voice.name, voice.speaker_id))
            .collect::<Vec<_>>()
    );

    let selected_voice_name = selected_voice_name
        .filter(|value| voices.iter().any(|voice| voice.name == **value))
        .map(ToOwned::to_owned)
        .unwrap_or_default();

    let selected_voice_display_name = if selected_voice_name.is_empty() || voices.len() <= 1 {
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
        let result = play_text_streaming(
            &snapshot,
            &language_code,
            &text,
            speech_speed,
            voice_name.as_deref(),
            generation,
            &ui,
        );

        if PLAYBACK_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }

        match result {
            Ok(()) => {
                if PLAYBACK_GENERATION.load(Ordering::SeqCst) == generation {
                    (ui.set_tts_state)(false, false);
                }
            }
            Err(err) => {
                eprintln!("TTS streaming failed: {err}");
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
    let _ = files;
    let data_dir = Path::new(&snapshot.base_dir).join("bin");
    data_dir
        .join("espeak-ng-data")
        .is_dir()
        .then(|| data_dir.display().to_string())
}

#[derive(Debug)]
struct QueuedAudioChunk {
    audio: PcmAudio,
    pause_after_ms: Option<i32>,
}

fn play_text_streaming(
    snapshot: &CatalogSnapshot,
    language_code: &str,
    text: &str,
    speech_speed: f32,
    voice_name: Option<&str>,
    generation: u64,
    ui: &UiCallbacks,
) -> Result<(), String> {
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

    let (tx, rx) = sync_channel::<Result<QueuedAudioChunk, String>>(SYNTHESIS_QUEUE_DEPTH);
    let producer_files_engine = files.engine.clone();
    let producer_model_path = model_path.clone();
    let producer_aux_path = aux_path.clone();
    let producer_support_data_root = support_data_root.clone();
    let producer_language_code = language_code.to_string();
    let producer_voice_name = voice_name.map(ToOwned::to_owned);

    thread::spawn(move || {
        produce_audio_chunks(
            planned_chunks,
            tx,
            generation,
            producer_files_engine,
            producer_model_path,
            producer_aux_path,
            producer_support_data_root,
            producer_language_code,
            speech_speed,
            producer_voice_name,
            speaker_id,
        );
    });

    let mut should_stop = || PLAYBACK_GENERATION.load(Ordering::SeqCst) != generation;
    let first_chunk = match recv_chunk(&rx, &mut should_stop)? {
        Some(chunk) => chunk,
        None => return Ok(()),
    };

    let playback = PulsePlaybackStream::new(first_chunk.audio.sample_rate)?;
    (ui.set_tts_state)(false, true);
    playback.write_audio(&first_chunk.audio, &mut should_stop)?;
    if let Some(pause_after_ms) = first_chunk.pause_after_ms {
        playback.write_pause_ms(pause_after_ms, &mut should_stop)?;
    }

    while let Some(chunk) = recv_chunk(&rx, &mut should_stop)? {
        playback.write_audio(&chunk.audio, &mut should_stop)?;
        if let Some(pause_after_ms) = chunk.pause_after_ms {
            playback.write_pause_ms(pause_after_ms, &mut should_stop)?;
        }
    }

    if should_stop() {
        let _ = playback.flush();
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn produce_audio_chunks(
    planned_chunks: Vec<translator::SpeechChunk>,
    tx: SyncSender<Result<QueuedAudioChunk, String>>,
    generation: u64,
    engine: String,
    model_path: String,
    aux_path: String,
    support_data_root: Option<String>,
    language_code: String,
    speech_speed: f32,
    voice_name: Option<String>,
    speaker_id: Option<i64>,
) {
    for chunk in planned_chunks {
        if PLAYBACK_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }

        let pcm = match catch_tts_panic(|| {
            synthesize_pcm(
                &engine,
                &model_path,
                &aux_path,
                support_data_root.as_deref(),
                &language_code,
                &chunk.content,
                speech_speed,
                voice_name.as_deref(),
                speaker_id,
                chunk.is_phonemes,
            )
        }) {
            Ok(pcm) => pcm,
            Err(err) => {
                let _ = tx.send(Err(err));
                return;
            }
        };

        if PLAYBACK_GENERATION.load(Ordering::SeqCst) != generation {
            return;
        }

        if tx
            .send(Ok(QueuedAudioChunk {
                audio: pcm,
                pause_after_ms: chunk.pause_after_ms,
            }))
            .is_err()
        {
            return;
        }
    }
}

fn recv_chunk<F>(
    rx: &std::sync::mpsc::Receiver<Result<QueuedAudioChunk, String>>,
    should_stop: &mut F,
) -> Result<Option<QueuedAudioChunk>, String>
where
    F: FnMut() -> bool,
{
    loop {
        if should_stop() {
            return Ok(None);
        }

        match rx.recv_timeout(Duration::from_millis(STREAM_POLL_INTERVAL_MS)) {
            Ok(Ok(chunk)) => return Ok(Some(chunk)),
            Ok(Err(err)) => return Err(err),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return Ok(None),
        }
    }
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
