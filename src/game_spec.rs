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

/// # バイオーム採掘システム仕様
///
/// 採掘機は設置場所のバイオームに応じた確率テーブルで鉱石を生成する。
/// 真下のブロックは無視し、バイオームの特性で産出物が決まる。
///
/// ## 仕様
/// - 採掘機は座標からバイオームを判定
/// - バイオームごとに確率テーブルが存在
/// - 1回の採掘で1種類のアイテムを確率で選択
/// - レア鉱石（金など）は低確率で出現
/// - 一部バイオームは採掘不可（海、溶岩など）
///
/// ## スポーン地点保証
/// - 初期納品プラットフォーム周辺（半径15ブロック以内）に
///   鉄鉱石、石炭、銅鉱石バイオームが必ず生成される
/// - 初期コンベア10本で到達可能な範囲に配置
#[allow(dead_code)]
pub mod biome_mining_spec {
    use crate::BlockType;

    /// 採掘確率エントリ（鉱石タイプ, 確率%）
    pub type MiningProbability = (BlockType, u32);

    /// 鉄鉱石バイオームの確率テーブル
    /// 合計100%
    pub const IRON_BIOME: &[MiningProbability] = &[
        (BlockType::IronOre, 70), // 70% 鉄鉱石
        (BlockType::Stone, 22),   // 22% 石
        (BlockType::Coal, 8),     // 8% 石炭
    ];

    /// 銅鉱石バイオームの確率テーブル
    /// 合計100%
    pub const COPPER_BIOME: &[MiningProbability] = &[
        (BlockType::CopperOre, 70), // 70% 銅鉱石
        (BlockType::Stone, 22),     // 22% 石
        (BlockType::IronOre, 8),    // 8% 鉄鉱石
    ];

    /// 石炭バイオームの確率テーブル
    pub const COAL_BIOME: &[MiningProbability] = &[
        (BlockType::Coal, 75),   // 75% 石炭
        (BlockType::Stone, 20),  // 20% 石
        (BlockType::IronOre, 5), // 5% 鉄鉱石
    ];

    /// 石バイオームの確率テーブル（貧鉱地帯）
    pub const STONE_BIOME: &[MiningProbability] = &[
        (BlockType::Stone, 85),  // 85% 石
        (BlockType::Coal, 10),   // 10% 石炭
        (BlockType::IronOre, 5), // 5% 鉄鉱石
    ];

    /// 混合バイオームの確率テーブル（バランス型）
    pub const MIXED_BIOME: &[MiningProbability] = &[
        (BlockType::IronOre, 30),   // 30% 鉄鉱石
        (BlockType::CopperOre, 25), // 25% 銅鉱石
        (BlockType::Coal, 25),      // 25% 石炭
        (BlockType::Stone, 20),     // 20% 石
    ];

    /// スポーン地点の保証半径（ブロック単位）
    pub const SPAWN_GUARANTEE_RADIUS: u32 = 15;

    /// スポーン地点で保証されるバイオーム
    /// これらが必ずSPAWN_GUARANTEE_RADIUS内に生成される
    pub const GUARANTEED_SPAWN_BIOMES: &[&str] = &["iron", "coal", "copper"];

    /// 採掘不可バイオーム
    pub const UNMAILABLE_BIOMES: &[&str] = &["ocean", "lava", "void"];
}

