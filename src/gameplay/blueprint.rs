//! ブループリント設計台システム（Satisfactory/Shapez2風）
//!
//! ## 概要
//! 工場の一部を設計図として保存し、他の場所に複製可能
//!
//! ## 機能
//! - 範囲選択で構造物をキャプチャ
//! - ブループリントとして保存
//! - 配置モードでプレビュー表示
//! - 必要素材の自動計算
//! - エクスポート/インポート（共有用）

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ブループリント設計台のティア
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlueprintDesignerTier {
    /// Mk1: 最大32m立方体
    Mk1,
    /// Mk2: 最大64m立方体、ホログラムプレビュー
    Mk2,
}

impl BlueprintDesignerTier {
    pub fn max_size(&self) -> u32 {
        match self {
            BlueprintDesignerTier::Mk1 => 32,
            BlueprintDesignerTier::Mk2 => 64,
        }
    }

    pub fn has_hologram(&self) -> bool {
        matches!(self, BlueprintDesignerTier::Mk2)
    }
}

/// ブループリント設計台コンポーネント
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintDesigner {
    /// 設計台のティア
    pub tier: BlueprintDesignerTier,
    /// 選択開始位置
    pub selection_start: Option<IVec3>,
    /// 選択終了位置
    pub selection_end: Option<IVec3>,
    /// 現在編集中のブループリント
    pub current_blueprint: Option<Blueprint>,
}

impl Default for BlueprintDesigner {
    fn default() -> Self {
        Self {
            tier: BlueprintDesignerTier::Mk1,
            selection_start: None,
            selection_end: None,
            current_blueprint: None,
        }
    }
}

impl BlueprintDesigner {
    pub fn new(tier: BlueprintDesignerTier) -> Self {
        Self {
            tier,
            ..Default::default()
        }
    }

    /// 選択範囲を設定
    pub fn set_selection(&mut self, start: IVec3, end: IVec3) -> Result<(), &'static str> {
        let size = (end - start).abs();
        let max = self.tier.max_size() as i32;

        if size.x > max || size.y > max || size.z > max {
            return Err("Selection too large for this designer tier");
        }

        self.selection_start = Some(start);
        self.selection_end = Some(end);
        Ok(())
    }

    /// 選択をクリア
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }
}

/// ブループリント（設計図）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    /// ブループリントID
    pub id: String,
    /// 表示名
    pub name: String,
    /// 説明
    pub description: String,
    /// サイズ（相対座標）
    pub size: IVec3,
    /// 含まれる構造物
    pub structures: Vec<BlueprintStructure>,
    /// 必要素材（自動計算）
    pub required_materials: HashMap<String, u32>,
    /// 作成日時（タイムスタンプ）
    pub created_at: u64,
    /// 最終更新日時
    pub updated_at: u64,
}

