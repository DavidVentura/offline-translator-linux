use qmetaobject::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::IoEvent;
use crate::catalog_state::{format_size, total_size};
use crate::model::{Direction, FeatureKind, Language, Screen};
use crate::settings::{Settings, save_settings};

#[derive(Clone, Default, SimpleListItem)]
pub struct LanguageListItem {
    pub code: QString,
    pub name: QString,
    pub size: QString,
    pub installed: bool,
    pub download_progress: f32,
    pub built_in: bool,
}

#[derive(Clone, Default, SimpleListItem)]
pub struct ManageLanguageListItem {
    pub code: QString,
    pub name: QString,
    pub total_size: QString,
    pub built_in: bool,
    pub expanded: bool,
    pub core_available: bool,
    pub core_installed: bool,
    pub core_size: QString,
    pub core_progress: f32,
    pub dictionary_available: bool,
    pub dictionary_installed: bool,
    pub dictionary_size: QString,
    pub dictionary_progress: f32,
    pub tts_available: bool,
    pub tts_installed: bool,
    pub tts_size: QString,
    pub tts_progress: f32,
}

#[derive(Clone, Default, SimpleListItem)]
pub struct ImageOverlayListItem {
    pub block_x: f32,
    pub block_y: f32,
    pub block_width: f32,
    pub block_height: f32,
    pub translated_text: QString,
    pub background_color: QString,
    pub foreground_color: QString,
}

#[derive(QObject, Default)]
pub struct AppBridge {
    base: qt_base_class!(trait QObject),

    pub current_screen: qt_property!(i32; NOTIFY current_screen_changed),
    pub current_screen_changed: qt_signal!(),

    pub disable_auto_detect: qt_property!(bool; NOTIFY disable_auto_detect_changed),
    pub disable_auto_detect_changed: qt_signal!(),

    pub has_languages: qt_property!(bool; NOTIFY has_languages_changed),
    pub has_languages_changed: qt_signal!(),

    pub input_text: qt_property!(QString; NOTIFY input_text_changed),
    pub input_text_changed: qt_signal!(),

    pub output_text: qt_property!(QString; NOTIFY output_text_changed),
    pub output_text_changed: qt_signal!(),

    pub image_mode: qt_property!(bool; NOTIFY image_mode_changed),
    pub image_mode_changed: qt_signal!(),

    pub selected_image_url: qt_property!(QString; NOTIFY selected_image_url_changed),
    pub selected_image_url_changed: qt_signal!(),

    pub processed_image_width: qt_property!(f32; NOTIFY processed_image_width_changed),
    pub processed_image_width_changed: qt_signal!(),

    pub processed_image_height: qt_property!(f32; NOTIFY processed_image_height_changed),
    pub processed_image_height_changed: qt_signal!(),

    pub source_language_name: qt_property!(QString; NOTIFY source_language_name_changed),
    pub source_language_name_changed: qt_signal!(),

    pub target_language_name: qt_property!(QString; NOTIFY target_language_name_changed),
    pub target_language_name_changed: qt_signal!(),

    pub installed_from_language_names: qt_property!(QStringList; NOTIFY installed_from_language_names_changed),
    pub installed_from_language_names_changed: qt_signal!(),

    pub installed_to_language_names: qt_property!(QStringList; NOTIFY installed_to_language_names_changed),
    pub installed_to_language_names_changed: qt_signal!(),

    pub swap_enabled: qt_property!(bool; NOTIFY swap_enabled_changed),
    pub swap_enabled_changed: qt_signal!(),

    pub detected_language_name: qt_property!(QString; NOTIFY detected_language_name_changed),
    pub detected_language_name_changed: qt_signal!(),

    pub detected_language_installed: qt_property!(bool; NOTIFY detected_language_installed_changed),
    pub detected_language_installed_changed: qt_signal!(),

    pub detected_language_progress: qt_property!(f32; NOTIFY detected_language_progress_changed),
    pub detected_language_progress_changed: qt_signal!(),

    pub show_missing_card: qt_property!(bool; NOTIFY show_missing_card_changed),
    pub show_missing_card_changed: qt_signal!(),

    pub active_tab: qt_property!(i32; NOTIFY active_tab_changed),
    pub active_tab_changed: qt_signal!(),

    pub manage_filter_text: qt_property!(QString; NOTIFY manage_filter_text_changed),
    pub manage_filter_text_changed: qt_signal!(),

    pub installed_languages_model: qt_property!(RefCell<SimpleListModel<LanguageListItem>>; CONST),
    pub available_languages_model: qt_property!(RefCell<SimpleListModel<LanguageListItem>>; CONST),
    pub manage_languages_model: qt_property!(RefCell<SimpleListModel<ManageLanguageListItem>>; CONST),
    pub image_overlay_model: qt_property!(RefCell<SimpleListModel<ImageOverlayListItem>>; CONST),