/// # 初期支給（全体在庫に追加）
///
/// v0.2 新仕様:
/// - 採掘機×2, コンベア×30, 精錬炉×1 を全体在庫に支給
/// - 納品プラットフォームはワールドに設置済み
/// - 組立機はクエスト報酬でアンロック
///
/// ## 設計意図
/// - コンベア30本: 納品PFから各鉱石バイオーム（半径15以内）に
///   往復ラインを敷設可能（片道約10本×3ライン）
/// - 採掘機2台: 鉄と石炭（または銅）を同時に採掘開始可能
/// - 精錬炉1台: インゴット生成でクエスト1をクリア可能
pub const INITIAL_EQUIPMENT: &[(BlockType, u32)] = &[
    (BlockType::MinerBlock, 2),
    (BlockType::ConveyorBlock, 30),
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

// =============================================================================
// 機械仕様
// =============================================================================

/// # 機械の入出力方向仕様
///
/// 全ての機械は設置時の向きに基づいて入出力方向が固定される。
/// プレイヤーが向いている方向を「前」として設置される。
///
/// ## 方向定義
/// - 前 (Front): プレイヤーが向いていた方向（設置時に固定）
/// - 後 (Back): 前の反対方向
/// - 左/右 (Left/Right): 前に対して90度回転
///
/// ## 入出力パターン
/// 各機械は以下のパターンを持つ:
/// - 入力: コンベアからアイテムを受け取る方向
/// - 出力: 完成品をコンベアに流す方向
///
/// ## 接続ルール
/// - 機械の入力口にはコンベアの「出口」が接続される必要がある
/// - 機械の出力口にはコンベアの「入口」が接続される必要がある
/// - 方向が合わない場合、アイテムは流れない
#[allow(dead_code)]
pub mod machine_io_spec {
    /// 機械の向き（設置時のプレイヤーの向き）
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum MachineFacing {
        North, // -Z
        South, // +Z
        East,  // +X
        West,  // -X
    }

    /// 入出力ポートの位置
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum PortSide {
        Front,  // 機械の前面
        Back,   // 機械の背面
        Left,   // 機械の左側
        Right,  // 機械の右側
        Top,    // 機械の上面
        Bottom, // 機械の下面（未使用だが将来用）
    }

    /// 入出力ポート定義
    #[derive(Clone, Copy, Debug)]
    pub struct IoPort {
        /// ポートの位置
        pub side: PortSide,
        /// 入力ポートか（false = 出力ポート）
        pub is_input: bool,
        /// スロットID（複数入力/出力がある場合）
        pub slot_id: u8,
    }
}

/// # 機械定義（データ駆動）
///
/// 全ての機械は `MachineSpec` として統一的に定義される。
/// 入出力ポート、処理時間、バッファサイズなどを一元管理。
#[allow(dead_code)]
pub mod machine_spec {
    use super::machine_io_spec::{IoPort, PortSide};
    use crate::BlockType;

    /// 機械仕様定義
    #[derive(Clone, Debug)]
    pub struct MachineSpec {
        /// 機械ID
        pub id: &'static str,
        /// 表示名
        pub name: &'static str,
        /// 対応するBlockType
        pub block_type: BlockType,
        /// 入出力ポート
        pub ports: &'static [IoPort],
        /// 内部バッファサイズ（各スロット）
        pub buffer_size: u32,
        /// 基本処理時間（秒）
        pub process_time: f32,
        /// 燃料を必要とするか
        pub requires_fuel: bool,
        /// 自動生成するか（採掘機など）
        pub auto_generate: bool,
    }

    // =========================================================================
    // 機械定義
    // =========================================================================

    /// 採掘機
    pub const MINER: MachineSpec = MachineSpec {
        id: "miner",
        name: "採掘機",
        block_type: BlockType::MinerBlock,
        ports: &[IoPort {
            side: PortSide::Front,
            is_input: false,
            slot_id: 0,
        }],
        buffer_size: 64,
        process_time: 1.5,
        requires_fuel: false,
        auto_generate: true, // 地面から自動で資源を生成
    };

    /// 精錬炉
    pub const FURNACE: MachineSpec = MachineSpec {
        id: "furnace",
        name: "精錬炉",
        block_type: BlockType::FurnaceBlock,
        ports: &[
            IoPort {
                side: PortSide::Back,
                is_input: true,
                slot_id: 0,
            }, // 鉱石入力
            IoPort {
                side: PortSide::Front,
                is_input: false,
                slot_id: 0,
            }, // インゴット出力
        ],
        buffer_size: 64,
        process_time: 2.0,
        requires_fuel: true, // 石炭必要（現在はUI経由で投入）
        auto_generate: false,
    };

    /// 粉砕機
    pub const CRUSHER: MachineSpec = MachineSpec {
        id: "crusher",
        name: "粉砕機",
        block_type: BlockType::CrusherBlock,
        ports: &[
            IoPort {
                side: PortSide::Back,
                is_input: true,
                slot_id: 0,
            },
            IoPort {
                side: PortSide::Front,
                is_input: false,
                slot_id: 0,
            },
        ],
        buffer_size: 64,
        process_time: 1.5,
        requires_fuel: false,
        auto_generate: false,
    };

    /// 全機械一覧
    pub const ALL_MACHINES: &[&MachineSpec] = &[&MINER, &FURNACE, &CRUSHER];

    /// BlockTypeから機械仕様を取得
    pub fn get_machine_spec(block_type: BlockType) -> Option<&'static MachineSpec> {
        ALL_MACHINES
            .iter()
            .find(|m| m.block_type == block_type)
            .copied()
    }

    /// 機械の入力ポートを取得
    pub fn get_input_ports(spec: &MachineSpec) -> impl Iterator<Item = &IoPort> {
        spec.ports.iter().filter(|p| p.is_input)
    }

    /// 機械の出力ポートを取得
    pub fn get_output_ports(spec: &MachineSpec) -> impl Iterator<Item = &IoPort> {
        spec.ports.iter().filter(|p| !p.is_input)
    }
}

