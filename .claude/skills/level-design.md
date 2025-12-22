# Level Design & Progression Skill

レベルデザインとゲーム進行の設計を支援するスキルです。

## 参照ドキュメント

- `.specify/specs/level-design-progression-best-practices.md`
- `.specify/specs/factory-game-ux-research.md`

---

## 進行フェーズ設計

### Factorio/Satisfactoryモデル

```
Phase 1: 学習期 (0-2時間)
├── 基本操作の習得
├── 手動クラフト
└── 最初の自動化

Phase 2: 戦略期 (2-10時間)
├── 生産ライン構築
├── リソース管理
└── 問題解決

Phase 3: 拡張期 (10-50時間)
├── 大規模工場
├── 最適化
└── 新技術開放

Phase 4: マスタリー (50時間+)
├── メガベース
├── 効率極大化
└── 創造的表現
```

---

## 学習曲線

### チュートリアル設計

```rust
struct TutorialStep {
    objective: String,
    hint: String,
    completion_condition: CompletionCondition,
    highlight_elements: Vec<UiElement>,
}

// 段階的に複雑化
let tutorial = vec![
    TutorialStep {
        objective: "鉄鉱石を10個採掘する",
        hint: "左クリックで採掘",
        completion_condition: HasItem("iron_ore", 10),
        highlight_elements: vec![IronOreVein],
    },
    TutorialStep {
        objective: "製錬炉を設置する",
        hint: "インベントリから製錬炉を選択",
        completion_condition: Placed("furnace"),
        highlight_elements: vec![Hotbar, FurnaceSlot],
    },
    // ...
];
```

### Just-in-Time学習

```rust
// 必要になった時に教える
fn show_contextual_hint(context: &GameContext) {
    match context {
        FirstTimeOpeningInventory => show_hint("inventory_basics"),
        FirstMachineOverstress => show_hint("power_management"),
        FirstConveyorJam => show_hint("conveyor_optimization"),
        _ => {}
    }
}
```

---

## 探索報酬

### 発見可能なコンテンツ

| タイプ | 例 | 報酬 |
|--------|-----|------|
| リソース | 新鉱脈 | 生産効率向上 |
| 技術 | 壊れた機械 | 新レシピ解放 |
| ロア | 日記 | 世界観理解 |
| 秘密 | 隠し部屋 | 限定アイテム |

### 探索動機付け

```rust
struct ExplorationReward {
    discovery_type: DiscoveryType,
    visibility: RewardVisibility,
    permanence: RewardPermanence,
}

enum RewardVisibility {
    Obvious,      // マップに表示
    Hinted,       // 痕跡がある
    Hidden,       // 完全に隠れている
}

enum RewardPermanence {
    Permanent,    // 一度きり
    Respawning,   // 再生成
    Timed,        // 期間限定
}
```

---

## 待機時間管理

### 待機中の代替タスク

```
待機中にできること:
├── 探索
│   └── 新エリアの発見
├── 最適化
│   └── 既存ラインの改善
├── 計画
│   └── 次の拡張を設計
├── 管理
│   └── インベントリ整理
└── 装飾
    └── 工場の見た目改善
```

### 待機時間の可視化

```rust
struct ProductionTimer {
    current: f32,
    total: f32,
    show_eta: bool,
}

fn display_production_status(timer: &ProductionTimer) -> String {
    let progress = timer.current / timer.total * 100.0;
    let remaining = timer.total - timer.current;

    format!(
        "進捗: {:.0}% | 残り: {:.1}秒",
        progress, remaining
    )
}
```

---

## マップレイアウト

### ゾーン設計

```
スタートエリア
├── 基本リソース（鉄、石炭）
├── 平坦な建設地
└── チュートリアル対象

中間エリア
├── 中級リソース（銅、石油）
├── やや複雑な地形
└── 探索報酬

上級エリア
├── 希少リソース（ウラン、希土類）
├── 挑戦的な地形
└── 隠しコンテンツ
```

### リソース配置

```rust
struct ResourceDistribution {
    resource_type: ResourceType,
    density: f32,
    cluster_size: (u32, u32),
    min_distance_from_start: f32,
}

let distributions = vec![
    ResourceDistribution {
        resource_type: Iron,
        density: 0.8,
        cluster_size: (10, 20),
        min_distance_from_start: 0.0,
    },
    ResourceDistribution {
        resource_type: Uranium,
        density: 0.1,
        cluster_size: (3, 8),
        min_distance_from_start: 500.0,
    },
];
```

---

## テクノロジーツリー

### アンロック構造

```
Tier 0: 基礎
├── 手動採掘
├── 基本製錬
└── コンベア

Tier 1: 自動化
├── 採掘機
├── 組立機
└── 電力基礎

Tier 2: 効率化
├── 高速コンベア
├── ロジスティクス
└── 高度製錬

Tier 3: 拡張
├── 列車
├── ドローン
└── 高度電力
```

### アンロック条件

```rust
struct TechnologyUnlock {
    id: String,
    prerequisites: Vec<String>,
    resource_cost: Vec<ItemStack>,
    time_cost: f32,
}

fn can_unlock(tech: &TechnologyUnlock, state: &GameState) -> bool {
    // 前提条件チェック
    for prereq in &tech.prerequisites {
        if !state.unlocked_technologies.contains(prereq) {
            return false;
        }
    }

    // リソースチェック
    for cost in &tech.resource_cost {
        if !state.inventory.has(cost) {
            return false;
        }
    }

    true
}
```

---

## ペース調整

### 目標サイクル

```
短期目標 (5-15分)
├── 次のアイテムをクラフト
└── 小さな生産ライン

中期目標 (30分-2時間)
├── 新技術をアンロック
└── 新エリアへ拡張

長期目標 (数時間-数日)
├── マイルストーン達成
└── メガプロジェクト完成
```

### 達成感の演出

```rust
fn on_milestone_reached(milestone: &Milestone) {
    // 視覚的フィードバック
    play_celebration_effect();

    // 報酬
    grant_rewards(&milestone.rewards);

    // 進捗の可視化
    update_progress_display();

    // 次の目標を提示
    show_next_objectives();
}
```

---

## リプレイ性

### バリエーション要素

| 要素 | バリエーション |
|------|----------------|
| マップ生成 | シード値による変化 |
| 開始条件 | シナリオ、チャレンジ |
| 目標 | 速度、効率、創造性 |
| 制約 | 難易度モード |

### 実績システム

```rust
struct Achievement {
    id: String,
    name: String,
    description: String,
    condition: AchievementCondition,
    hidden: bool,
}

enum AchievementCondition {
    FirstCraft(ItemId),
    TotalProduced(ItemId, u64),
    SpeedRun(Duration),
    NoDeaths,
    Custom(Box<dyn Fn(&GameState) -> bool>),
}
```

---

## チェックリスト

### 進行設計

- [ ] 明確なフェーズ区分があるか
- [ ] 各フェーズに達成感があるか
- [ ] 待機時間に代替タスクがあるか

### 学習曲線

- [ ] チュートリアルが段階的か
- [ ] Just-in-Time学習があるか
- [ ] 失敗からの回復が容易か

### 探索

- [ ] 探索に報酬があるか
- [ ] 報酬の可視性が適切か
- [ ] 探索動機が継続するか

### マップ

- [ ] リソース配置が論理的か
- [ ] 難易度勾配があるか
- [ ] 秘密エリアがあるか

### テクノロジー

- [ ] アンロック順序が明確か
- [ ] 前提条件が論理的か
- [ ] 分岐選択肢があるか

---

*このスキルはレベルデザインと進行設計の品質を確保するためのガイドです。*
