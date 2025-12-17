use crate::models::{LocaleFile, LocalizationEntry};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// ローカライズマネージャー
pub struct LocalizationManager {
    base_path: PathBuf,
}

impl LocalizationManager {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// ロケールファイルのパスを取得
    fn locale_path(&self, lang: &str) -> PathBuf {
        self.base_path.join(format!("{}.ron", lang))
    }

    /// ロケールファイルを読み込み
    pub fn load_locale(&self, lang: &str) -> Result<LocaleFile, String> {
        let path = self.locale_path(lang);
        if !path.exists() {
            return Ok(LocaleFile::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read locale file: {}", e))?;

        ron::from_str(&content).map_err(|e| format!("Failed to parse locale file: {}", e))
    }

    /// ロケールファイルを保存
    pub fn save_locale(&self, lang: &str, locale: &LocaleFile) -> Result<(), String> {
        let path = self.locale_path(lang);

        // ディレクトリが存在しない場合は作成
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create locale directory: {}", e))?;
        }

        let content = ron::ser::to_string_pretty(locale, ron::ser::PrettyConfig::default())
            .map_err(|e| format!("Failed to serialize locale: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write locale file: {}", e))
    }

    /// エントリを更新/追加
    pub fn update_entry(
        &self,
        lang: &str,
        key: &str,
        entry: LocalizationEntry,
    ) -> Result<(), String> {
        let mut locale = self.load_locale(lang)?;
        locale.entries.insert(key.to_string(), entry);
        self.save_locale(lang, &locale)
    }

    /// 複数言語のエントリを一括更新
    pub fn update_entries(
        &self,
        key: &str,
        entries: HashMap<String, LocalizationEntry>,
    ) -> Result<(), String> {
        for (lang, entry) in entries {
            self.update_entry(&lang, key, entry)?;
        }
        Ok(())
    }

    /// エントリを取得
    pub fn get_entry(&self, lang: &str, key: &str) -> Result<Option<LocalizationEntry>, String> {
        let locale = self.load_locale(lang)?;
        Ok(locale.entries.get(key).cloned())
    }

    /// 全エントリを取得 (指定キーの全言語)
    #[allow(dead_code)]
    pub fn get_all_entries(
        &self,
        key: &str,
    ) -> Result<HashMap<String, LocalizationEntry>, String> {
        let mut result = HashMap::new();

        for lang in &["ja", "en"] {
            if let Some(entry) = self.get_entry(lang, key)? {
                result.insert(lang.to_string(), entry);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_localization_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = LocalizationManager::new(temp_dir.path());

        let entry = LocalizationEntry {
            name: "鉄鉱石".to_string(),
            description: "基本的な鉱石です".to_string(),
        };

        manager.update_entry("ja", "item.iron_ore", entry.clone()).unwrap();

        let loaded = manager.get_entry("ja", "item.iron_ore").unwrap().unwrap();
        assert_eq!(loaded.name, "鉄鉱石");
    }
}
