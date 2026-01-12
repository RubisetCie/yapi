use std::collections::HashMap;

use crate::db_context::DbContext;
use crate::error::Result;
use crate::models::{EditorKeymap, Settings, SettingsIden};
use crate::util::UpdateSource;

impl<'a> DbContext<'a> {
    pub fn get_settings(&self) -> Settings {
        let id = "default".to_string();

        if let Some(s) = self.find_optional::<Settings>(SettingsIden::Id, &id) {
            return s;
        };

        let settings = Settings {
            model: "settings".to_string(),
            id,
            created_at: Default::default(),
            updated_at: Default::default(),

            appearance: "system".to_string(),
            client_certificates: Vec::new(),
            editor_font_size: 12,
            editor_font: None,
            editor_keymap: EditorKeymap::Default,
            editor_soft_wrap: true,
            interface_font_size: 14,
            interface_scale: 1.0,
            interface_font: None,
            hide_window_controls: false,
            use_native_titlebar: false,
            open_workspace_new_window: None,
            proxy: None,
            theme_dark: "yaak-dark".to_string(),
            theme_light: "yaak-light".to_string(),
            colored_methods: false,
            hotkeys: HashMap::new(),
        };
        self.upsert(&settings, &UpdateSource::Background).expect("Failed to upsert settings")
    }

    pub fn upsert_settings(&self, settings: &Settings, source: &UpdateSource) -> Result<Settings> {
        self.upsert(settings, source)
    }
}
