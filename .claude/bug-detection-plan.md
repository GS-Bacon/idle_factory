# バグ取りシステム強化計画

**作成日**: 2026-01-02
**相談者**: Claude + Gemini

## 概要

5つの柱で段階的にバグ取りシステムを強化する。
「基盤の安定」→「検出の質向上」→「自動化・効率化」の順で進める。

## 現状

| 項目 | 値 |
|------|-----|
| コード行数 | 11,500行 |
| テスト数 | 142件 |
| アサーション | 418個 |
| unwrap() | 38箇所（src内） |
| VLMチェック | 6レベル対応 |
| ファズテスト | 基本版あり |

## フェーズ構成

### Phase 1: 防御壁の構築（最優先）

**目標**: クラッシュさせない、壊れたコードを入れない

| # | タスク | 柱 | 工数 | 効果 |
|---|--------|-----|------|------|
| 1-1 | unwrap()削除（38箇所） | 基盤 | 小 | 極大 |
| 1-2 | pre-commit hook設定 | 自動化 | 小 | 中 |
| 1-3 | 状態ダンプ機能 | デバッグ | 中 | 大 |

#### 1-1: unwrap()削除

```rust
// Before
let value = map.get(&key).unwrap();

// After: Option 1 - expect with context
let value = map.get(&key).expect("key should exist in map");

// After: Option 2 - match/if let
if let Some(value) = map.get(&key) {
    // use value
}

// After: Option 3 - ? operator (in functions returning Result)
let value = map.get(&key).ok_or(GameError::KeyNotFound)?;
```

#### 1-2: pre-commit hook

```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo fmt --check || exit 1
cargo check || exit 1
cargo test --lib || exit 1
cargo clippy -- -D warnings || exit 1
```

#### 1-3: 状態ダンプ機能

```rust
// src/debug/state_dump.rs
pub fn dump_game_state(world: &World) -> GameStateDump {
    GameStateDump {
        timestamp: Utc::now(),
        player: extract_player_state(world),
        inventory: extract_inventory(world),
        machines: extract_machines(world),
        conveyors: extract_conveyors(world),
    }
}

// テスト失敗時に自動保存
// logs/state_dump_YYYYMMDD_HHMMSS.json
```

---

### Phase 2: テストの質向上

**目標**: 「動いている」→「正しく動いている」へ

| # | タスク | 柱 | 工数 | 効果 |
|---|--------|-----|------|------|
| 2-1 | E2Eアサーション強化 | 精度 | 中 | 大 |
| 2-2 | カバレッジ計測導入 | 新手法 | 小 | 中 |
| 2-3 | 決定論的実行の保証 | 基盤 | 中 | 大 |

#### 2-1: E2Eアサーション強化

```rust
// Before: 動くことだけ確認
miner.tick();
assert!(miner.output_slot.is_some());

// After: 値の正しさも確認
miner.tick();
assert_eq!(miner.output_slot.as_ref().unwrap().item_type, ItemType::IronOre);
assert_eq!(miner.output_slot.as_ref().unwrap().count, 1);
assert_eq!(miner.progress, 0.0); // リセット確認
assert!(miner.last_tick_time > prev_tick_time); // 時刻更新確認
```

#### 2-2: カバレッジ計測

```bash
# cargo-tarpaulin導入
cargo install cargo-tarpaulin

# 計測実行
cargo tarpaulin --out Html --output-dir coverage/

# 目標: 主要ロジック70%以上
```

#### 2-3: 決定論的実行の保証

```rust
// 乱数シード固定
fn setup_rng(mut commands: Commands) {
    let seed = std::env::var("GAME_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12345);
    commands.insert_resource(GameRng(StdRng::seed_from_u64(seed)));
}

// システム順序の明示的定義
app.add_systems(Update, (
    conveyor_input.before(conveyor_move),
    conveyor_move.before(conveyor_output),
    conveyor_output,
));
```

