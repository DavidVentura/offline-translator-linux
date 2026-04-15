use std::path::Path;
use std::time::Instant;

use translator::BergamotEngine;

pub struct Translator {
    data_path: String,
    engine: BergamotEngine,
}

impl Translator {
    pub fn new(data_path: String) -> Translator {
        Translator {
            data_path,
            engine: BergamotEngine::new(),
        }
    }

    pub fn load_language_pair(&mut self, from_lang: &str, to_lang: &str) -> Result<(), String> {
        let pairs = self.get_pairs(from_lang, to_lang);
        for (from, to) in pairs {
            self._load_language_pair(from, to)?;
        }
        Ok(())
    }

    fn _load_language_pair(&mut self, from_lang: &str, to_lang: &str) -> Result<(), String> {
        let key = from_lang.to_string() + to_lang;
        let start = Instant::now();
        let data_path = &self.data_path;
        let model_fname = format!("model.{from_lang}{to_lang}.intgemm.alphas.bin");
        let (src_vocab, tgt_vocab) = if vec!["zh", "ko", "ja"].contains(&to_lang) {
            (
                format!("srcvocab.{from_lang}{to_lang}.spm"),
                format!("trgvocab.{from_lang}{to_lang}.spm"),
            )
        } else {
            (
                format!("vocab.{from_lang}{to_lang}.spm"),
                format!("vocab.{from_lang}{to_lang}.spm"),
            )
        };

        let model_path = Path::new(data_path).join(&model_fname);
        let src_vocab_path = Path::new(data_path).join(&src_vocab);
        let tgt_vocab_path = Path::new(data_path).join(&tgt_vocab);

        if !model_path.exists() {
            return Err(format!("Model file not found: {}", model_path.display()));
        }
        if !src_vocab_path.exists() {
            return Err(format!(
                "Source vocab file not found: {}",
                src_vocab_path.display()
            ));
        }
        if !tgt_vocab_path.exists() {
            return Err(format!(
                "Target vocab file not found: {}",
                tgt_vocab_path.display()
            ));
        }

        let model_path_str = model_path.to_str().ok_or("Model path is not valid UTF-8")?;
        let src_vocab_path_str = src_vocab_path
            .to_str()
            .ok_or("Source vocab path is not valid UTF-8")?;
        let tgt_vocab_path_str = tgt_vocab_path
            .to_str()
            .ok_or("Target vocab path is not valid UTF-8")?;

        let config = format!(
            r#"
models:
  - {model_path_str}
vocabs:
  - {src_vocab_path_str}
  - {tgt_vocab_path_str}
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
        println!("load {key} took {:?}", start.elapsed());
        self.engine.load_model_into_cache(&config, &key)
    }

    pub fn translate(
        &self,
        from_lang: &str,
        to_lang: &str,
        texts: &[&str],
    ) -> Result<Vec<String>, String> {
        let inputs = texts
            .iter()
            .map(|text| (*text).to_string())
            .collect::<Vec<_>>();
        let keys = self
            .get_pairs(from_lang, to_lang)
            .iter()
            .map(|(from, to)| format!("{from}{to}"))
            .collect::<Vec<_>>();

        if keys.len() == 2 {
            self.engine.pivot_multiple(&keys[0], &keys[1], &inputs)
        } else {
            assert_eq!(keys.len(), 1);
            self.engine.translate_multiple(&inputs, &keys[0])
        }
    }

    fn get_pairs<'a>(&self, from_lang: &'a str, to_lang: &'a str) -> Vec<(&'a str, &'a str)> {
        if from_lang != "en" && to_lang != "en" {
            vec![(from_lang, "en"), ("en", to_lang)]
        } else {
            vec![(from_lang, to_lang)]
        }
    }
}