// =============================================================================
// レシピシステム
// =============================================================================

/// # レシピシステム仕様
///
/// 全ての加工レシピは `RecipeSpec` として定義される。
/// 機械タイプごとにレシピが紐づく。
///
/// ## レシピの構成要素
/// - 機械タイプ: どの機械で加工するか
/// - 入力アイテム: 必要な素材（複数可、スロット指定）
/// - 出力アイテム: 完成品（複数可）
/// - 加工時間: 秒単位
///
/// ## スロット指定
/// 組立機のような複数入力機械では、素材をどのスロットに入れるか指定が必要。
/// スロット0 = メイン、スロット1 = サブ1、スロット2 = サブ2
#[allow(dead_code)]
pub mod recipe_spec {
    use crate::BlockType;

    /// 機械タイプ
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum MachineType {
        Furnace,   // 精錬炉
        Crusher,   // 粉砕機
        Assembler, // 組立機（将来）
    }

    /// レシピ入力
    #[derive(Clone, Copy, Debug)]
    pub struct RecipeInput {
        /// アイテムタイプ
        pub item: BlockType,
        /// 必要個数
        pub count: u32,
        /// 入力スロットID（0 = メイン、1以降 = サブ）
        pub slot: u8,
    }

    impl RecipeInput {
        pub const fn new(item: BlockType, count: u32, slot: u8) -> Self {
            Self { item, count, slot }
        }
    }

    /// レシピ出力
    #[derive(Clone, Copy, Debug)]
    pub struct RecipeOutput {
        /// アイテムタイプ
        pub item: BlockType,
        /// 出力個数
        pub count: u32,
        /// 出力確率（0.0-1.0、1.0 = 確定）
        pub chance: f32,
    }

    impl RecipeOutput {
        /// 確定出力を作成
        pub const fn guaranteed(item: BlockType, count: u32) -> Self {
            Self {
                item,
                count,
                chance: 1.0,
            }
        }

        /// 確率出力を作成
        pub const fn chance(item: BlockType, count: u32, chance: f32) -> Self {
            Self {
                item,
                count,
                chance,
            }
        }
    }

    /// 燃料要件
    #[derive(Clone, Copy, Debug)]
    pub struct FuelRequirement {
        /// 燃料アイテムタイプ
        pub fuel_type: BlockType,
        /// 1回の加工で消費する量
        pub amount: u32,
    }

    impl FuelRequirement {
        pub const fn new(fuel_type: BlockType, amount: u32) -> Self {
            Self { fuel_type, amount }
        }
    }

