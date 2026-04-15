use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use translator::{
    CatalogSnapshot, DeletePlan, DownloadPlan, LanguageCatalog, PackInstallChecker, PackResolver,
    build_catalog_snapshot, language_rows_in_snapshot, parse_and_validate_catalog,
    plan_delete_dictionary_in_snapshot, plan_delete_language_in_snapshot,
    plan_delete_tts_in_snapshot, plan_dictionary_download, plan_dictionary_download_in_snapshot,
    plan_language_download, plan_language_download_in_snapshot, plan_tts_download,
    plan_tts_download_in_snapshot,
};

use crate::model::{Direction, FeatureKind, Language, TtsVoicePackOption, TtsVoicePickerRegion};

const BUNDLED_CATALOG_JSON: &str = include_str!("../data/catalog.json");

pub struct FsInstallChecker {
    base_dir: PathBuf,
}

struct EmptyInstallChecker;

impl FsInstallChecker {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    fn resolve(&self, relative_path: &str) -> PathBuf {
        self.base_dir.join(relative_path)
    }
}

impl PackInstallChecker for FsInstallChecker {
    fn file_exists(&self, install_path: &str) -> bool {
        self.resolve(install_path).exists()
    }

    fn install_marker_exists(&self, marker_path: &str, expected_version: i32) -> bool {
        let marker_file = self.resolve(marker_path);
        if !marker_file.exists() {
            return false;
        }

        let Ok(contents) = fs::read_to_string(marker_file) else {
            return false;
        };

        contents.contains(&format!("\"version\":{expected_version}"))
            || contents.contains(&format!("\"version\": {expected_version}"))
    }
}

impl PackInstallChecker for EmptyInstallChecker {
    fn file_exists(&self, _install_path: &str) -> bool {
        false
    }

    fn install_marker_exists(&self, _marker_path: &str, _expected_version: i32) -> bool {
        false
    }
}

pub fn bundled_catalog() -> LanguageCatalog {
    let catalog =
        parse_and_validate_catalog(BUNDLED_CATALOG_JSON).expect("bundled catalog should parse");
    eprintln!(
        "catalog loaded: format={} languages={}",
        catalog.format_version,
        catalog.language_list().len()
    );
    catalog
}

pub fn build_snapshot(catalog: &LanguageCatalog, base_dir: &str) -> CatalogSnapshot {
    let checker = FsInstallChecker::new(base_dir);
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
    let empty_checker = EmptyInstallChecker;
    let mut full_resolver = PackResolver::new(catalog, &empty_checker);

    let mut languages = language_rows_in_snapshot(snapshot)
        .into_iter()
        .map(|row| {
            let language = row.language;
            let code = language.code.clone();
            let core_size_bytes =
                plan_language_download(catalog, &code, &mut full_resolver).total_size;
            let dictionary_size_bytes =
                plan_dictionary_download(catalog, &code, &mut full_resolver)
                    .map(|plan| plan.total_size)
                    .unwrap_or(0);
            let tts_size_bytes = plan_tts_download(catalog, &code, None, &mut full_resolver)
                .map(|plan| plan.total_size)
                .unwrap_or(0);

            let core_installed = core_size_bytes > 0
                && plan_language_download_in_snapshot(snapshot, &code)
                    .tasks
                    .is_empty();
            let dictionary_installed = dictionary_size_bytes > 0
                && plan_dictionary_download_in_snapshot(snapshot, &code)
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
                tts_voice_picker_regions: snapshot
                    .catalog
                    .tts_voice_picker_regions(&language.code)
                    .into_iter()
                    .map(|region| TtsVoicePickerRegion {
                        code: region.code,
                        display_name: region.display_name,
                        voices: region
                            .voices
                            .into_iter()
                            .map(|voice| TtsVoicePackOption {
                                installed: plan_tts_download_in_snapshot(
                                    snapshot,
                                    &language.code,
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

pub fn download_plan_for_feature(
    snapshot: &CatalogSnapshot,
    language_code: &str,
    feature: FeatureKind,
    selected_tts_pack_id: Option<&str>,
) -> Option<DownloadPlan> {
    match feature {
        FeatureKind::Core => Some(plan_language_download_in_snapshot(snapshot, language_code)),
        FeatureKind::Dictionary => plan_dictionary_download_in_snapshot(snapshot, language_code),
        FeatureKind::Tts => {
            plan_tts_download_in_snapshot(snapshot, language_code, selected_tts_pack_id)
        }
    }
}

pub fn delete_plan_for_feature(
    snapshot: &CatalogSnapshot,
    language_code: &str,
    feature: FeatureKind,
) -> DeletePlan {
    match feature {
        FeatureKind::Core => plan_delete_language_in_snapshot(snapshot, language_code),
        FeatureKind::Dictionary => plan_delete_dictionary_in_snapshot(snapshot, language_code),
        FeatureKind::Tts => plan_delete_tts_in_snapshot(snapshot, language_code),
    }
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