---

### Phase 3: 自動化と深掘り

**目標**: 人間では見つけにくいバグを機械に発見させる

| # | タスク | 柱 | 工数 | 効果 |
|---|--------|-----|------|------|
| 3-1 | proptest導入 | 新手法 | 大 | 大 |
| 3-2 | スクリーンショット比較 | 自動化 | 中 | 中 |
| 3-3 | ファズテスト強化 | 新手法 | 中 | 中 |
| 3-4 | パフォーマンス監視 | 精度 | 小 | 中 |

#### 3-1: proptest導入

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.4"
```

```rust
use proptest::prelude::*;

proptest! {
    // 不変条件: アイテム移動前後で総量は変わらない
    #[test]
    fn inventory_preserves_total(
        initial in 0u32..1000,
        transfer in 0u32..500
    ) {
        let mut inv_a = Inventory::with_items(initial);
        let mut inv_b = Inventory::new();
        let before_total = inv_a.count() + inv_b.count();

        inv_a.transfer_to(&mut inv_b, transfer.min(initial));

        let after_total = inv_a.count() + inv_b.count();
        prop_assert_eq!(before_total, after_total);
    }

    // 不変条件: インベントリは負にならない
    #[test]
    fn inventory_never_negative(
        add in 0u32..1000,
        remove in 0u32..2000
    ) {
        let mut inv = Inventory::new();
        inv.add("iron", add);
        inv.remove("iron", remove);
        prop_assert!(inv.count("iron") >= 0);
    }
}
```

#### 3-2: スクリーンショット比較

```bash
# scripts/visual_regression.sh
#!/bin/bash
# ベースライン保存
./scripts/vlm_check.sh --save-baseline

# 変更後に比較（ピクセル差分）
./scripts/vlm_check.sh --compare-baseline --threshold 0.01

# 差分が閾値を超えたら警告
```

```python
# scripts/vlm_check/pixel_compare.py
from PIL import Image, ImageChops

def compare_images(baseline_path, current_path, threshold=0.01):
    baseline = Image.open(baseline_path)
    current = Image.open(current_path)

    diff = ImageChops.difference(baseline, current)
    diff_ratio = sum(diff.histogram()) / (baseline.width * baseline.height * 3 * 255)

    return {
        "match": diff_ratio < threshold,
        "diff_ratio": diff_ratio,
        "diff_image": diff
    }
```

#### 3-3: ファズテスト強化

```bash
# 追加するコマンドパターン
COMMANDS+=(
    # 境界値テスト
    "/tp 0 0 0"
    "/tp 999999 999999 999999"
    "/give iron 0"
    "/give iron 999999"

    # 連続操作
    "/spawn 0 8 0 miner && /spawn 0 8 0 miner"  # 重複設置
    "/save test && /load test && /save test"    # 連続セーブロード

    # 不正入力
    "/give '' 1"
    "/tp a b c"
)
```

#### 3-4: パフォーマンス監視

```rust
// E2Eテスト内でフレームタイム計測
#[test]
fn test_performance_baseline() {
    let mut app = create_test_app();

    let start = Instant::now();
    for _ in 0..1000 {
        app.update();
    }
    let elapsed = start.elapsed();

    let avg_frame_time = elapsed.as_millis() as f64 / 1000.0;
    assert!(avg_frame_time < 16.67, "FPS dropped below 60: {}ms/frame", avg_frame_time);
}
```

---

### Phase 4: 高度なAI活用

**目標**: 確定的テストで拾えないものをAIで検出

| # | タスク | 柱 | 工数 | 効果 |
|---|--------|-----|------|------|
| 4-1 | VLMベースライン比較 | 精度 | 大 | 中 |
| 4-2 | AIログ解析 | デバッグ | 中 | 中 |
| 4-3 | テストレポート統合 | 自動化 | 中 | 中 |

#### 4-1: VLMベースライン比較

```python
# 2枚の画像をClaudeに比較させる
def vlm_compare(baseline_path, current_path):
    prompt = """
    2枚のゲームスクリーンショットを比較してください。

    1枚目: ベースライン（正常）
    2枚目: 現在の状態

    以下の観点で差異を報告:
    1. レイアウトの変化
    2. 色味の変化
    3. 欠落している要素
    4. 新しく追加された要素
    5. 位置ずれ

    JSON形式で:
    {
        "identical": true/false,
        "differences": ["差異1", "差異2"],
        "severity": "none" | "minor" | "major" | "critical"
    }
    """
