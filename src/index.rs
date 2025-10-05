use miniserde::{Deserialize, Serialize};

use crate::Language;

#[derive(Serialize, Deserialize)]
pub struct Index {
    languages: Vec<IndexLanguage>,
}

#[derive(Serialize, Deserialize)]
pub struct IndexFile {
    size_bytes: u32,
}
#[derive(Serialize, Deserialize)]
pub struct IndexLanguage {
    code: String,
    name: String,
    url: String,
    files: Vec<IndexFile>,
    release_date: u64,
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
        let size_bytes = value.files.iter().map(|f| f.size_bytes).sum();

        Self {
            code: value.code.into(),
            name: value.name.into(),
            size: pretty_size(size_bytes).into(),
        }
    }
}