    pub desktop_mode: qt_property!(bool; CONST),

    // Settings properties
    pub font_size: qt_property!(i32; NOTIFY font_size_changed),
    pub font_size_changed: qt_signal!(),

    pub ocr_background_mode: qt_property!(QString; NOTIFY ocr_background_mode_changed),
    pub ocr_background_mode_changed: qt_signal!(),

    pub ocr_min_confidence: qt_property!(i32; NOTIFY ocr_min_confidence_changed),
    pub ocr_min_confidence_changed: qt_signal!(),

    pub ocr_max_image_size: qt_property!(i32; NOTIFY ocr_max_image_size_changed),
    pub ocr_max_image_size_changed: qt_signal!(),

    pub catalog_index_url: qt_property!(QString; NOTIFY catalog_index_url_changed),
    pub catalog_index_url_changed: qt_signal!(),

    pub disable_ocr: qt_property!(bool; NOTIFY disable_ocr_changed),
    pub disable_ocr_changed: qt_signal!(),

    pub show_transliteration_output: qt_property!(bool; NOTIFY show_transliteration_output_changed),
    pub show_transliteration_output_changed: qt_signal!(),

    pub show_transliteration_input: qt_property!(bool; NOTIFY show_transliteration_input_changed),
    pub show_transliteration_input_changed: qt_signal!(),

    pub asset_url: qt_method!(
        fn asset_url(&self, name: QString) -> QString {
            format!("file://{}/{}", self.asset_dir, name).into()
        }
    ),

