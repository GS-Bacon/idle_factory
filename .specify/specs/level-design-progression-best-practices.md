# 工場ゲーム レベルデザイン・進行設計ベストプラクティス

**作成日**: 2025-12-22
**目的**: プレイヤーを自然に導き、長期間楽しませるレベルデザインの指針

---

## 1. 進行フェーズ設計

### 1.1 Factorioモデル

```
Phase 1: 学習フェーズ（0-2時間）
├── ベルト、インサーターの基本
├── 電力システムの理解
└── 目標: 最初の機械を動かす

Phase 2: 戦略学習フェーズ（2-10時間）
├── 効率的な配置を学ぶ
├── 「ブループリント」的思考
└── 目標: 安定したラインを構築

Phase 3: 中盤（10-30時間）
├── 流体処理、トレイン
├── 拡張可能な設計
├── ロボット解放
└── 目標: 複数資源を並行処理

Phase 4: 終盤（30時間+）
├── 大規模工場
├── 最適化追求
└── 目標: ロケット発射/実績達成
```

### 1.2 Satisfactoryモデル

```
Tier 1-2: 手動 → 基本自動化
  - ペースはゆっくり
  - 「200時間でTier3でもOK」

Tier 3-5: 電力・物流拡張
  - 新しいバイオームへの探索
  - 3D建築の導入

Tier 6-8: 高度な自動化
  - 大規模再設計の奨励
  - 効率的な輸送手段
```

---

## 2. 学習曲線設計

### 2.1 段階的複雑化

```rust
// レシピの複雑さを段階的に増加
struct RecipeComplexity {
    input_count: u32,      // 入力素材の種類
    output_count: u32,     // 出力素材の種類
    processing_steps: u32, // 前提となる加工ステップ
}

// 推奨進行
const EARLY_GAME: RecipeComplexity = RecipeComplexity {
    input_count: 1,       // 鉄鉱石 → 鉄インゴット
    output_count: 1,
    processing_steps: 1,
};

const MID_GAME: RecipeComplexity = RecipeComplexity {
    input_count: 2-3,     // 鉄板 + 銅線 → 回路
    output_count: 1,
    processing_steps: 2-3,
};

const LATE_GAME: RecipeComplexity = RecipeComplexity {
    input_count: 3-5,     // 複合素材
    output_count: 1-2,
    processing_steps: 4-5,
};
```

### 2.2 チュートリアル統合

```rust
// ゲームプレイに統合されたチュートリアル
enum TutorialType {
    // 良い例: ゲーム内で自然に学ぶ
    IntegratedQuest {
        quest: QuestId,
        teaches: Skill,
    },

    // 避けるべき: 強制的なポップアップ
    ForcedPopup {
        message: String,
        blocks_gameplay: bool,  // これはNG
    },
}

// Factorioスタイル: キャンペーンがチュートリアル
fn first_steps_campaign() -> Vec<Quest> {
    vec![
        Quest::new("採掘機を置く", Skill::Mining),
        Quest::new("ベルトで接続する", Skill::Belts),
        Quest::new("精錬炉を動かす", Skill::Smelting),
        // 自然な流れで学ぶ
    ]
}
```

---

## 3. マップ/ワールドデザイン

### 3.1 資源配置

```rust
struct ResourceDistribution {
    // 基本資源は近くに
    iron: SpawnConfig {
        distance_from_spawn: 0..100,
        frequency: High,
    },
    copper: SpawnConfig {
        distance_from_spawn: 0..100,
        frequency: High,
    },

    // 高度な資源は遠くに（探索の動機）
    uranium: SpawnConfig {
        distance_from_spawn: 500..1000,
        frequency: Low,
    },
}
```

### 3.2 拡張の余地

```
スポーン地点の設計:
  ┌────────────────────────────┐
  │                            │
  │    [石炭]    [鉄]          │
  │         \   /              │
  │          [S]  ← スポーン    │
  │         /   \              │
  │    [銅]      [石]          │
  │                            │
  │    (拡張スペース)           │
  │                            │
  └────────────────────────────┘

原則:
  - 最初は狭い範囲で始められる
  - 拡張方向に障害物がない
  - 後から再設計できる余地
```

### 3.3 探索の報酬

```rust
// 探索で見つかるもの
enum ExplorationReward {
    RichResourcePatch,      // 高品質な資源
    AbandonedFactory,       // 設計図/技術
    NaturalLandmark,        // 建設に適した地形
    HiddenChallenge,        // オプショナルコンテンツ
}

// 発見が次の探索を促す
fn place_exploration_rewards(world: &mut World) {
    // 資源は連鎖的に配置
    // 銅を見つける → その先に金がある → さらに先にウランがある
}
```

---

## 4. ペーシング

### 4.1 活動と休息のサイクル

```
理想的なリズム:
  [建設] → [観察] → [最適化] → [建設] ...

  建設: 新しいラインを作る（能動的）
  観察: 動作を見守る（受動的、満足感）
  最適化: ボトルネックを解消（問題解決）
```

### 4.2 待機時間の管理