impl Blueprint {
    pub fn new(name: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: format!("bp_{}", now),
            name: name.to_string(),
            description: String::new(),
            size: IVec3::ZERO,
            structures: Vec::new(),
            required_materials: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 構造物を追加
    pub fn add_structure(&mut self, structure: BlueprintStructure) {
        // 必要素材を更新
        *self.required_materials
            .entry(structure.block_id.clone())
            .or_insert(0) += 1;

        self.structures.push(structure);
        self.recalculate_size();
    }

    /// サイズを再計算
    fn recalculate_size(&mut self) {
        if self.structures.is_empty() {
            self.size = IVec3::ZERO;
            return;
        }

        let mut min = IVec3::new(i32::MAX, i32::MAX, i32::MAX);
        let mut max = IVec3::new(i32::MIN, i32::MIN, i32::MIN);

        for s in &self.structures {
            min = min.min(s.relative_pos);
            max = max.max(s.relative_pos);
        }

        self.size = max - min + IVec3::ONE;
    }

    /// 構造物数
    pub fn structure_count(&self) -> usize {
        self.structures.len()
    }

    /// シリアライズ（共有用）
    pub fn export(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// デシリアライズ（インポート用）
    pub fn import(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

/// ブループリント内の構造物
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStructure {
    /// ブロックID
    pub block_id: String,
    /// 相対位置（ブループリント原点からのオフセット）
    pub relative_pos: IVec3,
    /// 向き
    pub orientation: BlueprintOrientation,
    /// 追加設定（機械の設定など）
    pub settings: HashMap<String, String>,
}

/// ブループリント内の向き
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BlueprintOrientation {
    #[default]
    North,
    East,
    South,
    West,
}

impl BlueprintOrientation {
    /// 90度回転
    pub fn rotate_cw(&self) -> Self {
        match self {
            BlueprintOrientation::North => BlueprintOrientation::East,
            BlueprintOrientation::East => BlueprintOrientation::South,
            BlueprintOrientation::South => BlueprintOrientation::West,
            BlueprintOrientation::West => BlueprintOrientation::North,
        }
    }

    /// 90度逆回転
    pub fn rotate_ccw(&self) -> Self {
        match self {
            BlueprintOrientation::North => BlueprintOrientation::West,
            BlueprintOrientation::West => BlueprintOrientation::South,
            BlueprintOrientation::South => BlueprintOrientation::East,
            BlueprintOrientation::East => BlueprintOrientation::North,
        }
    }
}

/// ブループリントライブラリリソース
#[derive(Resource, Debug, Clone, Default)]
pub struct BlueprintLibrary {
    /// 保存されたブループリント
    pub blueprints: HashMap<String, Blueprint>,
    /// カテゴリ分け
    pub categories: HashMap<String, Vec<String>>, // category -> blueprint IDs
}

impl BlueprintLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// ブループリントを保存
    pub fn save(&mut self, blueprint: Blueprint) {
        self.blueprints.insert(blueprint.id.clone(), blueprint);
    }

    /// ブループリントを取得
    pub fn get(&self, id: &str) -> Option<&Blueprint> {
        self.blueprints.get(id)
    }

    /// ブループリントを削除
    pub fn delete(&mut self, id: &str) -> Option<Blueprint> {
        self.blueprints.remove(id)
    }

    /// カテゴリにブループリントを追加
    pub fn add_to_category(&mut self, category: &str, blueprint_id: &str) {
        self.categories
            .entry(category.to_string())
            .or_default()
            .push(blueprint_id.to_string());
    }

    /// 全ブループリントのリスト
    pub fn list_all(&self) -> Vec<&Blueprint> {
        self.blueprints.values().collect()
    }
}

/// ブループリント配置モード
#[derive(Debug, Clone, Default)]
pub struct BlueprintPlacementMode {
    /// 配置するブループリント
    pub blueprint: Option<Blueprint>,
    /// 配置位置
    pub position: IVec3,
    /// 回転（0, 90, 180, 270度）
    pub rotation: u8,
    /// ホログラム表示中か
    pub show_hologram: bool,
    /// 配置可能かどうか
    pub can_place: bool,
    /// 不足素材
    pub missing_materials: HashMap<String, u32>,
}

impl BlueprintPlacementMode {
    /// 90度回転
    pub fn rotate(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    /// 位置を移動
    pub fn move_to(&mut self, position: IVec3) {
        self.position = position;
    }
}

// =====================================
// テスト
// =====================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blueprint_creation() {
        let mut bp = Blueprint::new("Test Factory");
        assert_eq!(bp.name, "Test Factory");
        assert_eq!(bp.structures.len(), 0);

        bp.add_structure(BlueprintStructure {
            block_id: "conveyor".to_string(),
            relative_pos: IVec3::new(0, 0, 0),
            orientation: BlueprintOrientation::North,
            settings: HashMap::new(),
        });

        assert_eq!(bp.structure_count(), 1);
        assert_eq!(bp.required_materials.get("conveyor"), Some(&1));
    }

    #[test]
    fn test_blueprint_export_import() {
        let mut bp = Blueprint::new("Export Test");
        bp.add_structure(BlueprintStructure {
            block_id: "assembler".to_string(),
            relative_pos: IVec3::new(0, 0, 0),
            orientation: BlueprintOrientation::East,
            settings: HashMap::new(),
        });

        let exported = bp.export().unwrap();
        let imported = Blueprint::import(&exported).unwrap();

        assert_eq!(imported.name, "Export Test");
        assert_eq!(imported.structure_count(), 1);
    }

    #[test]
    fn test_designer_selection() {
        let mut designer = BlueprintDesigner::new(BlueprintDesignerTier::Mk1);

        // 有効な選択
        assert!(designer.set_selection(IVec3::ZERO, IVec3::new(10, 10, 10)).is_ok());

        // サイズオーバー
        assert!(designer.set_selection(IVec3::ZERO, IVec3::new(50, 50, 50)).is_err());
    }

    #[test]
    fn test_orientation_rotation() {
        let north = BlueprintOrientation::North;
        assert_eq!(north.rotate_cw(), BlueprintOrientation::East);
        assert_eq!(north.rotate_ccw(), BlueprintOrientation::West);
    }

    #[test]
    fn test_library() {
        let mut lib = BlueprintLibrary::new();
        let bp = Blueprint::new("Test");
        let id = bp.id.clone();

        lib.save(bp);
        assert!(lib.get(&id).is_some());

        lib.delete(&id);
        assert!(lib.get(&id).is_none());
    }
}
