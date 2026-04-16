use qmetaobject::QString;

use crate::IoEvent;
use crate::catalog_state::format_size;
use crate::model::FeatureKind;

use super::{AppBridge, ManageTtsVoicePackListItem, TtsVoiceListItem};

impl AppBridge {
    pub(crate) fn set_tts_state_value(&mut self, loading: bool, playing: bool) {
        if self.tts_loading != loading {
            self.tts_loading = loading;
            self.tts_loading_changed();
        }
        if self.tts_playing != playing {
            self.tts_playing = playing;
            self.tts_playing_changed();
        }
    }

    pub(crate) fn set_tts_voices_value(
        &mut self,
        _available: bool,
        items: Vec<TtsVoiceListItem>,
        selected_name: String,
        selected_display_name: String,
    ) {
        let item_count = items.len() as i32;
        self.tts_voice_options_model.borrow_mut().reset_data(items);

        if self.tts_voice_option_count != item_count {
            self.tts_voice_option_count = item_count;
            self.tts_voice_option_count_changed();
        }

        let selected_name = QString::from(selected_name);
        if self.tts_selected_voice_name != selected_name {
            self.tts_selected_voice_name = selected_name;
            self.tts_selected_voice_name_changed();
        }

        let selected_display_name = QString::from(selected_display_name);
        if self.tts_selected_voice_display_name != selected_display_name {
            self.tts_selected_voice_display_name = selected_display_name;
            self.tts_selected_voice_display_name_changed();
        }
    }

    pub(crate) fn set_manage_tts_picker_open_value(&mut self, value: bool) {
        if self.manage_tts_picker_open != value {
            self.manage_tts_picker_open = value;
            self.manage_tts_picker_open_changed();
        }

        if !value {
            self.manage_tts_picker_language_code.clear();
        }
    }

    pub(crate) fn set_manage_tts_picker_language_name_value(&mut self, value: String) {
        let value = QString::from(value);
        if self.manage_tts_picker_language_name != value {
            self.manage_tts_picker_language_name = value;
            self.manage_tts_picker_language_name_changed();
        }
    }

    pub(crate) fn toggle_speak_output_impl(&mut self) {
        if self.tts_loading || self.tts_playing {
            eprintln!("tts.ui: speaker pressed while active; stopping playback");
            self.stop_tts();
            return;
        }

        let text = self.output_text.to_string();
        if text.trim().is_empty() || !self.tts_available {
            eprintln!(
                "tts.ui: speaker pressed but ignored text_empty={} tts_available={}",
                text.trim().is_empty(),
                self.tts_available
            );
            return;
        }

        let voice_name = self
            .tts_voice_overrides
            .get(&self.target_language_code)
            .cloned()
            .unwrap_or_default();
        eprintln!(
            "tts.ui: speaker pressed target_language={} chars={} speed={} voice='{}'",
            self.target_language_code,
            text.chars().count(),
            self.tts_playback_speed.clamp(0.5, 2.0),
            if voice_name.is_empty() {
                "Default"
            } else {
                &voice_name
            }
        );

        self.send_io(IoEvent::SpeakRequest {
            language_code: self.target_language_code.clone(),
            text,
            speech_speed: self.tts_playback_speed.clamp(0.5, 2.0),
            voice_name,
        });
    }

    pub(crate) fn prepare_tts_options_impl(&mut self) {
        if self.tts_available {
            eprintln!(
                "tts.ui: opening voice options target_language={}",
                self.target_language_code
            );
            self.refresh_tts_voices();
        }
    }

    pub(crate) fn set_tts_playback_speed_impl(&mut self, value: f32) {
        let quantized = ((value.clamp(0.5, 2.0) * 10.0).round() / 10.0).clamp(0.5, 2.0);
        if (self.tts_playback_speed - quantized).abs() > f32::EPSILON {
            self.tts_playback_speed = quantized;
            self.tts_playback_speed_changed();
            self.persist_settings();
        }
    }

    pub(crate) fn set_tts_voice_name_impl(&mut self, value: String) {
        if value.is_empty() {
            self.tts_voice_overrides.remove(&self.target_language_code);
        } else {
            self.tts_voice_overrides
                .insert(self.target_language_code.clone(), value.clone());
        }
        self.persist_settings();
        self.apply_tts_voice_selection_preview(value.as_str());
    }

