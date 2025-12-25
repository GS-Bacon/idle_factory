//! AWESOME Sink報酬システム（Satisfactory風）
//!
//! ## 概要
//! 余剰アイテムをポイントに変換し、クーポンと交換できるシステム
//!
//! ## フロー
//! 1. アイテムをAWESOME Sinkに投入
//! 2. アイテムの価値に応じたポイントを獲得
//! 3. ポイントが閾値に達するとクーポンを獲得
//! 4. クーポンでショップから報酬を購入
//!
//! ## ポイント計算
//! - 基本レシピのアイテムは低ポイント
//! - 加工段階が進むほど高ポイント
//! - 指数関数的にクーポン取得コストが増加

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AWESOME Sinkコンポーネント
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AwesomeSink {
    /// 累積ポイント
    pub total_points: u64,
    /// 現在のクーポン進捗ポイント
    pub current_progress: u64,
    /// 次のクーポンに必要なポイント
    pub points_for_next_coupon: u64,
    /// 獲得済みクーポン数
    pub coupons_earned: u32,
    /// 処理待ちアイテムキュー
    pub item_queue: Vec<(String, u32)>,
}

impl AwesomeSink {
    pub fn new() -> Self {
        Self {
            total_points: 0,
            current_progress: 0,
            points_for_next_coupon: Self::calculate_next_threshold(0),
            coupons_earned: 0,
            item_queue: Vec::new(),
        }
    }

    /// 次のクーポン閾値を計算（指数的増加）
    pub fn calculate_next_threshold(coupons_earned: u32) -> u64 {
        // 基本閾値: 1000ポイント
        // 増加率: 1.1^クーポン数
        let base = 1000u64;
        let multiplier = 1.1f64.powi(coupons_earned as i32);
        (base as f64 * multiplier) as u64
    }

    /// アイテムを投入
    pub fn sink_item(&mut self, item_id: &str, count: u32, point_values: &PointValueTable) {
        let points = point_values.get_points(item_id) * count as u64;
        self.add_points(points);
    }

    /// ポイントを追加し、クーポン獲得をチェック
    pub fn add_points(&mut self, points: u64) {
        self.total_points += points;
        self.current_progress += points;

        // クーポン獲得チェック
        while self.current_progress >= self.points_for_next_coupon {
            self.current_progress -= self.points_for_next_coupon;
            self.coupons_earned += 1;
            self.points_for_next_coupon = Self::calculate_next_threshold(self.coupons_earned);
        }
    }

    /// 進捗率（0.0-1.0）
    pub fn progress_percentage(&self) -> f32 {
        if self.points_for_next_coupon == 0 {
            0.0
        } else {
            self.current_progress as f32 / self.points_for_next_coupon as f32
        }
    }
}

/// アイテムのポイント価値テーブル
#[derive(Resource, Debug, Clone, Default)]
pub struct PointValueTable {
    /// アイテムID -> ポイント価値
    pub values: HashMap<String, u64>,
}

impl PointValueTable {
    pub fn new() -> Self {
        let mut table = Self::default();
        table.initialize_default_values();
        table
    }

    /// デフォルトのポイント価値を設定
    fn initialize_default_values(&mut self) {
        // Tier 1: 基本素材（1-10ポイント）
        self.set("iron_ore", 1);
        self.set("copper_ore", 1);
        self.set("tin_ore", 1);
        self.set("coal_ore", 1);
        self.set("stone", 1);
        self.set("log", 2);
        self.set("coal", 3);

        // Tier 1: 一次加工品（5-20ポイント）
        self.set("iron_ingot", 5);
        self.set("copper_ingot", 5);
        self.set("tin_ingot", 5);
        self.set("wood_plank", 4);
        self.set("glass", 6);
        self.set("brick", 6);

        // Tier 2: 二次加工品（15-50ポイント）
        self.set("iron_plate", 15);
        self.set("copper_plate", 15);
        self.set("bronze_ingot", 20);
        self.set("steel_ingot", 35);
        self.set("iron_gear", 25);
        self.set("copper_wire", 20);
        self.set("iron_pipe", 20);

        // Tier 2-3: 部品（50-200ポイント）
        self.set("circuit_board", 100);
        self.set("bearing", 60);
        self.set("piston", 80);
        self.set("electric_motor", 180);
        self.set("advanced_circuit", 250);

        // Tier 4: 高度部品（300+ポイント）
        self.set("processor", 500);
        self.set("battery", 200);
        self.set("solar_cell", 400);

        // 流体（バケット単位）
        self.set("water_bucket", 5);
        self.set("crude_oil_bucket", 20);
        self.set("fuel_bucket", 100);
    }

    /// ポイント価値を設定
    pub fn set(&mut self, item_id: &str, points: u64) {
        self.values.insert(item_id.to_string(), points);
    }

    /// ポイント価値を取得（未登録は1ポイント）
    pub fn get_points(&self, item_id: &str) -> u64 {
        *self.values.get(item_id).unwrap_or(&1)
    }
}

/// クーポンショップアイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    /// アイテムID
    pub id: String,
    /// 表示名
    pub name: String,
    /// 説明
    pub description: String,
    /// 必要クーポン数
    pub coupon_cost: u32,
    /// カテゴリ
    pub category: ShopCategory,
    /// 購入可能回数（Noneなら無制限）
    pub purchase_limit: Option<u32>,
}

/// ショップカテゴリ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShopCategory {
    /// コスメティック（外観）
    Cosmetic,
    /// 乗り物設計図
    VehicleBlueprint,
    /// 特殊アイテム
    SpecialItem,
    /// 建物アンロック
    BuildingUnlock,
    /// ブースト効果
    Boost,
}

/// クーポンショップリソース
#[derive(Resource, Debug, Clone, Default)]
pub struct CouponShop {
    /// 販売アイテム一覧
    pub items: Vec<ShopItem>,
    /// 購入履歴（アイテムID -> 購入回数）
    pub purchase_history: HashMap<String, u32>,
    /// プレイヤーのクーポン残高
    pub player_coupons: u32,
}

impl CouponShop {
    pub fn new() -> Self {
        let mut shop = Self::default();
        shop.initialize_default_items();
        shop
    }

    /// デフォルトのショップアイテムを設定
    fn initialize_default_items(&mut self) {
        self.items.push(ShopItem {
            id: "golden_factory_cart".to_string(),
            name: "Golden Factory Cart".to_string(),
            description: "A shiny golden cart for transporting items in style".to_string(),
            coupon_cost: 5,
            category: ShopCategory::VehicleBlueprint,
            purchase_limit: Some(1),
        });

        self.items.push(ShopItem {
            id: "neon_conveyor_skin".to_string(),
            name: "Neon Conveyor Skin".to_string(),
            description: "Make your conveyors glow with neon lights".to_string(),
            coupon_cost: 3,
            category: ShopCategory::Cosmetic,
            purchase_limit: Some(1),
        });

        self.items.push(ShopItem {
            id: "efficiency_boost".to_string(),
            name: "Efficiency Boost (1h)".to_string(),
            description: "All machines run 10% faster for 1 hour".to_string(),
            coupon_cost: 2,
            category: ShopCategory::Boost,
            purchase_limit: None,
        });

        self.items.push(ShopItem {
            id: "hard_drive".to_string(),
            name: "Hard Drive".to_string(),
            description: "Contains alternate recipe data".to_string(),
            coupon_cost: 10,
            category: ShopCategory::SpecialItem,
            purchase_limit: None,
        });
    }

    /// アイテムを購入
    pub fn purchase(&mut self, item_id: &str) -> Result<(), &'static str> {
        let item = self.items.iter()
            .find(|i| i.id == item_id)
            .ok_or("Item not found")?;

        if self.player_coupons < item.coupon_cost {
            return Err("Not enough coupons");
        }

        // 購入制限チェック
        if let Some(limit) = item.purchase_limit {
            let purchased = self.purchase_history.get(item_id).copied().unwrap_or(0);
            if purchased >= limit {
                return Err("Purchase limit reached");
            }
        }

        // 購入実行
        self.player_coupons -= item.coupon_cost;
        *self.purchase_history.entry(item_id.to_string()).or_insert(0) += 1;

        Ok(())
    }

    /// クーポンを追加
    pub fn add_coupons(&mut self, amount: u32) {
        self.player_coupons += amount;
    }
}

// =====================================
// テスト
// =====================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awesome_sink_points() {
        let mut sink = AwesomeSink::new();
        let points = PointValueTable::new();

        // 基本素材を投入
        sink.sink_item("iron_ore", 100, &points);
        assert_eq!(sink.total_points, 100);

        // 加工品を投入
        sink.sink_item("iron_plate", 10, &points);
        assert_eq!(sink.total_points, 250);
    }

    #[test]
    fn test_coupon_earning() {
        let mut sink = AwesomeSink::new();

        // 最初の閾値は1000ポイント
        assert_eq!(sink.points_for_next_coupon, 1000);

        // 1000ポイント追加 -> 1クーポン獲得
        sink.add_points(1000);
        assert_eq!(sink.coupons_earned, 1);
        assert_eq!(sink.current_progress, 0);

        // 次の閾値は増加
        assert!(sink.points_for_next_coupon > 1000);
    }

    #[test]
    fn test_coupon_shop_purchase() {
        let mut shop = CouponShop::new();
        shop.add_coupons(20);

        // 購入成功
        assert!(shop.purchase("neon_conveyor_skin").is_ok());
        assert_eq!(shop.player_coupons, 17);

        // 同じアイテムは購入不可（制限あり）
        assert!(shop.purchase("neon_conveyor_skin").is_err());

        // クーポン不足
        shop.player_coupons = 0;
        assert!(shop.purchase("hard_drive").is_err());
    }

    #[test]
    fn test_point_values() {
        let table = PointValueTable::new();

        // 登録済みアイテム
        assert_eq!(table.get_points("iron_ore"), 1);
        assert_eq!(table.get_points("processor"), 500);

        // 未登録アイテムは1ポイント
        assert_eq!(table.get_points("unknown_item"), 1);
    }
}
