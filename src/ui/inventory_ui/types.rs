// src/ui/inventory_ui/types.rs
//! インベントリUI関連の型定義

use bevy::prelude::*;
use crate::gameplay::inventory::EquipmentSlotType;

/// インベントリUIのステート
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum InventoryUiState {
    #[default]
    Closed,          // HUDのみ
    PlayerInventory, // 'E'キー: [装備] + [Player在庫] + [クラフトリスト]
    Container,       // 右クリック: [装備] + [Player在庫] + [コンテナ在庫]
}

/// 現在開いているコンテナの情報
#[derive(Resource, Default)]
pub struct OpenContainer {
    pub pos: Option<IVec3>,
}

/// ドラッグ中のアイテム情報
#[derive(Resource, Default)]
pub struct DraggedItem {
    pub item_id: Option<String>,
    pub count: u32,
    pub source_slot: Option<SlotIdentifier>,
}

/// スロットの識別子
#[derive(Debug, Clone, PartialEq)]
pub enum SlotIdentifier {
    PlayerInventory(usize),
    Equipment(EquipmentSlotType),
    Container(usize),
}

/// UIルートマーカー
#[derive(Component)]
pub struct InventoryUiRoot;

/// スロットコンポーネント
#[derive(Component)]
pub struct UiSlot {
    pub identifier: SlotIdentifier,
}

/// スプリットダイアログマーカー
#[derive(Component)]
pub struct SplitDialog;

/// ツールチップコンポーネント
#[derive(Component)]
pub struct Tooltip;

/// クラフトボタン
#[derive(Component)]
pub struct CraftButton {
    pub recipe_id: String,
}

/// クリエイティブモードアイテムボタン
#[derive(Component)]
pub struct CreativeItemButton {
    pub item_id: String,
}

/// ソートボタン
#[derive(Component)]
pub struct SortButton;

/// 装備パネルマーカー（クリエイティブモードで非表示にするため）
#[derive(Component)]
pub struct EquipmentPanel;

/// メインインベントリパネルマーカー（クリエイティブモードで非表示にするため）
#[derive(Component)]
pub struct MainInventoryPanel;

/// クリエイティブアイテムカタログマーカー
#[derive(Component)]
pub struct CreativeItemList;

/// クリエイティブ表示モード
#[derive(Resource, Default, PartialEq, Debug)]
pub enum CreativeViewMode {
    #[default]
    Catalog,    // アイテムカタログ
    Inventory,  // プレイヤーインベントリ
}

/// 表示切替ボタン
#[derive(Component)]
pub struct ViewToggleButton;

/// ドラッグ中のアイテム表示マーカー
#[derive(Component)]
pub struct DraggedItemVisual;

/// ゴミ箱スロット
#[derive(Component)]
pub struct TrashSlot;

/// ホットバーHUDマーカー
#[derive(Component)]
pub struct HotbarHud;

/// ホットバー上のアイテム名表示マーカー
#[derive(Component)]
pub struct HotbarItemName;

/// イベント: インベントリを開く
#[derive(Event)]
pub struct OpenInventoryEvent;

/// イベント: インベントリを閉じる
#[derive(Event)]
pub struct CloseInventoryEvent;

/// イベント: コンテナを開く
#[derive(Event)]
pub struct OpenContainerEvent {
    pub pos: IVec3,
}

/// イベント: インベントリをソートする
#[derive(Event)]
pub struct SortInventoryEvent;

/// イベント: アイテムをクラフトする
#[derive(Event)]
pub struct CraftItemEvent {
    pub recipe_id: String,
}