    pub set_from: qt_method!(
        fn set_from(&mut self, name: QString) {
            self.set_source_language_by_name(name.to_string());
        }
    ),
    pub set_to: qt_method!(
        fn set_to(&mut self, name: QString) {
            self.set_target_language_by_name(name.to_string());
        }
    ),
    pub swap_languages: qt_method!(
        fn swap_languages(&mut self) {
            self.swap_languages_impl();
        }
    ),
    pub process_text: qt_method!(
        fn process_text(&mut self, text: QString) {
            self.process_text_impl(text.to_string());
        }
    ),
    pub download_language: qt_method!(
        fn download_language(&mut self, code: QString) {
            self.send_feature_request(code.to_string(), FeatureKind::Core, true);
        }
    ),
    pub delete_language: qt_method!(
        fn delete_language(&mut self, code: QString) {
            self.send_feature_request(code.to_string(), FeatureKind::Core, false);
        }
    ),
    pub download_feature: qt_method!(
        fn download_feature(&mut self, code: QString, feature: i32) {
            if let Some(feature) = FeatureKind::from_i32(feature) {
                self.send_feature_request(code.to_string(), feature, true);
            }
        }
    ),
    pub delete_feature: qt_method!(
        fn delete_feature(&mut self, code: QString, feature: i32) {
            if let Some(feature) = FeatureKind::from_i32(feature) {
                self.send_feature_request(code.to_string(), feature, false);
            }
        }
    ),
    pub download_all_features: qt_method!(
        fn download_all_features(&mut self, code: QString) {
            let code = code.to_string();
            if let Some(language) = self.find_language_by_code(&code).cloned() {
                if language.core_size_bytes > 0 && !language.core_installed {
                    self.send_feature_request(code.clone(), FeatureKind::Core, true);
                }
                if language.dictionary_size_bytes > 0 && !language.dictionary_installed {
                    self.send_feature_request(code.clone(), FeatureKind::Dictionary, true);
                }
                if language.tts_size_bytes > 0 && !language.tts_installed {
                    self.send_feature_request(code, FeatureKind::Tts, true);
                }
            }
        }
    ),
    pub delete_all_features: qt_method!(
        fn delete_all_features(&mut self, code: QString) {
            let code = code.to_string();
            if let Some(language) = self.find_language_by_code(&code).cloned() {
                if language.tts_size_bytes > 0 && language.tts_installed {
                    self.send_feature_request(code.clone(), FeatureKind::Tts, false);
                }
                if language.dictionary_size_bytes > 0 && language.dictionary_installed {
                    self.send_feature_request(code.clone(), FeatureKind::Dictionary, false);
                }
                if language.core_size_bytes > 0 && language.core_installed {
                    self.send_feature_request(code, FeatureKind::Core, false);
                }
            }
        }
    ),
    pub toggle_manage_language: qt_method!(
        fn toggle_manage_language(&mut self, code: QString) {
            let code = code.to_string();
            let expanded = if !self.expanded_languages.remove(&code) {
                self.expanded_languages.insert(code.clone());
                true
            } else {
                false
            };

            let visible = self.manage_filter.is_empty()
                || self
                    .find_language_by_code(&code)
                    .map(|language| {
                        language
                            .name
                            .to_lowercase()
                            .contains(self.manage_filter.as_str())
                    })
                    .unwrap_or(false);

            if visible
                && let Some(language) = self.find_language_by_code(&code).cloned()
            {
                update_manage_progress_item(
                    &mut self.manage_languages_model.borrow_mut(),
                    &language,
                    expanded,
                );
            }
        }
    ),
    pub set_manage_filter: qt_method!(
        fn set_manage_filter(&mut self, text: QString) {
            let text = text.to_string();
            let qtext = QString::from(text.clone());
            if self.manage_filter_text == qtext {
                return;
            }
            self.manage_filter_text = qtext;
            self.manage_filter_text_changed();
            self.manage_filter = text.to_lowercase();
            self.refresh_manage_model();
        }
    ),
    pub show_settings: qt_method!(
        fn show_settings(&mut self) {
            self.set_current_screen(Screen::Settings);
        }
    ),
    pub back_from_settings: qt_method!(
        fn back_from_settings(&mut self) {
            self.set_current_screen(Screen::Translation);
        }
    ),
    pub show_manage_languages: qt_method!(
        fn show_manage_languages(&mut self) {
            self.previous_screen = Screen::Settings;
            self.set_current_screen(Screen::ManageLanguages);
        }
    ),
    pub back_from_manage_languages: qt_method!(
        fn back_from_manage_languages(&mut self) {
            if self.previous_screen == Screen::Settings {
                self.set_current_screen(Screen::Settings);
            } else if self.has_languages {
                self.set_current_screen(Screen::Translation);
            } else {
                self.set_current_screen(Screen::NoLanguages);
            }
        }
    ),
    pub finish_language_setup: qt_method!(
        fn finish_language_setup(&mut self) {
            if self.has_languages {
                self.set_current_screen(Screen::Translation);
            }
        }
    ),
    pub set_disable_auto_detect_value: qt_method!(
        fn set_disable_auto_detect_value(&mut self, value: bool) {
            self.set_disable_auto_detect_impl(value);
        }
    ),
    pub set_active_tab: qt_method!(
        fn set_active_tab(&mut self, tab: i32) {
            if self.active_tab != tab {
                self.active_tab = tab;
                self.active_tab_changed();
            }
        }
    ),
    pub missing_language_action: qt_method!(
        fn missing_language_action(&mut self) {
            self.missing_language_action_impl();
        }
    ),
    pub camera_clicked: qt_method!(
        fn camera_clicked(&self) {
            println!("Camera clicked");
        }
    ),
    pub process_image_selection: qt_method!(
        fn process_image_selection(&mut self, url: QString) {
            self.process_image_selection_impl(url.to_string());
        }
    ),
    pub clear_selected_image: qt_method!(
        fn clear_selected_image(&mut self) {
            self.original_image_path.clear();
            self.set_image_mode_value(false);
            self.set_selected_image_url_value(String::new());
            self.set_image_overlay_value(Vec::new(), 0.0, 0.0);
        }
    ),

    // Settings setters
    pub set_font_size_value: qt_method!(
        fn set_font_size_value(&mut self, value: i32) {
            if self.font_size != value {
                self.font_size = value;
                self.font_size_changed();
                self.persist_settings();
            }
        }
    ),
    pub set_ocr_background_mode_value: qt_method!(
        fn set_ocr_background_mode_value(&mut self, value: QString) {
            if self.ocr_background_mode != value {
                self.ocr_background_mode = value;
                self.ocr_background_mode_changed();
                self.persist_settings();
            }
        }
    ),
    pub set_ocr_min_confidence_value: qt_method!(
        fn set_ocr_min_confidence_value(&mut self, value: i32) {
            if self.ocr_min_confidence != value {
                self.ocr_min_confidence = value;
                self.ocr_min_confidence_changed();
                self.persist_settings();
            }
        }
    ),
    pub set_ocr_max_image_size_value: qt_method!(
        fn set_ocr_max_image_size_value(&mut self, value: i32) {
            if self.ocr_max_image_size != value {
                self.ocr_max_image_size = value;
                self.ocr_max_image_size_changed();
                self.persist_settings();
            }
        }
    ),
    pub set_catalog_index_url_value: qt_method!(
        fn set_catalog_index_url_value(&mut self, value: QString) {
            if self.catalog_index_url != value {
                self.catalog_index_url = value;
                self.catalog_index_url_changed();
                self.persist_settings();
            }
        }
    ),
    pub set_disable_ocr_value: qt_method!(
        fn set_disable_ocr_value(&mut self, value: bool) {
            if self.disable_ocr != value {
                self.disable_ocr = value;
                self.disable_ocr_changed();
                self.persist_settings();
            }
        }
    ),
    pub set_show_transliteration_output_value: qt_method!(
        fn set_show_transliteration_output_value(&mut self, value: bool) {
            if self.show_transliteration_output != value {
                self.show_transliteration_output = value;
                self.show_transliteration_output_changed();
                self.persist_settings();
            }
        }
    ),
    pub set_show_transliteration_input_value: qt_method!(
        fn set_show_transliteration_input_value(&mut self, value: bool) {
            if self.show_transliteration_input != value {
                self.show_transliteration_input = value;
                self.show_transliteration_input_changed();
                self.persist_settings();
            }
        }
    ),

