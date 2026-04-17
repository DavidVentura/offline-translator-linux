use qmetaobject::QString;
use translator::lookup_dictionary_for_code;
use translator::tarkka::WordWithTaggedEntries;

use super::{AppBridge, types::DictionaryPopupRowItem};

impl AppBridge {
    pub(crate) fn lookup_dictionary_for_language(&mut self, word: &str, language_code: &str) {
        eprintln!(
            "dictionary long-press lookup requested: {} ({})",
            word, language_code
        );
        let trimmed = word.trim();
        if trimmed.is_empty() {
            return;
        }

        let Some(language) = self.find_language_by_code(language_code) else {
            return;
        };
        if !language.dictionary_installed || language.dictionary_code.is_empty() {
            self.show_toast_impl(format!("No {} dictionary installed", language.name));
            return;
        }

        let lookup_result =
            lookup_dictionary_for_code(&self.data_dir, &language.dictionary_code, trimmed);

        match lookup_result {
            Ok(Some(word_data)) => {
                self.dictionary_popup_lookup_language_code = language_code.to_string();
                self.dictionary_popup_data = Some(word_data);
                self.dictionary_popup_selected_entry_index = 0;
                self.dictionary_popup_selected_entry_index_changed();
                self.rebuild_dictionary_popup_state();
                self.set_dictionary_popup_open_value(true);
            }
            Ok(None) => {
                eprintln!(
                    "dictionary lookup: '{}' not found for {}",
                    trimmed, language_code
                );
                self.show_toast_impl(format!(
                    "‘{}’ not found in {} dictionary",
                    trimmed, language.name
                ));
            }
            Err(err) => {
                eprintln!(
                    "dictionary lookup failed for '{}' ({}): {}",
                    trimmed, language_code, err
                );
                self.show_toast_impl("Dictionary lookup failed".to_string());
            }
        }
    }

    pub(crate) fn close_dictionary_popup_impl(&mut self) {
        self.set_dictionary_popup_open_value(false);
    }

    pub(crate) fn select_dictionary_popup_entry_impl(&mut self, index: i32) {
        let Some(word) = &self.dictionary_popup_data else {
            return;
        };
        if index < 0 || (index as usize) >= word.entries.len() {
            return;
        }
        if self.dictionary_popup_selected_entry_index != index {
            self.dictionary_popup_selected_entry_index = index;
            self.dictionary_popup_selected_entry_index_changed();
            self.rebuild_dictionary_popup_rows();
        }
    }

    pub(crate) fn rebuild_dictionary_popup_state(&mut self) {
        let Some(word) = self.dictionary_popup_data.as_ref() else {
            self.set_dictionary_popup_open_value(false);
            self.set_dictionary_popup_word_value(String::new());
            self.set_dictionary_popup_subtitle_value(String::new());
            self.set_dictionary_popup_primary_label_value(String::new());
            self.set_dictionary_popup_secondary_label_value(String::new());
            self.set_dictionary_popup_has_secondary_value(false);
            self.dictionary_popup_rows_model
                .borrow_mut()
                .reset_data(Vec::new());
            return;
        };

        let popup_word = word.word.clone();
        let subtitle = dictionary_subtitle(word);
        let has_secondary = word.entries.len() > 1;
        let primary_label = self.dictionary_popup_lookup_language_code.clone();
        let secondary_label = "en".to_string();

        self.set_dictionary_popup_word_value(popup_word);
        self.set_dictionary_popup_subtitle_value(subtitle);
        self.set_dictionary_popup_primary_label_value(primary_label);
        self.set_dictionary_popup_secondary_label_value(secondary_label);
        self.set_dictionary_popup_has_secondary_value(has_secondary);
        self.rebuild_dictionary_popup_rows();
    }

    fn rebuild_dictionary_popup_rows(&mut self) {
        let Some(word) = &self.dictionary_popup_data else {
            self.dictionary_popup_rows_model
                .borrow_mut()
                .reset_data(Vec::new());
            return;
        };

        let selected_index = self.dictionary_popup_selected_entry_index.max(0) as usize;
        let entry = word
            .entries
            .get(selected_index)
            .or_else(|| word.entries.first());

        let mut rows = Vec::new();
        if let Some(entry) = entry {
            let mut last_pos = String::new();
            for sense in &entry.senses {
                let pos = sense.pos.to_string().replace('_', " ");
                if pos != last_pos {
                    rows.push(DictionaryPopupRowItem {
                        kind: QString::from("pos"),
                        text: QString::from(pos.clone()),
                    });
                    last_pos = pos;
                }
                for gloss in &sense.glosses {
                    for line in &gloss.gloss_lines {
                        rows.push(DictionaryPopupRowItem {
                            kind: QString::from("gloss"),
                            text: QString::from(line.clone()),
                        });
                    }
                }
            }
        }
        self.dictionary_popup_rows_model
            .borrow_mut()
            .reset_data(rows);
    }

    fn set_dictionary_popup_open_value(&mut self, value: bool) {
        if self.dictionary_popup_open != value {
            self.dictionary_popup_open = value;
            self.dictionary_popup_open_changed();
        }
    }

    fn set_dictionary_popup_word_value(&mut self, value: String) {
        let q = QString::from(value);
        if self.dictionary_popup_word != q {
            self.dictionary_popup_word = q;
            self.dictionary_popup_word_changed();
        }
    }

    fn set_dictionary_popup_subtitle_value(&mut self, value: String) {
        let q = QString::from(value);
        if self.dictionary_popup_subtitle != q {
            self.dictionary_popup_subtitle = q;
            self.dictionary_popup_subtitle_changed();
        }
    }

    fn set_dictionary_popup_primary_label_value(&mut self, value: String) {
        let q = QString::from(value);
        if self.dictionary_popup_primary_label != q {
            self.dictionary_popup_primary_label = q;
            self.dictionary_popup_primary_label_changed();
        }
    }

    fn set_dictionary_popup_secondary_label_value(&mut self, value: String) {
        let q = QString::from(value);
        if self.dictionary_popup_secondary_label != q {
            self.dictionary_popup_secondary_label = q;
            self.dictionary_popup_secondary_label_changed();
        }
    }

    fn set_dictionary_popup_has_secondary_value(&mut self, value: bool) {
        if self.dictionary_popup_has_secondary != value {
            self.dictionary_popup_has_secondary = value;
            self.dictionary_popup_has_secondary_changed();
        }
    }

    pub(crate) fn show_toast_impl(&mut self, message: String) {
        let q = QString::from(message);
        if self.toast_visible {
            self.toast_visible = false;
            self.toast_visible_changed();
        }
        if self.toast_message != q {
            self.toast_message = q;
            self.toast_message_changed();
        } else {
            self.toast_message_changed();
        }
        self.toast_visible = true;
        self.toast_visible_changed();
    }

    pub(crate) fn clear_toast_impl(&mut self) {
        if self.toast_visible {
            self.toast_visible = false;
            self.toast_visible_changed();
        }
    }
}

fn dictionary_subtitle(word: &WordWithTaggedEntries) -> String {
    let ipa = word
        .sounds
        .as_ref()
        .map(|value| value.replace(['[', ']'], "/"))
        .filter(|value| !value.trim().is_empty());
    let hyphenation = (!word.hyphenations.is_empty()).then(|| word.hyphenations.join("·"));

    match (ipa, hyphenation) {
        (Some(ipa), Some(hyphenation)) => format!("{}    {}", ipa, hyphenation),
        (Some(ipa), None) => ipa,
        (None, Some(hyphenation)) => hyphenation,
        (None, None) => String::new(),
    }
}