```

#### 4-2: AIログ解析

```bash
# scripts/analyze_logs.sh
#!/bin/bash
LOG_FILE=$(ls -t logs/game_*.log | head -1)

./scripts/ask_gemini.sh "
以下のゲームログを分析して、問題を検出してください。

確認項目:
1. エラーパターン
2. 警告の頻度
3. 異常な値（負数、極端に大きい数）
4. 繰り返しパターン（無限ループの兆候）
5. タイミング異常

ログ:
$(tail -500 $LOG_FILE)
"
```

---

## 優先順位マトリクス

| 順位 | タスク | フェーズ | 効果 | 工数 |
|------|--------|----------|------|------|
| 1 | unwrap()削除 | 1 | 極大 | 小 |
| 2 | 状態ダンプ機能 | 1 | 大 | 中 |
| 3 | pre-commit hook | 1 | 中 | 小 |
| 4 | E2Eアサーション強化 | 2 | 大 | 中 |
| 5 | 決定論的実行の保証 | 2 | 大 | 中 |
| 6 | カバレッジ計測 | 2 | 中 | 小 |
| 7 | proptest導入 | 3 | 大 | 大 |
| 8 | スクリーンショット比較 | 3 | 中 | 中 |
| 9 | パフォーマンス監視 | 3 | 中 | 小 |
| 10 | ファズテスト強化 | 3 | 中 | 中 |
| 11 | VLMベースライン比較 | 4 | 中 | 大 |
| 12 | AIログ解析 | 4 | 中 | 中 |

## 型安全強化（継続的リファクタリング）

新コード追加時に徐々に適用:

```rust
// NewType pattern
pub struct ItemCount(pub u32);
pub struct BlockPos(pub IVec3);
pub struct ChunkPos(pub IVec2);

// 間違った引数の型エラーをコンパイル時に検出
fn place_block(pos: BlockPos, block: BlockType) { ... }
fn get_chunk(pos: ChunkPos) -> Option<&Chunk> { ... }

// 不変条件の型表現
pub struct NonEmptyInventory(Inventory); // 空でないことを保証
pub struct ValidatedSaveData(SaveData);  // 検証済みデータ
```

## Geminiからの追加提案

### A. 決定論的実行の保証（重要）

スクリーンショット比較やE2Eテストが安定するために必須:
- 乱数シードの固定
- Bevyスケジュール順序の明示的定義
- フレームレート非依存の物理演算

### B. パフォーマンスリグレッション検知

- E2Eテスト内でフレームタイム計測
- 閾値超過で警告
- ベンチマークテストの追加

## 成功指標

| 指標 | 現在 | Phase 1後 | Phase 2後 | Phase 3後 |
|------|------|-----------|-----------|-----------|
| unwrap() | 38 | 0 | 0 | 0 |
| テストカバレッジ | 8.5% | 8.5% | 20%+ | 40%+ |
| E2Eアサーション | 418 | 450+ | 600+ | 800+ |
| Flaky tests | ? | 0 | 0 | 0 |
| pre-commit通過率 | - | 100% | 100% | 100% |

## 次のアクション

1. **今すぐ**: Phase 1-1 unwrap()削除に着手
2. **今週中**: Phase 1完了（防御壁構築）
3. **来週**: Phase 2開始（テストの質向上）