    all_languages: Vec<Language>,
    source_language_code: String,
    target_language_code: String,
    detected_language_code: String,
    previous_screen: Screen,
    bus_tx: Option<Sender<IoEvent>>,
    asset_dir: String,
    config_dir: String,
    original_image_path: String,
    manage_filter: String,
    expanded_languages: HashSet<String>,
}

#[derive(Clone)]
pub struct UiCallbacks {
    pub set_languages: Arc<dyn Fn(Vec<Language>) + Send + Sync>,
    pub set_feature_progress: Arc<dyn Fn(String, FeatureKind, f32) + Send + Sync>,
    pub set_input_text: Arc<dyn Fn(String) + Send + Sync>,
    pub set_output_text: Arc<dyn Fn(String) + Send + Sync>,
    pub set_selected_image_url: Arc<dyn Fn(String) + Send + Sync>,
    pub set_image_overlay: Arc<dyn Fn(Vec<ImageOverlayListItem>, f32, f32) + Send + Sync>,
    pub set_detected_language_code: Arc<dyn Fn(String) + Send + Sync>,
    pub set_current_screen: Arc<dyn Fn(Screen) + Send + Sync>,
}

impl AppBridge {
    pub fn new(
        languages: Vec<Language>,
        bus_tx: Sender<IoEvent>,
        asset_dir: String,
        config_dir: String,
        settings: Settings,
    ) -> Self {
        let mut app = Self::default();
        app.bus_tx = Some(bus_tx);
        app.current_screen = std::env::var("START_SCREEN")
            .ok()
            .filter(|s| !s.is_empty())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(Screen::NoLanguages.as_i32());
        app.previous_screen = Screen::Translation;
        app.desktop_mode = std::env::var_os("CLICKABLE_DESKTOP_MODE").is_some();
        app.asset_dir = asset_dir;
        app.config_dir = config_dir;
        app.source_language_code = "en".to_string();
        app.target_language_code = "en".to_string();
        app.source_language_name = QString::from("English");
        app.target_language_name = QString::from("English");

        // Apply loaded settings
        app.disable_auto_detect = settings.disable_auto_detect;
        app.font_size = settings.font_size;
        app.ocr_background_mode = QString::from(settings.ocr_background_mode);
        app.ocr_min_confidence = settings.ocr_min_confidence;
        app.ocr_max_image_size = settings.ocr_max_image_size;
        app.catalog_index_url = QString::from(settings.catalog_index_url);
        app.disable_ocr = settings.disable_ocr;
        app.show_transliteration_output = settings.show_transliteration_output;
        app.show_transliteration_input = settings.show_transliteration_input;

        app.set_languages_value(languages);
        app
    }

    fn persist_settings(&self) {
        let settings = Settings {
            default_from: self.source_language_name.to_string(),
            default_to: self.target_language_name.to_string(),
            font_size: self.font_size,
            ocr_background_mode: self.ocr_background_mode.to_string(),
            ocr_min_confidence: self.ocr_min_confidence,
            ocr_max_image_size: self.ocr_max_image_size,
            catalog_index_url: self.catalog_index_url.to_string(),
            disable_ocr: self.disable_ocr,
            disable_auto_detect: self.disable_auto_detect,
            show_transliteration_output: self.show_transliteration_output,
            show_transliteration_input: self.show_transliteration_input,
        };
        save_settings(&self.config_dir, &settings);
    }

    pub fn set_languages_value(&mut self, mut languages: Vec<Language>) {
        languages.sort_by(|left, right| left.name.cmp(&right.name));
        eprintln!("ui.set_languages_value: {} languages", languages.len());
        self.all_languages = languages;
        self.refresh_language_views();

        if self.current_screen == Screen::NoLanguages.as_i32() && self.has_languages {
            self.set_current_screen(Screen::Translation);
        }
    }

    pub fn set_input_text_value(&mut self, text: String) {
        let text = QString::from(text);
        if self.input_text != text {
            self.input_text = text;
            self.input_text_changed();
        }
    }