    /// レシピ定義
    #[derive(Clone, Debug)]
    pub struct RecipeSpec {
        /// レシピID（ユニーク）
        pub id: &'static str,
        /// 使用する機械
        pub machine: MachineType,
        /// 入力素材リスト
        pub inputs: &'static [RecipeInput],
        /// 出力アイテムリスト（確率付き）
        pub outputs: &'static [RecipeOutput],
        /// 加工時間（秒）
        pub craft_time: f32,
        /// 燃料要件（None = 燃料不要）
        pub fuel: Option<FuelRequirement>,
    }

    impl RecipeSpec {
        /// 確定出力のみを取得
        pub fn guaranteed_outputs(&self) -> impl Iterator<Item = &RecipeOutput> {
            self.outputs.iter().filter(|o| o.chance >= 1.0)
        }

        /// 確率出力を取得
        pub fn chance_outputs(&self) -> impl Iterator<Item = &RecipeOutput> {
            self.outputs.iter().filter(|o| o.chance < 1.0)
        }
    }

    // =========================================================================
    // 精錬レシピ
    // =========================================================================

    /// 鉄鉱石 → 鉄インゴット（石炭1消費）
    pub const RECIPE_SMELT_IRON: RecipeSpec = RecipeSpec {
        id: "smelt_iron",
        machine: MachineType::Furnace,
        inputs: &[RecipeInput::new(BlockType::IronOre, 1, 0)],
        outputs: &[RecipeOutput::guaranteed(BlockType::IronIngot, 1)],
        craft_time: 2.0,
        fuel: Some(FuelRequirement::new(BlockType::Coal, 1)),
    };

    /// 銅鉱石 → 銅インゴット（石炭1消費）
    pub const RECIPE_SMELT_COPPER: RecipeSpec = RecipeSpec {
        id: "smelt_copper",
        machine: MachineType::Furnace,
        inputs: &[RecipeInput::new(BlockType::CopperOre, 1, 0)],
        outputs: &[RecipeOutput::guaranteed(BlockType::CopperIngot, 1)],
        craft_time: 2.0,
        fuel: Some(FuelRequirement::new(BlockType::Coal, 1)),
    };

    /// 全精錬レシピ
    pub const FURNACE_RECIPES: &[&RecipeSpec] = &[&RECIPE_SMELT_IRON, &RECIPE_SMELT_COPPER];

    // =========================================================================
    // 粉砕レシピ
    // =========================================================================

    /// 鉄鉱石 → 鉄鉱石×2（燃料不要）
    pub const RECIPE_CRUSH_IRON: RecipeSpec = RecipeSpec {
        id: "crush_iron",
        machine: MachineType::Crusher,
        inputs: &[RecipeInput::new(BlockType::IronOre, 1, 0)],
        outputs: &[RecipeOutput::guaranteed(BlockType::IronOre, 2)],
        craft_time: 1.5,
        fuel: None,
    };

    /// 銅鉱石 → 銅鉱石×2
    pub const RECIPE_CRUSH_COPPER: RecipeSpec = RecipeSpec {
        id: "crush_copper",
        machine: MachineType::Crusher,
        inputs: &[RecipeInput::new(BlockType::CopperOre, 1, 0)],
        outputs: &[RecipeOutput::guaranteed(BlockType::CopperOre, 2)],
        craft_time: 1.5,
        fuel: None,
    };

    /// 全粉砕レシピ
    pub const CRUSHER_RECIPES: &[&RecipeSpec] = &[&RECIPE_CRUSH_IRON, &RECIPE_CRUSH_COPPER];

    // =========================================================================
    // 組立レシピ（将来用）
    // =========================================================================

    // /// 鉄インゴット×2 → 歯車
    // pub const RECIPE_CRAFT_GEAR: RecipeSpec = RecipeSpec {
    //     id: "craft_gear",
    //     machine: MachineType::Assembler,
    //     inputs: &[RecipeInput::new(BlockType::IronIngot, 2, 0)],
    //     outputs: &[RecipeOutput::guaranteed(BlockType::Gear, 1)],
    //     craft_time: 3.0,
    //     fuel: None,
    // };

