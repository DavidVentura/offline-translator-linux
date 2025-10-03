use std::collections::HashMap;

use bergamot_sys::{BlockingService, TranslationModel};

pub struct Translator {
    data_path: String,
    service: BlockingService,
    languages: HashMap<String, TranslationModel>,
}

impl Translator {
    pub fn new(data_path: String) -> Translator {
        let service = BlockingService::new(256);

        Translator {
            data_path,
            service,
            languages: HashMap::new(),
        }
    }

    pub fn load_language_pair(&mut self, from_lang: &str, to_lang: &str) -> Result<(), String> {
        let key = from_lang.to_string() + to_lang;
        if self.languages.contains_key(&key) {
            return Ok(());
        }

        let data_path = &self.data_path;
        let model_fname = format!("model.{from_lang}{to_lang}.intgemm.alphas.bin");
        let src_vocab = format!("vocab.{from_lang}{to_lang}.spm");
        let tgt_vocab = format!("vocab.{from_lang}{to_lang}.spm"); // TODO
        let config = format!(
            r#"
models:
  - {data_path}/{model_fname}
vocabs:
  - {data_path}/{src_vocab}
  - {data_path}/{tgt_vocab}
beam-size: 1
normalize: 1.0
word-penalty: 0
max-length-break: 128
mini-batch-words: 1024
max-length-factor: 2.0
skip-cost: true
cpu-threads: 1
quiet: true
quiet-translation: true
gemm-precision: int8shiftAlphaAll
alignment: soft"#
        );
        let model = TranslationModel::from_config(&config)?;
        self.languages.insert(key, model);
        Ok(())
    }
    pub fn translate(
        &self,
        from_lang: &str,
        to_lang: &str,
        texts: &[&str],
    ) -> Result<Vec<String>, String> {
        let key = from_lang.to_string() + to_lang;
        // TODO: pivoting
        if let Some(model) = self.languages.get(&key) {
            Ok(self.service.translate(model, texts))
        } else {
            Err(format!("Language {key} not loaded"))
        }
    }
}