    pub fn set_feature_progress_value(&mut self, code: &str, feature: FeatureKind, progress: f32) {
        let Some(language) = self
            .all_languages
            .iter_mut()
            .find(|language| language.code == code)
        else {
            return;
        };

        match feature {
            FeatureKind::Core => language.core_progress = progress,
            FeatureKind::Dictionary => language.dictionary_progress = progress,
            FeatureKind::Tts => language.tts_progress = progress,
        }

        let language = language.clone();
        update_progress_list_item(
            &mut self.installed_languages_model.borrow_mut(),
            &language,
            false,
        );
        update_progress_list_item(
            &mut self.available_languages_model.borrow_mut(),
            &language,
            true,
        );
        if self.manage_filter.is_empty()
            || language
                .name
                .to_lowercase()
                .contains(self.manage_filter.as_str())
        {
            update_manage_progress_item(
                &mut self.manage_languages_model.borrow_mut(),
                &language,
                self.expanded_languages.contains(&language.code),
            );
        }
        self.refresh_detected_language();
    }

    pub fn set_output_text_value(&mut self, text: String) {
        let text = QString::from(text);
        if self.output_text != text {
            self.output_text = text;
            self.output_text_changed();
        }
    }

    pub fn set_image_mode_value(&mut self, value: bool) {
        if self.image_mode != value {
            self.image_mode = value;
            self.image_mode_changed();
        }
    }

    pub fn set_selected_image_url_value(&mut self, url: String) {
        let url = QString::from(url);
        if self.selected_image_url != url {
            self.selected_image_url = url;
            self.selected_image_url_changed();
        }
    }

    pub fn set_image_overlay_value(
        &mut self,
        items: Vec<ImageOverlayListItem>,
        width: f32,
        height: f32,
    ) {
        self.image_overlay_model.borrow_mut().reset_data(items);

        if (self.processed_image_width - width).abs() > f32::EPSILON {
            self.processed_image_width = width;
            self.processed_image_width_changed();
        }
        if (self.processed_image_height - height).abs() > f32::EPSILON {
            self.processed_image_height = height;
            self.processed_image_height_changed();
        }
    }

    pub fn set_detected_language_code_value(&mut self, code: &str) {
        if self.detected_language_code != code {
            self.detected_language_code = code.to_string();
            self.refresh_detected_language();
        }
    }

    pub fn set_current_screen(&mut self, screen: Screen) {
        let screen = screen.as_i32();
        if self.current_screen != screen {
            self.current_screen = screen;
            self.current_screen_changed();
        }
    }

    fn set_source_language_by_name(&mut self, name: String) {
        if let Some(language) = self
            .all_languages
            .iter()
            .find(|language| language.name == name)
            .cloned()
        {
            self.source_language_code = language.code.clone();
            let qname = QString::from(language.name);
            if self.source_language_name != qname {
                self.source_language_name = qname;
                self.source_language_name_changed();
            }
            self.refresh_swap_enabled();
            self.refresh_detected_language();
            self.refresh_translation_content();
        }
    }

    fn set_target_language_by_name(&mut self, name: String) {
        if let Some(language) = self
            .all_languages
            .iter()
            .find(|language| language.name == name)
            .cloned()
        {
            self.target_language_code = language.code.clone();
            let qname = QString::from(language.name);
            if self.target_language_name != qname {
                self.target_language_name = qname;
                self.target_language_name_changed();
            }
            self.refresh_swap_enabled();
            self.refresh_translation_content();
        }
    }

    fn swap_languages_impl(&mut self) {
        let source = self.source_language_name.to_string();
        let target = self.target_language_name.to_string();
        self.set_source_language_by_name(target);
        self.set_target_language_by_name(source);
    }

    fn process_text_impl(&mut self, text: String) {
        let qtext = QString::from(text.clone());
        if self.input_text != qtext {
            self.input_text = qtext;
            self.input_text_changed();
        }

        self.send_io(IoEvent::TranslationRequest {
            text,
            from: self.source_language_code.clone(),
            to: self.target_language_code.clone(),
        });
    }

    fn process_image_selection_impl(&mut self, url: String) {
        if self.disable_ocr {
            self.set_output_text_value("OCR is disabled in settings".to_string());
            return;
        }

        if url.is_empty() {
            return;
        }

        let Some(path) = crate::image_ocr::resolve_local_path(&url) else {
            self.set_output_text_value("Couldn't open the selected image".to_string());
            return;
        };

        self.original_image_path = path.display().to_string();
        self.set_image_mode_value(true);
        self.set_selected_image_url_value(url);
        self.set_image_overlay_value(Vec::new(), 0.0, 0.0);
        self.set_input_text_value(String::new());
        self.set_output_text_value("Running OCR...".to_string());
        self.set_detected_language_code_value("");

        self.send_io(IoEvent::ImageTranslationRequest {
            image_path: self.original_image_path.clone(),
            from: self.source_language_code.clone(),
            to: self.target_language_code.clone(),
            min_confidence: self.ocr_min_confidence.max(0) as u32,
            max_image_size: self.ocr_max_image_size.max(0) as u32,
            background_mode: self.ocr_background_mode.to_string(),
        });
    }