    // =========================================================================
    // 確率副産物の例（将来用）
    // =========================================================================

    // /// 鉄鉱石精錬（副産物付き）
    // /// メイン: 鉄インゴット×1 (確定)
    // /// 副産物: スラグ×1 (20%確率)
    // pub const RECIPE_SMELT_IRON_ADVANCED: RecipeSpec = RecipeSpec {
    //     id: "smelt_iron_advanced",
    //     machine: MachineType::Furnace,
    //     inputs: &[RecipeInput::new(BlockType::IronOre, 1, 0)],
    //     outputs: &[
    //         RecipeOutput::guaranteed(BlockType::IronIngot, 1),
    //         RecipeOutput::chance(BlockType::Slag, 1, 0.2),
    //     ],
    //     craft_time: 2.0,
    //     fuel: Some(FuelRequirement::new(BlockType::Coal, 1)),
    // };

    /// 全レシピ一覧
    pub const ALL_RECIPES: &[&RecipeSpec] = &[
        // Furnace
        &RECIPE_SMELT_IRON,
        &RECIPE_SMELT_COPPER,
        // Crusher
        &RECIPE_CRUSH_IRON,
        &RECIPE_CRUSH_COPPER,
    ];

    /// 入力アイテムからレシピを検索（機械タイプ指定）
    pub fn find_recipe(machine: MachineType, input: BlockType) -> Option<&'static RecipeSpec> {
        ALL_RECIPES
            .iter()
            .find(|r| r.machine == machine && r.inputs.iter().any(|i| i.item == input))
            .copied()
    }

    /// 機械タイプの全レシピを取得
    pub fn get_recipes_for_machine(
        machine: MachineType,
    ) -> impl Iterator<Item = &'static RecipeSpec> {
        ALL_RECIPES
            .iter()
            .filter(move |r| r.machine == machine)
            .copied()
    }
}

/// # 機械の処理速度仕様
///
/// 各機械の基本処理速度。将来的にはアップグレードで変更可能。
#[allow(dead_code)]
pub mod machine_speed_spec {
    /// 採掘機の採掘間隔（秒/個）
    pub const MINER_INTERVAL: f32 = 1.5;

    /// 精錬炉の精錬時間（秒/個）
    pub const FURNACE_SMELT_TIME: f32 = 2.0;

    /// 粉砕機の粉砕時間（秒/個）
    pub const CRUSHER_CRUSH_TIME: f32 = 1.5;

    /// 組立機の組立時間（秒/個、レシピで上書き可能）
    pub const ASSEMBLER_BASE_TIME: f32 = 3.0;

    /// コンベアの搬送速度（ブロック/秒）
    pub const CONVEYOR_SPEED: f32 = 2.0;
}

/// # 機械間直接接続仕様
///
/// 機械同士が隣接して入出力面が正しく向き合っている場合、
/// コンベアなしで直接アイテムを受け渡しできる。
///
/// ## 接続条件
/// - 機械Aの出力ポートと機械Bの入力ポートが隣接している
/// - ポートの向きが正しい（AのFrontがBのBackに接続 等）
///
/// ## 例: 採掘機 → 精錬炉
/// ```text
///   [採掘機] → [精錬炉] → ...
///      Front→Back
/// ```
/// 採掘機の前面（出力）が精錬炉の背面（入力）に接続。
///
/// ## 例: 粉砕機 → 精錬炉
/// ```text
///   [粉砕機] → [精錬炉] → ...
///      Front→Back
/// ```
///
/// ## 搬送速度
/// 機械間の直接接続はコンベアより遅い（処理完了時に即座に渡す）。
/// 出力バッファから入力バッファへ1個ずつ移動。
#[allow(dead_code)]
pub mod machine_connection_spec {
    /// 機械間直接接続を有効にするか
    pub const ENABLE_DIRECT_CONNECTION: bool = true;