    pub(crate) fn open_tts_download_picker_impl(&mut self, code: String) {
        let Some(language) = self.find_language_by_code(&code).cloned() else {
            return;
        };

        let items = language
            .tts_voice_picker_regions
            .into_iter()
            .flat_map(|region| {
                let region_display_name = if region.display_name.is_empty() {
                    region.code
                } else {
                    region.display_name
                };

                region.voices.into_iter().map(move |voice| {
                    let quality_text = voice
                        .quality
                        .clone()
                        .unwrap_or_else(|| "Default quality".to_string());
                    ManageTtsVoicePackListItem {
                        pack_id: voice.pack_id.into(),
                        region_display_name: region_display_name.clone().into(),
                        voice_display_name: voice.display_name.into(),
                        quality_text: quality_text.clone().into(),
                        size_text: format!("{}, {}", format_size(voice.size_bytes), quality_text)
                            .into(),
                        installed: voice.installed,
                    }
                })
            })
            .collect::<Vec<_>>();

        if items.is_empty() {
            self.send_feature_request(code, FeatureKind::Tts, true, None);
            return;
        }

        if items.len() == 1 {
            self.send_feature_request(
                code,
                FeatureKind::Tts,
                true,
                Some(items[0].pack_id.to_string()),
            );
            return;
        }

        self.manage_tts_picker_model.borrow_mut().reset_data(items);
        self.manage_tts_picker_language_code = language.code;
        self.set_manage_tts_picker_language_name_value(language.name);
        self.set_manage_tts_picker_open_value(true);
    }

    pub(crate) fn refresh_tts_voices(&mut self) {
        let selected_voice_name = self
            .tts_voice_overrides
            .get(&self.target_language_code)
            .cloned()
            .unwrap_or_default();

        self.send_io(IoEvent::RefreshTtsVoices {
            language_code: self.target_language_code.clone(),
            selected_voice_name,
        });
    }

    pub(crate) fn eager_load_tts_destination(&mut self) {
        if !self.tts_available || self.target_language_code.is_empty() {
            self.tts_prewarmed_language_code.clear();
            return;
        }

        if self.tts_prewarmed_language_code == self.target_language_code {
            return;
        }

        eprintln!(
            "tts.ui: eager loading destination target_language={}",
            self.target_language_code
        );
        self.tts_prewarmed_language_code = self.target_language_code.clone();
        self.send_io(IoEvent::WarmTtsModel {
            language_code: self.target_language_code.clone(),
        });
    }

    pub(crate) fn refresh_tts_availability(&mut self) {
        let available = self
            .find_language_by_code(&self.target_language_code)
            .map(|language| language.tts_installed)
            .unwrap_or(false);

        if self.tts_available != available {
            self.tts_available = available;
            self.tts_available_changed();
        }

        if !available {
            self.tts_prewarmed_language_code.clear();
        } else {
            self.eager_load_tts_destination();
        }
    }

    pub(crate) fn reset_tts_voice_selection_state(&mut self) {
        self.tts_voice_options_model
            .borrow_mut()
            .reset_data(Vec::new());
        if self.tts_voice_option_count != 0 {
            self.tts_voice_option_count = 0;
            self.tts_voice_option_count_changed();
        }

        let selected_name = QString::from("");
        if self.tts_selected_voice_name != selected_name {
            self.tts_selected_voice_name = selected_name;
            self.tts_selected_voice_name_changed();
        }

        let selected_display_name = QString::from("Default");
        if self.tts_selected_voice_display_name != selected_display_name {
            self.tts_selected_voice_display_name = selected_display_name;
            self.tts_selected_voice_display_name_changed();
        }
    }

    pub(crate) fn apply_tts_voice_selection_preview(&mut self, selected_voice_name: &str) {
        let display_name = if selected_voice_name.is_empty() {
            "Default".to_string()
        } else {
            self.tts_voice_options_model
                .borrow()
                .iter()
                .find(|item| item.name.to_string() == selected_voice_name)
                .map(|item| item.display_name.to_string())
                .unwrap_or_else(|| "Default".to_string())
        };

        let selected_name = QString::from(selected_voice_name);
        if self.tts_selected_voice_name != selected_name {
            self.tts_selected_voice_name = selected_name;
            self.tts_selected_voice_name_changed();
        }

        let selected_display_name = QString::from(display_name);
        if self.tts_selected_voice_display_name != selected_display_name {
            self.tts_selected_voice_display_name = selected_display_name;
            self.tts_selected_voice_display_name_changed();
        }
    }

    pub(crate) fn stop_tts(&mut self) {
        self.send_io(IoEvent::StopTts);
        self.set_tts_state_value(false, false);
    }
}