    fn refresh_translation_content(&mut self) {
        if self.image_mode {
            self.rerun_current_image();
        } else {
            self.retranslate();
        }
    }

    fn rerun_current_image(&mut self) {
        if self.original_image_path.is_empty() {
            return;
        }

        self.set_image_overlay_value(Vec::new(), 0.0, 0.0);
        self.set_output_text_value("Running OCR...".to_string());
        self.set_detected_language_code_value("");

        self.send_io(IoEvent::ImageTranslationRequest {
            image_path: self.original_image_path.clone(),
            from: self.source_language_code.clone(),
            to: self.target_language_code.clone(),
            min_confidence: self.ocr_min_confidence.max(0) as u32,
            max_image_size: self.ocr_max_image_size.max(0) as u32,
            background_mode: self.ocr_background_mode.to_string(),
        });
    }

    fn retranslate(&mut self) {
        self.send_io(IoEvent::TranslationRequest {
            text: self.input_text.to_string(),
            from: self.source_language_code.clone(),
            to: self.target_language_code.clone(),
        });
    }

    fn set_disable_auto_detect_impl(&mut self, value: bool) {
        if self.disable_auto_detect != value {
            self.disable_auto_detect = value;
            self.disable_auto_detect_changed();
            self.refresh_detected_language();
            self.persist_settings();
        }
    }

    fn missing_language_action_impl(&mut self) {
        let detected_code = self.detected_language_code.clone();
        if detected_code.is_empty() {
            return;
        }

        if let Some(language) = self
            .all_languages
            .iter()
            .find(|language| language.code == detected_code)
            .cloned()
        {
            if language.core_installed || language.built_in {
                self.set_source_language_by_name(language.name);
            } else {
                self.send_feature_request(language.code, FeatureKind::Core, true);
            }
        }
    }

    fn refresh_manage_model(&mut self) {
        let manage_items = self
            .all_languages
            .iter()
            .filter(|language| {
                self.manage_filter.is_empty()
                    || language
                        .name
                        .to_lowercase()
                        .contains(self.manage_filter.as_str())
            })
            .cloned()
            .map(|language| {
                manage_language_to_list_item(
                    &language,
                    self.expanded_languages.contains(&language.code),
                )
            })
            .collect::<Vec<_>>();

        self.manage_languages_model
            .borrow_mut()
            .reset_data(manage_items);
    }

    fn refresh_language_views(&mut self) {
        let installed_items = self
            .all_languages
            .iter()
            .filter(|language| language.core_installed || language.built_in)
            .cloned()
            .map(language_to_list_item)
            .collect::<Vec<_>>();
        let available_items = self
            .all_languages
            .iter()
            .filter(|language| !language.core_installed && !language.built_in)
            .cloned()
            .map(language_to_list_item)
            .collect::<Vec<_>>();

        let manage_items = self
            .all_languages
            .iter()
            .filter(|language| {
                self.manage_filter.is_empty()
                    || language
                        .name
                        .to_lowercase()
                        .contains(self.manage_filter.as_str())
            })
            .cloned()
            .map(|language| {
                manage_language_to_list_item(
                    &language,
                    self.expanded_languages.contains(&language.code),
                )
            })
            .collect::<Vec<_>>();

        eprintln!(
            "ui.refresh_language_views: installed={} available={} manage={} filter='{}'",
            installed_items.len(),
            available_items.len(),
            manage_items.len(),
            self.manage_filter
        );

        self.installed_languages_model
            .borrow_mut()
            .reset_data(installed_items);
        self.available_languages_model
            .borrow_mut()
            .reset_data(available_items);
        eprintln!("ui.refresh_language_views: resetting manage model");
        self.manage_languages_model
            .borrow_mut()
            .reset_data(manage_items);

        let from_names = self
            .all_languages
            .iter()
            .filter(|language| self.is_language_available(language, true))
            .map(|language| QString::from(language.name.clone()))
            .collect::<QStringList>();
        if self.installed_from_language_names != from_names {
            self.installed_from_language_names = from_names;
            self.installed_from_language_names_changed();
        }

        let to_names = self
            .all_languages
            .iter()
            .filter(|language| self.is_language_available(language, false))
            .map(|language| QString::from(language.name.clone()))
            .collect::<QStringList>();
        if self.installed_to_language_names != to_names {
            self.installed_to_language_names = to_names;
            self.installed_to_language_names_changed();
        }

        let has_languages = self
            .all_languages
            .iter()
            .any(|language| !language.built_in && language.core_installed);
        if self.has_languages != has_languages {
            self.has_languages = has_languages;
            self.has_languages_changed();
        }

        self.ensure_selected_languages_are_valid();
        self.refresh_swap_enabled();
        self.refresh_detected_language();
    }

