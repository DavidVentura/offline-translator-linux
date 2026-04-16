use qmetaobject::QString;
use std::path::Path;
use translator::transliterate_with_policy_for_language;

use super::AppBridge;

impl AppBridge {
    pub(crate) fn set_input_text_value(&mut self, text: String) {
        let text = QString::from(text);
        if self.input_text != text {
            self.input_text = text;
            self.input_text_changed();
        }
        self.refresh_input_transliteration();
    }

    pub(crate) fn set_output_text_value(&mut self, text: String) {
        let text = QString::from(text);
        if self.output_text != text {
            self.output_text = text;
            self.output_text_changed();
        }
        self.refresh_output_transliteration();
    }

    pub(crate) fn set_input_transliteration_value(&mut self, text: String) {
        let text = QString::from(text);
        if self.input_transliteration != text {
            self.input_transliteration = text;
            self.input_transliteration_changed();
        }
    }

    pub(crate) fn set_output_transliteration_value(&mut self, text: String) {
        let text = QString::from(text);
        if self.output_transliteration != text {
            self.output_transliteration = text;
            self.output_transliteration_changed();
        }
    }

    pub(crate) fn refresh_input_transliteration(&mut self) {
        let transliteration = if self.show_transliteration_input {
            self.compute_transliteration(&self.input_text.to_string(), &self.source_language_code)
        } else {
            String::new()
        };
        self.set_input_transliteration_value(transliteration);
    }

    pub(crate) fn refresh_output_transliteration(&mut self) {
        let transliteration = if self.show_transliteration_output {
            self.compute_transliteration(&self.output_text.to_string(), &self.target_language_code)
        } else {
            String::new()
        };
        self.set_output_transliteration_value(transliteration);
    }

    pub(crate) fn compute_transliteration(&self, text: &str, language_code: &str) -> String {
        if text.trim().is_empty() || !text.chars().any(|ch| !ch.is_ascii()) {
            return String::new();
        }

        let Some(language) = self.find_language_by_code(language_code) else {
            return String::new();
        };

        let japanese_dict_path = if language.code == "ja" {
            let candidate = Path::new(&self.data_dir).join("bin/mucab.bin");
            candidate
                .exists()
                .then(|| candidate.to_string_lossy().into_owned())
        } else {
            None
        };

        transliterate_with_policy_for_language(
            text,
            &language.code,
            &language.script,
            "Latn",
            japanese_dict_path.as_deref(),
            true,
        )
        .unwrap_or_default()
    }
}