    /// 直接接続時の搬送間隔（秒/個）
    /// 0.0 = 即座に渡す（処理完了時）
    pub const DIRECT_TRANSFER_INTERVAL: f32 = 0.0;

    /// 直接接続時、受け取り側のバッファが満杯なら待機するか
    pub const WAIT_ON_FULL_BUFFER: bool = true;
}

/// # 機械のブロック表現仕様
///
/// 機械は通常のブロックと同様に1x1x1のキューブとして描画される。
/// 見た目はテクスチャで区別する（3Dモデル不要）。
///
/// ## テクスチャ構成
/// 各機械は6面のテクスチャを持つ:
/// - 前面 (Front): 出力口を示すマーク
/// - 背面 (Back): 入力口を示すマーク
/// - 上面 (Top): 機械の種類を示すアイコン
/// - 下面 (Bottom): 統一テクスチャ
/// - 左右 (Left/Right): サイドテクスチャ
///
/// ## 向き表示
/// 機械の向きはテクスチャの向きで判別可能。
/// 矢印や入出力マークで直感的に分かるようにする。
///
/// ## 将来: 2x2機械
/// 大型機械は2x2ブロックを占有し、メインブロック+ダミーブロックで構成。
/// v0.3以降で検討。
#[allow(dead_code)]
pub mod machine_rendering_spec {
    /// 機械をブロックとして描画するか（true = ブロック、false = 3Dモデル）
    pub const RENDER_AS_BLOCK: bool = true;

    /// 機械ブロックのサイズ（1.0 = 通常ブロックと同じ）
    pub const MACHINE_BLOCK_SIZE: f32 = 1.0;

    /// 向きインジケーターを表示するか（矢印等）
    pub const SHOW_DIRECTION_INDICATOR: bool = true;

    /// 入出力ポートを視覚的に表示するか
    pub const SHOW_IO_PORTS: bool = true;

    /// テクスチャパス（将来用）
    pub const TEXTURE_PATH_PREFIX: &str = "textures/machines/";

    // ========================================
    // 視覚フィードバック仕様（マイクラ風）
    // ========================================

    /// 機械の動作状態
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum MachineState {
        /// 待機中（入力待ち、燃料切れなど）
        Idle,
        /// 稼働中（処理実行中）
        Working,
    }

    /// スケール/パルスアニメーション（廃止）
    /// 注意: 以前の採掘機のパルスアニメーションは削除
    pub const USE_SCALE_ANIMATION: bool = false;

    /// 煙パーティクル設定
    pub mod smoke {
        /// 煙パーティクルを有効化
        pub const ENABLED: bool = true;

        /// 煙を出す機械タイプ（Working状態のみ）
        pub const SMOKE_MACHINES: &[&str] = &["furnace", "crusher"];

        /// パーティクル発生間隔（秒）
        pub const SPAWN_INTERVAL: f32 = 0.3;

        /// パーティクル上昇速度
        pub const RISE_SPEED: f32 = 0.5;

        /// パーティクル寿命（秒）
        pub const LIFETIME: f32 = 1.5;

        /// パーティクルサイズ
        pub const SIZE: f32 = 0.15;

        /// パーティクル色（グレー、半透明）
        pub const COLOR: (f32, f32, f32, f32) = (0.5, 0.5, 0.5, 0.6);
    }

    /// 状態別の見た目変化（マイクラのカマド風）
    pub mod state_visuals {
        /// 稼働中の明るさ倍率（1.0 = 変化なし）
        pub const WORKING_BRIGHTNESS: f32 = 1.3;

        /// 稼働中に発光エフェクトを付けるか
        pub const WORKING_EMISSIVE: bool = true;

        /// 発光色（オレンジ系、精錬炉向け）
        pub const EMISSIVE_COLOR_FURNACE: (f32, f32, f32) = (1.0, 0.5, 0.1);

