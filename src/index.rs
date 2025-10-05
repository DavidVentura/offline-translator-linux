use miniserde::{Deserialize, Serialize};

use crate::Language;

#[derive(Serialize, Deserialize)]
pub struct Index {
    languages: Vec<IndexLanguage>,
}

#[derive(Serialize, Deserialize)]
pub struct IndexFile {
    name: String,
    size_bytes: u32,
}
#[derive(Serialize, Deserialize)]
pub struct ModelFiles {
    model: IndexFile,
    lex: IndexFile,
    src_vocab: IndexFile,
    tgt_vocab: IndexFile,
}
#[derive(Serialize, Deserialize)]
pub struct PairData {
    files: ModelFiles,
    release_date: u64,
}
#[derive(Serialize, Deserialize)]
pub struct IndexLanguage {
    code: String,
    name: String,
    script: String, // TODO: Enum?
    from: Option<PairData>,
    to: Option<PairData>,
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

impl From<IndexLanguage> for Language {
    fn from(value: IndexLanguage) -> Self {
        let size_bytes = value
            .from
            .iter()
            .chain(value.to.iter())
            .map(|pd| {
                pd.files.model.size_bytes
                    + pd.files.lex.size_bytes
                    + pd.files.src_vocab.size_bytes
                    + pd.files.tgt_vocab.size_bytes
            })
            .sum();

        Self {
            code: value.code.into(),
            name: value.name.into(),
            size: pretty_size(size_bytes).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_index() {
        let json = crate::data::INDEX_JSON;
        let _index: Index = miniserde::json::from_str(json).expect("Failed to deserialize Index");
    }
}
