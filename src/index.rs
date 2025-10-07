use miniserde::{Deserialize, Serialize};

use crate::{Direction, Language};

#[derive(Serialize, Deserialize)]
pub struct Index {
    pub languages: Vec<IndexLanguage>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct IndexFile {
    pub name: String,
    pub size_bytes: u32,
    pub release_date: u64,
    pub url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PairData {
    pub model: IndexFile,
    pub lex: IndexFile,
    pub src_vocab: IndexFile,
    pub tgt_vocab: IndexFile,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct IndexLanguage {
    pub code: String,
    pub name: String,
    pub script: String, // TODO: Enum?
    pub from: Option<PairData>,
    pub to: Option<PairData>,
    pub extra_files: Vec<IndexFile>,
    // TODO extra files
}

const ONE_MB: u32 = 1024 * 1024;
const ONE_KB: u32 = 1024;

fn pretty_size(size_bytes: u32) -> String {
    match size_bytes {
        0..ONE_KB => "<1KiB".to_string(),
        ONE_KB..ONE_MB => format!("{}KiB", size_bytes / ONE_KB),
        ONE_MB.. => format!("{}MiB", size_bytes / ONE_MB),
    }
}

impl IndexLanguage {
    pub fn files(&self) -> Vec<IndexFile> {
        let mut ret = Vec::new();
        if let Some(from) = &self.from {
            ret.push(from.model.clone());
            ret.push(from.lex.clone());
            ret.push(from.src_vocab.clone());
            ret.push(from.tgt_vocab.clone());
        }
        if let Some(to) = &self.to {
            ret.push(to.model.clone());
            ret.push(to.lex.clone());
            ret.push(to.src_vocab.clone());
            ret.push(to.tgt_vocab.clone());
        }
        ret.append(&mut self.extra_files.clone());
        ret
    }
}
impl From<&IndexLanguage> for Language {
    fn from(value: &IndexLanguage) -> Self {
        let size_bytes = value.files().iter().map(|f| f.size_bytes).sum();

        Self {
            code: value.code.clone().into(),
            name: value.name.clone().into(),
            size: pretty_size(size_bytes).into(),
            direction: match (&value.from, &value.to) {
                (None, None) => {
                    assert!(value.code == "en");
                    Direction::Both
                }
                (Some(_), None) => Direction::FromOnly,
                (None, Some(_)) => Direction::ToOnly,
                (Some(_), Some(_)) => Direction::Both,
            },
            installed: false,
            download_progress: 0f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_index() {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let json_gz = crate::data::INDEX_JSON;
        let mut decoder = GzDecoder::new(json_gz);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .expect("Failed to decompress gzip data");

        let _index: Index = miniserde::json::from_str(&json).expect("Failed to deserialize Index");
    }
}