        /// 発光色（青系、粉砕機向け）
        pub const EMISSIVE_COLOR_CRUSHER: (f32, f32, f32) = (0.3, 0.5, 1.0);

        /// 発光色（緑系、採掘機向け）
        pub const EMISSIVE_COLOR_MINER: (f32, f32, f32) = (0.2, 0.8, 0.3);

        /// 発光強度（0.0〜1.0）
        pub const EMISSIVE_INTENSITY: f32 = 0.5;
    }

    /// 各機械の状態判定ルール
    /// - Miner: buffer未満かつ下にリソースブロックあり → Working
    /// - Furnace: 入力あり かつ 燃料あり → Working
    /// - Crusher: 入力あり → Working
    pub const STATE_RULES: &str = r#"
        Miner:   Working if (buffer < 64) && has_resource_below
        Furnace: Working if (input > 0) && (fuel > 0)
        Crusher: Working if (input > 0)
    "#;
}

#[cfg(test)]
mod tests {
    use super::*;
    use recipe_spec::*;

    /// メインクエストの進行が妥当か
    #[test]
    fn test_main_quest_progression() {
        // クエスト1は序盤向け（合計10個以下）
        let q1_total: u32 = MAIN_QUESTS[0].required_items.iter().map(|(_, n)| n).sum();
        assert!(q1_total <= 20, "Quest 1 should be easy for early game");

        // 全てのメインクエストにアンロック要素がある
        for quest in MAIN_QUESTS {
            assert!(
                !quest.unlocks.is_empty(),
                "Main quest {} should unlock something",
                quest.id
            );
        }
    }

    /// クエストに報酬がある
    #[test]
    fn test_quest_rewards_not_empty() {
        for quest in MAIN_QUESTS.iter().chain(SUB_QUESTS.iter()) {
            assert!(
                !quest.rewards.is_empty(),
                "Quest {} should have rewards",
                quest.id
            );
        }
    }

