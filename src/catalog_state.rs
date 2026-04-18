use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::Path;

use translator::{
    CatalogSnapshot, DeletePlan, DictionaryCode, FsPackInstallChecker, LanguageCatalog,
    LanguageCode, build_catalog_snapshot, language_rows_in_snapshot, parse_and_validate_catalog,
    plan_dictionary_download, plan_language_download, plan_tts_download,
};

use crate::data::INDEX_JSON;
use crate::model::{Direction, Language, TtsVoicePackOption, TtsVoicePickerRegion};

pub fn bundled_catalog() -> LanguageCatalog {
    let mut decoder = flate2::read::GzDecoder::new(INDEX_JSON);
    let mut json = String::new();
    decoder
        .read_to_string(&mut json)
        .expect("bundled index should decompress");

    let catalog = parse_and_validate_catalog(&json).expect("bundled index should parse");
    eprintln!(
        "catalog loaded: format={} languages={}",
        catalog.format_version,
        catalog.language_list().len()
    );
    catalog
}

pub fn build_snapshot(catalog: &LanguageCatalog, base_dir: &str) -> CatalogSnapshot {
    let checker = FsPackInstallChecker::new(base_dir);
    let snapshot = build_catalog_snapshot(catalog.clone(), base_dir.to_string(), &checker);
    eprintln!(
        "snapshot built: base_dir={} languages={} statuses={}",
        base_dir,
        snapshot.catalog.language_list().len(),
        snapshot.pack_statuses.len()
    );
    snapshot
}

pub fn languages_from_snapshot(snapshot: &CatalogSnapshot) -> Vec<Language> {
    let catalog = &snapshot.catalog;

    let mut languages = language_rows_in_snapshot(snapshot)
        .into_iter()
        .map(|row| {
            let language = row.language;
            let code = language.code.clone();
            let language_code = LanguageCode::from(code.as_str());

            let core_size_bytes = catalog.translation_size_bytes_for_language(&language_code);
            let dictionary_size_bytes = catalog
                .dictionary_info(&DictionaryCode::from(language.dictionary_code.clone()))
                .map(|info| info.size)
                .unwrap_or(0);
            let tts_size_bytes = catalog.tts_size_bytes_for_language(&language_code);

            let core_installed = core_size_bytes > 0
                && plan_language_download(snapshot, &language_code)
                    .tasks
                    .is_empty();
            let dictionary_installed = dictionary_size_bytes > 0
                && plan_dictionary_download(snapshot, &language_code)
                    .is_some_and(|plan| plan.tasks.is_empty());

            let direction = match (
                row.availability.has_to_english || code == "en",
                row.availability.has_from_english || code == "en",
            ) {
                (true, true) => Direction::Both,
                (true, false) => Direction::FromOnly,
                (false, true) => Direction::ToOnly,
                (false, false) => {
                    if code == "en" {
                        Direction::Both
                    } else {
                        Direction::FromOnly
                    }
                }
            };

            Language {
                code,
                name: language.display_name.clone(),
                script: language.script.clone(),
                dictionary_code: language.dictionary_code.clone(),
                direction,
                built_in: language.is_english(),
                core_size_bytes,
                core_installed,
                core_progress: 0.0,
                dictionary_size_bytes,
                dictionary_installed,
                dictionary_progress: 0.0,
                tts_size_bytes,
                tts_installed: row.availability.tts_files,
                tts_progress: 0.0,
                tts_voice_picker_regions: catalog
                    .tts_voice_picker_regions(&language_code)
                    .into_iter()
                    .map(|region| TtsVoicePickerRegion {
                        code: region.code,
                        display_name: region.display_name,
                        voices: region
                            .voices
                            .into_iter()
                            .map(|voice| TtsVoicePackOption {
                                installed: plan_tts_download(
                                    snapshot,
                                    &language_code,
                                    Some(voice.pack_id.as_str()),
                                )
                                .is_some_and(|plan| plan.tasks.is_empty()),
                                pack_id: voice.pack_id,
                                display_name: voice.display_name,
                                quality: voice.quality,
                                size_bytes: voice.size_bytes,
                            })
                            .collect(),
                    })
                    .collect(),
            }
        })
        .collect::<Vec<_>>();

    languages.sort_by(|left, right| left.name.cmp(&right.name));
    eprintln!("languages_from_snapshot: {} rows", languages.len());
    languages
}

pub fn format_size(size_bytes: u64) -> String {
    const ONE_KB: u64 = 1024;
    const ONE_MB: u64 = 1024 * 1024;

    match size_bytes {
        0..ONE_KB => "<1 KB".to_string(),
        ONE_KB..ONE_MB => format!("{} KB", size_bytes / ONE_KB),
        _ => format!("{} MB", size_bytes / ONE_MB),
    }
}

pub fn total_size(language: &Language) -> u64 {
    language.core_size_bytes + language.dictionary_size_bytes + language.tts_size_bytes
}

pub fn remove_delete_plan(base_dir: &str, delete_plan: &DeletePlan) {
    let mut directories = delete_plan
        .directory_paths
        .iter()
        .map(|path| Path::new(base_dir).join(path))
        .collect::<Vec<_>>();
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));

    let mut files = delete_plan
        .file_paths
        .iter()
        .map(|path| Path::new(base_dir).join(path))
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();

    for file in files {
        let _ = fs::remove_file(file);
    }

    let mut seen = HashSet::new();
    for directory in directories {
        if seen.insert(directory.clone()) {
            let _ = fs::remove_dir_all(directory);
        }
    }
}
