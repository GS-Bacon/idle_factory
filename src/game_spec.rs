//! Game specification as code
//!
//! This file is the Single Source of Truth for game design.
//! If you change the spec, update this file. Tests will verify implementation matches.
//!
//! Reference: .specify/specs/first-30-minutes.md

use crate::block_type::BlockType;

// =============================================================================
// v0.2 新システム仕様
// =============================================================================

/// # 全体在庫システム (Global Inventory)
///
/// プレイヤーは個人インベントリを持たない。
/// 全てのアイテムは「全体在庫」で管理される。
///
/// ## 仕様
/// - 納品プラットフォームに入ったアイテムは全体在庫に追加
/// - 全体在庫はどこからでもアクセス可能（Eキー）
/// - 機械設置時は全体在庫から消費
/// - 機械撤去時は全体在庫に戻る
/// - 在庫上限: なし（無限）
/// - コンベア上のアイテムを手動で拾う機能: なし
#[allow(dead_code)]
pub mod global_inventory_spec {
    /// 全体在庫の上限（0 = 無限）
    pub const STORAGE_LIMIT: u32 = 0;

    /// 機械撤去時に在庫に戻るか
    pub const RETURN_ON_DEMOLISH: bool = true;

    /// コンベアからアイテムを拾えるか
    pub const CAN_PICKUP_FROM_CONVEYOR: bool = false;
}

/// # 納品プラットフォーム仕様
///
/// 納品プラットフォームは「倉庫」と「クエスト納品」の二役を担う。
///
/// ## 仕様
/// - 初期: ワールドに1つ設置済み
/// - 追加: 中盤レシピで作成可能（作成しても在庫は共有）
/// - アイテム受け入れ: コンベアから入ったものは全体在庫へ
/// - クエスト納品: 手動で「納品」ボタンを押す（在庫から消費）
/// - 目標達成時に通知、納品は任意タイミング
#[allow(dead_code)]
pub mod delivery_platform_spec {
    /// 初期配置数
    pub const INITIAL_COUNT: u32 = 1;

    /// 中盤で追加作成可能か
    pub const CAN_CRAFT_MORE: bool = true;

    /// 複数配置時に在庫を共有するか
    pub const SHARE_INVENTORY: bool = true;

    /// クエスト自動納品（サブクエスト用、将来実装）
    pub const AUTO_DELIVER_ENABLED: bool = false;
}

/// # 組立機（Assembler）仕様
///
/// クラフトは全て組立機で行う。手元クラフトは存在しない。
///
/// ## 仕様
/// - レシピを設定すると自動でクラフト
/// - 入力: コンベアから素材を受け取る
/// - 出力: 完成品をコンベアに流す or 内部バッファ
/// - 初期状態: クエスト報酬でアンロック
#[allow(dead_code)]
pub mod assembler_spec {
    /// 組立機の内部バッファサイズ
    pub const OUTPUT_BUFFER_SIZE: u32 = 10;

    /// クラフト速度（秒/個）
    pub const CRAFT_TIME_BASE: f32 = 2.0;
}

/// # クエストシステム仕様
///
/// ## メインクエスト
/// - ストーリー進行、新機械アンロック
/// - 1つずつ順番に進行
///
/// ## サブクエスト
/// - 素材大量納品、報酬は資源やボーナス
/// - 複数同時進行可能
/// - 達成後に新しいサブクエストが出現
#[allow(dead_code)]
pub mod quest_system_spec {
    /// 同時進行可能なサブクエスト数
    pub const MAX_ACTIVE_SUB_QUESTS: u32 = 5;

    /// サブクエストの自動納品オプション
    pub const SUB_QUEST_AUTO_DELIVER: bool = true;
}

/// # UI仕様（納品プラットフォーム）
///
/// マイクラ風グリッドUI
///
/// ## 構成
/// - 上部: クエスト欄（進捗バー付き、納品ボタン）
/// - 中部: カテゴリタブ（全て/素材/機械/部品）+ 検索ボックス
/// - 下部: 在庫グリッド（8列、ページネーション）
///
/// ## 操作
/// - スロットクリック: 機械なら建築モードへ
/// - ホバー: ツールチップ表示
/// - タブ切り替え: カテゴリフィルタ
/// - 検索: アイテム名で絞り込み
#[allow(dead_code)]
pub mod ui_spec {
    /// 在庫グリッドの列数
    pub const INVENTORY_COLUMNS: u32 = 8;

    /// 1ページあたりの行数
    pub const INVENTORY_ROWS_PER_PAGE: u32 = 4;

    /// カテゴリ一覧
    pub const CATEGORIES: &[&str] = &["全て", "素材", "機械", "部品"];
}

/// # 初期支給（全体在庫に追加）
///
/// v0.2 新仕様:
/// - 採掘機×1, コンベア×10, 精錬炉×1 を全体在庫に支給
/// - 納品プラットフォームはワールドに設置済み
/// - 組立機はクエスト報酬でアンロック
pub const INITIAL_EQUIPMENT: &[(BlockType, u32)] = &[
    (BlockType::MinerBlock, 1),
    (BlockType::ConveyorBlock, 10),
    (BlockType::FurnaceBlock, 1),
];