    /// 初期装備が存在する
    #[test]
    fn test_initial_equipment_not_empty() {
        assert!(
            !INITIAL_EQUIPMENT.is_empty(),
            "Player should start with some equipment"
        );
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

    /// レシピシステムの妥当性
    #[test]
    fn test_recipe_system() {
        // 全レシピが入力と出力を持つ
        for recipe in ALL_RECIPES {
            assert!(
                !recipe.inputs.is_empty(),
                "Recipe {} should have inputs",
                recipe.id
            );
            assert!(
                !recipe.outputs.is_empty(),
                "Recipe {} should have outputs",
                recipe.id
            );
            assert!(
                recipe.craft_time > 0.0,
                "Recipe {} should have positive craft time",
                recipe.id
            );
        }

        // レシピ検索が動作する
        let iron_smelt = find_recipe(MachineType::Furnace, BlockType::IronOre);
        assert!(iron_smelt.is_some(), "Should find iron smelting recipe");
        assert_eq!(
            iron_smelt.expect("iron smelt recipe should exist").id,
            "smelt_iron"
        );

        let copper_crush = find_recipe(MachineType::Crusher, BlockType::CopperOre);
        assert!(copper_crush.is_some(), "Should find copper crushing recipe");
    }

    /// 燃料要件の検証
    #[test]
    fn test_fuel_requirements() {
        // 精錬炉は燃料が必要
        let iron_smelt = find_recipe(MachineType::Furnace, BlockType::IronOre)
            .expect("iron smelt recipe should exist");
        assert!(
            iron_smelt.fuel.is_some(),
            "Furnace recipes should require fuel"
        );
        let fuel = iron_smelt.fuel.expect("fuel requirement should exist");
        assert_eq!(fuel.fuel_type, BlockType::Coal);
        assert_eq!(fuel.amount, 1);

        // 粉砕機は燃料不要
        let iron_crush = find_recipe(MachineType::Crusher, BlockType::IronOre)
            .expect("iron crush recipe should exist");
        assert!(
            iron_crush.fuel.is_none(),
            "Crusher recipes should not require fuel"
        );
    }

    /// 出力確率の検証
    #[test]
    fn test_output_chances() {
        // 現在のレシピは全て確定出力
        for recipe in ALL_RECIPES {
            for output in recipe.outputs {
                assert!(
                    output.chance >= 1.0,
                    "Recipe {} output {} should be guaranteed",
                    recipe.id,
                    output.item.name()
                );
            }
        }

        // guaranteed_outputs / chance_outputs のテスト
        let iron_smelt = find_recipe(MachineType::Furnace, BlockType::IronOre)
            .expect("iron smelt recipe should exist");
        let guaranteed: Vec<_> = iron_smelt.guaranteed_outputs().collect();
        let chance: Vec<_> = iron_smelt.chance_outputs().collect();
        assert_eq!(guaranteed.len(), 1);
        assert_eq!(chance.len(), 0);
    }

    /// 機械別レシピ取得
    #[test]
    fn test_get_recipes_for_machine() {
        let furnace_recipes: Vec<_> = get_recipes_for_machine(MachineType::Furnace).collect();
        assert_eq!(furnace_recipes.len(), 2);

        let crusher_recipes: Vec<_> = get_recipes_for_machine(MachineType::Crusher).collect();
        assert_eq!(crusher_recipes.len(), 2);

        let assembler_recipes: Vec<_> = get_recipes_for_machine(MachineType::Assembler).collect();
        assert_eq!(assembler_recipes.len(), 0); // 将来用
    }

    /// 機械仕様の妥当性
    #[test]
    fn test_machine_spec() {
        use machine_spec::*;

        // 全機械が定義されている
        assert!(!ALL_MACHINES.is_empty());

        // 各機械に少なくとも1つのポートがある
        for machine in ALL_MACHINES {
            assert!(
                !machine.ports.is_empty(),
                "Machine {} should have at least one port",
                machine.id
            );
            assert!(
                machine.buffer_size > 0,
                "Machine {} should have positive buffer size",
                machine.id
            );
        }

        // BlockTypeから機械仕様を取得できる
        let miner = get_machine_spec(BlockType::MinerBlock);
        assert!(miner.is_some());
        assert_eq!(miner.expect("miner spec should exist").id, "miner");

        let furnace = get_machine_spec(BlockType::FurnaceBlock);
        assert!(furnace.is_some());
        assert_eq!(furnace.expect("furnace spec should exist").id, "furnace");

        // 非機械ブロックはNone
        let stone = get_machine_spec(BlockType::Stone);
        assert!(stone.is_none());
    }

    /// 入出力ポートの整合性
    #[test]
    fn test_io_ports() {
        use machine_spec::*;

        // 採掘機: 出力のみ（入力なし）
        let miner_inputs: Vec<_> = get_input_ports(&MINER).collect();
        let miner_outputs: Vec<_> = get_output_ports(&MINER).collect();
        assert!(miner_inputs.is_empty(), "Miner should have no input ports");
        assert_eq!(miner_outputs.len(), 1, "Miner should have one output port");

        // 精錬炉: 入力1、出力1
        let furnace_inputs: Vec<_> = get_input_ports(&FURNACE).collect();
        let furnace_outputs: Vec<_> = get_output_ports(&FURNACE).collect();
        assert_eq!(
            furnace_inputs.len(),
            1,
            "Furnace should have one input port"
        );
        assert_eq!(
            furnace_outputs.len(),
            1,
            "Furnace should have one output port"
        );

        // 粉砕機: 入力1、出力1
        let crusher_inputs: Vec<_> = get_input_ports(&CRUSHER).collect();
        let crusher_outputs: Vec<_> = get_output_ports(&CRUSHER).collect();
        assert_eq!(
            crusher_inputs.len(),
            1,
            "Crusher should have one input port"
        );
        assert_eq!(
            crusher_outputs.len(),
            1,
            "Crusher should have one output port"
        );
    }
}