    fn ensure_selected_languages_are_valid(&mut self) {
        if !self.is_language_selectable(&self.source_language_code, true) {
            if let Some(language) = self.first_selectable_language(true) {
                self.source_language_code = language.code.clone();
                let qname = QString::from(language.name);
                if self.source_language_name != qname {
                    self.source_language_name = qname;
                    self.source_language_name_changed();
                }
            }
        }

        if !self.is_language_selectable(&self.target_language_code, false) {
            if let Some(language) = self.first_selectable_language(false) {
                self.target_language_code = language.code.clone();
                let qname = QString::from(language.name);
                if self.target_language_name != qname {
                    self.target_language_name = qname;
                    self.target_language_name_changed();
                }
            }
        }
    }

    fn refresh_swap_enabled(&mut self) {
        let enabled = self
            .find_language_by_code(&self.source_language_code)
            .zip(self.find_language_by_code(&self.target_language_code))
            .map(|(source, target)| {
                matches!(source.direction, Direction::Both)
                    && matches!(target.direction, Direction::Both)
            })
            .unwrap_or(false);

        if self.swap_enabled != enabled {
            self.swap_enabled = enabled;
            self.swap_enabled_changed();
        }
    }

    fn refresh_detected_language(&mut self) {
        let visible = !self.disable_auto_detect && !self.detected_language_code.is_empty();
        let detected = self.find_language_by_code(&self.detected_language_code);
        let (name, installed, progress, show_card) = match detected {
            Some(language) => {
                let show = visible
                    && !matches!(language.direction, Direction::ToOnly)
                    && language.code != self.source_language_code;
                (
                    QString::from(language.name.clone()),
                    language.core_installed || language.built_in,
                    language.core_progress,
                    show,
                )
            }
            None => (QString::default(), false, 0.0, false),
        };

        if self.detected_language_name != name {
            self.detected_language_name = name;
            self.detected_language_name_changed();
        }
        if self.detected_language_installed != installed {
            self.detected_language_installed = installed;
            self.detected_language_installed_changed();
        }
        if (self.detected_language_progress - progress).abs() > f32::EPSILON {
            self.detected_language_progress = progress;
            self.detected_language_progress_changed();
        }
        if self.show_missing_card != show_card {
            self.show_missing_card = show_card;
            self.show_missing_card_changed();
        }
    }

    fn is_language_available(&self, language: &Language, source: bool) -> bool {
        (language.core_installed || language.built_in)
            && if source {
                matches!(language.direction, Direction::FromOnly | Direction::Both)
            } else {
                matches!(language.direction, Direction::ToOnly | Direction::Both)
            }
    }

    fn is_language_selectable(&self, code: &str, source: bool) -> bool {
        self.find_language_by_code(code)
            .map(|language| self.is_language_available(language, source))
            .unwrap_or(false)
    }

    fn first_selectable_language(&self, source: bool) -> Option<Language> {
        self.all_languages
            .iter()
            .find(|language| self.is_language_available(language, source))
            .cloned()
    }

    fn find_language_by_code(&self, code: &str) -> Option<&Language> {
        self.all_languages
            .iter()
            .find(|language| language.code == code)
    }

    fn send_feature_request(&self, code: String, feature: FeatureKind, download: bool) {
        let event = if download {
            IoEvent::DownloadRequest { code, feature }
        } else {
            IoEvent::DeleteLanguage { code, feature }
        };
        self.send_io(event);
    }

    fn send_io(&self, event: IoEvent) {
        if let Some(bus_tx) = &self.bus_tx {
            bus_tx.send(event).unwrap();
        }
    }
}

fn language_to_list_item(language: Language) -> LanguageListItem {
    LanguageListItem {
        code: QString::from(language.code.clone()),
        name: QString::from(language.name),
        size: QString::from(format_size(language.core_size_bytes)),
        installed: language.core_installed,
        download_progress: language.core_progress,
        built_in: language.built_in,
    }
}