```rust
// 待機時間に別のタスクを提供
fn during_production_wait() -> Vec<Activity> {
    vec![
        Activity::OptimizeExistingLines,
        Activity::ExploreNewAreas,
        Activity::PlanNextExpansion,
        Activity::DecorateFactory,
    ]
}

// 早期ゲームでは待機時間を短く
const EARLY_GAME_CRAFTING_TIME: f32 = 1.0;  // 1秒
const LATE_GAME_CRAFTING_TIME: f32 = 10.0; // 10秒（でも並行処理）
```

### 4.3 マイルストーン報酬

```rust
struct Milestone {
    name: String,
    requirements: Vec<Requirement>,
    rewards: Vec<Reward>,
    celebration: Celebration,  // 達成感の演出
}

// 報酬は次の進行を加速
enum Reward {
    UnlockRecipe(RecipeId),
    UnlockBuilding(BuildingId),
    ResourceBonus { item: ItemId, amount: u32 },
    QualityOfLife(QoLFeature),  // 便利機能解放
}
```

---

## 5. 難易度バランス

### 5.1 ボトルネック設計

```rust
// 意図的なボトルネック（学習ポイント）
fn design_bottlenecks() -> Vec<Bottleneck> {
    vec![
        Bottleneck {
            resource: "oil",
            lesson: "流体処理を学ぶ",
            workaround: Some("石炭液化で代替可能"),
        },
        Bottleneck {
            resource: "copper",
            lesson: "規模拡大を学ぶ",
            workaround: None,  // 拡大するしかない
        },
    ]
}

// 避けるべき: 脱出不能なボトルネック
fn bad_bottleneck_example() {
    // NG: 鉄が枯渇して何もできない
    // NG: 敵が強すぎて進めない
    // NG: バグで詰む
}
```

### 5.2 難易度オプション

```rust
struct DifficultySettings {
    // 資源
    resource_richness: f32,     // 0.25-4.0
    resource_frequency: f32,

    // 敵（該当する場合）
    enemy_difficulty: EnemyDifficulty,
    enemy_expansion: bool,

    // 経済
    crafting_speed_multiplier: f32,
    technology_cost_multiplier: f32,

    // Peaceful Mode
    peaceful: bool,
}
```

---

## 6. 再プレイ性

### 6.1 マップバリエーション

```rust
struct WorldGenConfig {
    seed: u64,
    biomes: Vec<BiomeConfig>,
    resource_settings: ResourceSettings,
    special_features: Vec<SpecialFeature>,
}

// シード値で異なる体験
fn generate_world(config: &WorldGenConfig) -> World {
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed);
    // 同じシードなら同じ世界
}
```

### 6.2 チャレンジモード

```rust
enum GameMode {
    Standard,
    SpeedRun { target_time: Duration },
    MinimalFactory { max_machines: u32 },
    NoLogistics { no_belts: bool, no_trains: bool },
    DeathWorld { aggressive_enemies: bool },
}
```

### 6.3 目標の多様性

```rust
// 異なるプレイスタイルに対応
enum PlayerGoal {
    StoryCompletion,        // ストーリーを完了
    Efficiency,             // 最適化を追求
    Megabase,               // 巨大工場を建設
    Speedrun,               // 最速クリア
    AchievementHunting,     // 全実績達成
    CreativeBuilding,       // 見た目重視
}
```

---

## 7. 本プロジェクトへの適用

### 7.1 推奨進行構造

```
Phase 1: 導入（30分-1時間）
├── 手動採掘
├── 最初の機械配置
└── 電力接続

Phase 2: 基本自動化（1-3時間）
├── コンベアの理解
├── 複数機械の連携
└── 基本レシピマスター

Phase 3: 拡張（3-10時間）
├── 電力管理
├── 複雑なレシピ
└── 物流システム

Phase 4: 最適化（10時間+）
├── 効率追求
├── 大規模化
└── カスタム目標
```

### 7.2 実装優先順位

1. **コアループの確立**
   - 採掘 → 加工 → 出力
   - 即座に満足感を得られる

2. **チュートリアル統合**
   - クエストシステムで自然に教える
   - 強制的な説明は避ける

3. **進行システム**
   - 研究ツリー
   - マイルストーン報酬

4. **リプレイ要素**
   - シード生成
   - 難易度設定

---

## 8. チェックリスト

### 進行設計
- [ ] フェーズが明確に定義されているか
- [ ] 各フェーズに達成感があるか
- [ ] 難易度曲線は滑らかか

### マップデザイン
- [ ] 初期資源は近いか
- [ ] 拡張スペースがあるか
- [ ] 探索の動機があるか

### ペーシング
- [ ] 待機時間に別のタスクがあるか
- [ ] 達成感の演出があるか
- [ ] 詰まったときの迂回路があるか

### 再プレイ性
- [ ] ランダム要素があるか
- [ ] 異なるプレイスタイルに対応しているか
- [ ] チャレンジオプションがあるか

---

## 参考文献

- [Stages of Factorio gameplay - Factorio Forums](https://forums.factorio.com/viewtopic.php?t=96024)
- [High-Level Strategy for New Players - Steam Guide](https://steamcommunity.com/sharedfiles/filedetails/?id=2275950965)
- [Tutorial:Production line design tips - Satisfactory Wiki](https://satisfactory.fandom.com/wiki/Tutorial:Production_line_design_tips)

---

*このレポートはFactorioおよびSatisfactoryの進行設計調査に基づいています。*
