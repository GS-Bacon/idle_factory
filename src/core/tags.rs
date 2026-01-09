//! アイテムタグシステム
//!
//! Forge Ore Dictionary相当の機能を提供

use std::collections::{HashMap, HashSet};

use crate::core::ItemId;

/// タグレジストリ
#[derive(Default, Debug)]
pub struct TagRegistry {
    /// タグ → アイテムIDの逆引き
    tag_to_items: HashMap<String, HashSet<ItemId>>,
    /// アイテムID → タグ一覧
    item_to_tags: HashMap<ItemId, Vec<String>>,
}

impl TagRegistry {
    /// 新しいレジストリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// アイテムにタグを登録
    pub fn register(&mut self, item_id: ItemId, tags: &[String]) {
        // アイテム→タグ
        self.item_to_tags.insert(item_id, tags.to_vec());

        // タグ→アイテム（逆引き）
        for tag in tags {
            self.tag_to_items
                .entry(tag.clone())
                .or_default()
                .insert(item_id);

            // 階層タグの親も登録（例: "ingot/iron" → "ingot" にも登録）
            let mut parts: Vec<&str> = tag.split('/').collect();
            while parts.len() > 1 {
                parts.pop();
                let parent_tag = parts.join("/");
                self.tag_to_items
                    .entry(parent_tag)
                    .or_default()
                    .insert(item_id);
            }
        }
    }

    /// タグを持つ全アイテムを取得
    pub fn items_with_tag(&self, tag: &str) -> HashSet<ItemId> {
        self.tag_to_items.get(tag).cloned().unwrap_or_default()
    }

    /// アイテムが特定のタグを持つか
    pub fn has_tag(&self, item_id: ItemId, tag: &str) -> bool {
        self.tag_to_items
            .get(tag)
            .map(|items| items.contains(&item_id))
            .unwrap_or(false)
    }

    /// アイテムのタグ一覧を取得
    pub fn get_tags(&self, item_id: ItemId) -> &[String] {
        self.item_to_tags
            .get(&item_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// タグ指定を解決（#tag_name → アイテムリスト）
    /// 先頭の # を除去して検索
    pub fn resolve_tag_reference(&self, reference: &str) -> Option<HashSet<ItemId>> {
        if let Some(tag) = reference.strip_prefix('#') {
            let items = self.items_with_tag(tag);
            if items.is_empty() {
                None
            } else {
                Some(items)
            }
        } else {
            None // タグ参照ではない
        }
    }

    /// 登録されている全タグを取得
    pub fn all_tags(&self) -> Vec<&str> {
        self.tag_to_items.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    #[test]
    fn test_tag_registration() {
        let mut registry = TagRegistry::new();

        // テスト用にアイテムIDを取得（既存のbase itemsを使用）
        let iron_ingot = items::iron_ingot();
        let copper_ingot = items::copper_ingot();

        registry.register(
            iron_ingot,
            &[
                "ingot".to_string(),
                "ingot/iron".to_string(),
                "metal".to_string(),
            ],
        );
        registry.register(
            copper_ingot,
            &[
                "ingot".to_string(),
                "ingot/copper".to_string(),
                "metal".to_string(),
            ],
        );

        // タグで検索
        let ingots = registry.items_with_tag("ingot");
        assert!(ingots.contains(&iron_ingot));
        assert!(ingots.contains(&copper_ingot));

        // 特定のインゴット
        let iron_ingots = registry.items_with_tag("ingot/iron");
        assert!(iron_ingots.contains(&iron_ingot));
        assert!(!iron_ingots.contains(&copper_ingot));
    }

    #[test]
    fn test_has_tag() {
        let mut registry = TagRegistry::new();
        let iron_ingot = items::iron_ingot();

        registry.register(iron_ingot, &["ingot".to_string(), "metal".to_string()]);

        assert!(registry.has_tag(iron_ingot, "ingot"));
        assert!(registry.has_tag(iron_ingot, "metal"));
        assert!(!registry.has_tag(iron_ingot, "fuel"));
    }

    #[test]
    fn test_resolve_tag_reference() {
        let mut registry = TagRegistry::new();
        let coal = items::coal();

        registry.register(coal, &["fuel".to_string()]);

        // タグ参照の解決
        let fuels = registry.resolve_tag_reference("#fuel");
        assert!(fuels.is_some());
        assert!(fuels.unwrap().contains(&coal));

        // 存在しないタグ
        assert!(registry.resolve_tag_reference("#nonexistent").is_none());

        // タグ参照ではない
        assert!(registry.resolve_tag_reference("coal").is_none());
    }

    #[test]
    fn test_get_tags() {
        let mut registry = TagRegistry::new();
        let iron_ore = items::iron_ore();

        registry.register(iron_ore, &["ore".to_string(), "ore/iron".to_string()]);

        let tags = registry.get_tags(iron_ore);
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"ore".to_string()));
        assert!(tags.contains(&"ore/iron".to_string()));
    }

    #[test]
    fn test_all_tags() {
        let mut registry = TagRegistry::new();
        let stone = items::stone();

        registry.register(
            stone,
            &["terrain".to_string(), "building_material".to_string()],
        );

        let all_tags = registry.all_tags();
        assert!(all_tags.contains(&"terrain"));
        assert!(all_tags.contains(&"building_material"));
    }

    #[test]
    fn test_hierarchical_tags() {
        let mut registry = TagRegistry::new();
        let iron_ingot = items::iron_ingot();
        let copper_ingot = items::copper_ingot();

        // 階層タグを登録
        registry.register(iron_ingot, &["ingot/iron".to_string()]);
        registry.register(copper_ingot, &["ingot/copper".to_string()]);

        // 親タグ "ingot" でも検索できる
        let ingots = registry.items_with_tag("ingot");
        assert!(ingots.contains(&iron_ingot));
        assert!(ingots.contains(&copper_ingot));

        // 子タグで検索
        let iron_only = registry.items_with_tag("ingot/iron");
        assert!(iron_only.contains(&iron_ingot));
        assert!(!iron_only.contains(&copper_ingot));
    }
}