fn manage_language_to_list_item(language: &Language, expanded: bool) -> ManageLanguageListItem {
    ManageLanguageListItem {
        code: QString::from(language.code.clone()),
        name: QString::from(language.name.clone()),
        total_size: QString::from(format_size(total_size(language))),
        built_in: language.built_in,
        expanded,
        core_available: language.core_size_bytes > 0,
        core_installed: language.core_installed,
        core_size: QString::from(format_size(language.core_size_bytes)),
        core_progress: language.core_progress,
        dictionary_available: language.dictionary_size_bytes > 0,
        dictionary_installed: language.dictionary_installed,
        dictionary_size: QString::from(format_size(language.dictionary_size_bytes)),
        dictionary_progress: language.dictionary_progress,
        tts_available: language.tts_size_bytes > 0,
        tts_installed: language.tts_installed,
        tts_size: QString::from(format_size(language.tts_size_bytes)),
        tts_progress: language.tts_progress,
    }
}

fn update_progress_list_item(
    model: &mut SimpleListModel<LanguageListItem>,
    language: &Language,
    available_list: bool,
) {
    let target_code = QString::from(language.code.clone());
    let index = {
        model.iter().position(|item| item.code == target_code)
    };
    if let Some(index) = index {
        let should_be_visible = if available_list {
            !language.core_installed && !language.built_in
        } else {
            language.core_installed || language.built_in
        };
        if should_be_visible {
            model.change_line(index, language_to_list_item(language.clone()));
        }
    }
}

fn update_manage_progress_item(
    model: &mut SimpleListModel<ManageLanguageListItem>,
    language: &Language,
    expanded: bool,
) {
    let target_code = QString::from(language.code.clone());
    let index = {
        model.iter().position(|item| item.code == target_code)
    };
    if let Some(index) = index {
        model.change_line(index, manage_language_to_list_item(language, expanded));
    }
}

pub fn argb_to_qml_color(color: u32) -> QString {
    QString::from(format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        (color >> 24) & 0xFF,
        (color >> 16) & 0xFF,
        (color >> 8) & 0xFF,
        color & 0xFF
    ))
}

pub fn create_ui_callbacks(app: QPointer<AppBridge>) -> UiCallbacks {
    let language_app = app.clone();
    let set_languages = queued_callback(move |languages: Vec<Language>| {
        if let Some(app) = language_app.as_pinned() {
            app.borrow_mut().set_languages_value(languages);
        }
    });

    let progress_app = app.clone();
    let set_feature_progress = queued_callback(move |args: (String, i32, f32)| {
        if let Some(app) = progress_app.as_pinned()
            && let Some(feature) = FeatureKind::from_i32(args.1)
        {
            app.borrow_mut()
                .set_feature_progress_value(&args.0, feature, args.2);
        }
    });

    let input_app = app.clone();
    let set_input_text = queued_callback(move |text: String| {
        if let Some(app) = input_app.as_pinned() {
            app.borrow_mut().set_input_text_value(text);
        }
    });

    let output_app = app.clone();
    let set_output_text = queued_callback(move |text: String| {
        if let Some(app) = output_app.as_pinned() {
            app.borrow_mut().set_output_text_value(text);
        }
    });

    let selected_image_app = app.clone();
    let set_selected_image_url = queued_callback(move |url: String| {
        if let Some(app) = selected_image_app.as_pinned() {
            app.borrow_mut().set_selected_image_url_value(url);
        }
    });

    let image_overlay_app = app.clone();
    let set_image_overlay =
        queued_callback(move |args: (Vec<ImageOverlayListItem>, f32, f32)| {
            if let Some(app) = image_overlay_app.as_pinned() {
                app.borrow_mut()
                    .set_image_overlay_value(args.0, args.1, args.2);
            }
        });

    let detected_app = app.clone();
    let set_detected_language_code = queued_callback(move |code: String| {
        if let Some(app) = detected_app.as_pinned() {
            app.borrow_mut().set_detected_language_code_value(&code);
        }
    });

    let screen_app = app.clone();
    let set_current_screen = queued_callback(move |screen: Screen| {
        if let Some(app) = screen_app.as_pinned() {
            app.borrow_mut().set_current_screen(screen);
        }
    });

    UiCallbacks {
        set_languages: Arc::new(move |languages| set_languages(languages)),
        set_feature_progress: Arc::new(move |code, feature, progress| {
            set_feature_progress((code, feature.as_i32(), progress))
        }),
        set_input_text: Arc::new(move |text| set_input_text(text)),
        set_output_text: Arc::new(move |text| set_output_text(text)),
        set_selected_image_url: Arc::new(move |url| set_selected_image_url(url)),
        set_image_overlay: Arc::new(move |items, width, height| {
            set_image_overlay((items, width, height))
        }),
        set_detected_language_code: Arc::new(move |code| set_detected_language_code(code)),
        set_current_screen: Arc::new(move |screen| set_current_screen(screen)),
    }
}