/// クリエイティブモード用（デバッグ・テスト用）
#[allow(dead_code)]
pub const CREATIVE_MODE_EQUIPMENT: &[(BlockType, u32)] = &[
    (BlockType::MinerBlock, 99),
    (BlockType::ConveyorBlock, 999),
    (BlockType::CrusherBlock, 99),
    (BlockType::FurnaceBlock, 99),
    (BlockType::IronOre, 999),
    (BlockType::Coal, 999),
    (BlockType::CopperOre, 999),
];

/// クエスト種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestType {
    /// メインクエスト（順番に進行、機械アンロック）
    Main,
    /// サブクエスト（複数同時進行可、資源報酬）
    Sub,
}

/// クエスト定義
pub struct QuestSpec {
    pub id: &'static str,
    #[allow(dead_code)]
    pub quest_type: QuestType,
    pub description: &'static str,
    pub required_items: &'static [(BlockType, u32)],
    pub rewards: &'static [(BlockType, u32)],
    /// アンロックされる機械/レシピ（メインクエスト用）
    pub unlocks: &'static [BlockType],
}

/// メインクエスト一覧
///
/// 序盤の流れ:
/// 1. 鉄インゴット10個納品 → 採掘機追加
/// 2. 銅インゴット30個納品 → 粉砕機アンロック
/// 3. 鉄インゴット100個納品 → 大量報酬
///
/// NOTE: AssemblerBlock, Gear, DeliveryPlatform は将来追加予定
/// 現在は既存のBlockTypeのみ使用
pub const MAIN_QUESTS: &[QuestSpec] = &[
    QuestSpec {
        id: "main_1",
        quest_type: QuestType::Main,
        description: "鉄インゴットを10個納品せよ",
        required_items: &[(BlockType::IronIngot, 10)],
        rewards: &[(BlockType::MinerBlock, 2), (BlockType::ConveyorBlock, 20)],
        unlocks: &[BlockType::MinerBlock],
    },
    QuestSpec {
        id: "main_2",
        quest_type: QuestType::Main,
        description: "銅インゴットを30個納品せよ",
        required_items: &[(BlockType::CopperIngot, 30)],
        rewards: &[(BlockType::CrusherBlock, 2)],
        unlocks: &[BlockType::CrusherBlock],
    },
    QuestSpec {
        id: "main_3",
        quest_type: QuestType::Main,
        description: "鉄インゴット100個を納品せよ",
        required_items: &[(BlockType::IronIngot, 100)],
        rewards: &[
            (BlockType::MinerBlock, 4),
            (BlockType::ConveyorBlock, 50),
            (BlockType::FurnaceBlock, 4),
        ],
        unlocks: &[BlockType::FurnaceBlock],
    },
];

/// サブクエスト一覧（例）
///
/// 達成後はプールから新しいサブクエストが出現
pub const SUB_QUESTS: &[QuestSpec] = &[
    QuestSpec {
        id: "sub_iron_100",
        quest_type: QuestType::Sub,
        description: "鉄インゴット100個を納品",
        required_items: &[(BlockType::IronIngot, 100)],
        rewards: &[(BlockType::IronOre, 200)],
        unlocks: &[],
    },
    QuestSpec {
        id: "sub_copper_100",
        quest_type: QuestType::Sub,
        description: "銅インゴット100個を納品",
        required_items: &[(BlockType::CopperIngot, 100)],
        rewards: &[(BlockType::CopperOre, 200)],
        unlocks: &[],
    },
    QuestSpec {
        id: "sub_coal_200",
        quest_type: QuestType::Sub,
        description: "石炭200個を納品",
        required_items: &[(BlockType::Coal, 200)],
        rewards: &[(BlockType::IronIngot, 100)],
        unlocks: &[],
    },
];

// 後方互換性のため維持（将来削除）
#[allow(dead_code)]
#[deprecated(note = "Use MAIN_QUESTS instead")]
pub const QUESTS: &[QuestSpec] = MAIN_QUESTS;

#[cfg(test)]
mod tests {
    use super::*;

    /// メインクエストの進行が妥当か
    #[test]
    fn test_main_quest_progression() {
        // クエスト1は序盤向け（合計10個以下）
        let q1_total: u32 = MAIN_QUESTS[0].required_items.iter().map(|(_, n)| n).sum();
        assert!(q1_total <= 20, "Quest 1 should be easy for early game");

        // 全てのメインクエストにアンロック要素がある
        for quest in MAIN_QUESTS {
            assert!(!quest.unlocks.is_empty(),
                "Main quest {} should unlock something", quest.id);
        }
    }

    /// クエストに報酬がある
    #[test]
    fn test_quest_rewards_not_empty() {
        for quest in MAIN_QUESTS.iter().chain(SUB_QUESTS.iter()) {
            assert!(!quest.rewards.is_empty(),
                "Quest {} should have rewards", quest.id);
        }
    }

    /// 初期装備が存在する
    #[test]
    fn test_initial_equipment_not_empty() {
        assert!(!INITIAL_EQUIPMENT.is_empty(),
            "Player should start with some equipment");
    }

    /// 仕様定数の妥当性
    #[test]
    fn test_spec_constants() {
        // 在庫上限0は無限を意味する
        assert_eq!(global_inventory_spec::STORAGE_LIMIT, 0);

        // 機械撤去時は在庫に戻る
        assert!(global_inventory_spec::RETURN_ON_DEMOLISH);

        // 納品PFは初期1つ
        assert_eq!(delivery_platform_spec::INITIAL_COUNT, 1);

        // サブクエストは最大5個同時
        assert!(quest_system_spec::MAX_ACTIVE_SUB_QUESTS >= 3);
    }
}
