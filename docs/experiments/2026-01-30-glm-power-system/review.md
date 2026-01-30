# 電力システム実装レビュー

## 概要

GLM4.7 + Speckit + SwamTool で実装された電力システム（M3）をレビュー。

**結論**: 仕様書は高品質だが、**実装が仕様と乖離** + **コンパイルエラー多数**

---

## 1. 発見された問題

### 🔴 P0: ビルドブロッカー（4件）

| ファイル | 行 | 問題 |
|----------|-----|------|
| `src/setup/ui/mod.rs` | 12 | `pub use settings_ui::*{` → 構文エラー |
| `src/setup/ui/mod.rs` | 26 | `pub fn text_font:` → 関数シグネチャ破損 |
| `src/systems/power/generator_tick.rs` | 59-62 | 閉じ括弧の重複 |
| `src/systems/power/grid_calc.rs` | 177-190 | entities変数の重複定義 |

### 🟠 P1: 機能不全（3件）

| 問題 | 影響 |
|------|------|
| `PowerGridChanged` イベント未登録 | 電力グリッド変更が通知されない |
| `PowerSystemPlugin` 登録未確認 | システム自体が動かない可能性 |
| `Entity::from_raw(wire.grid_id as u32)` | u64→u32キャスト、意図不明 |

### 🟡 P2: 仕様との乖離（3件）

| 仕様 | 実装 | 問題 |
|------|------|------|
| `Machine.fuel` 再利用 | `PowerFuelSlot` 新規作成 | 設計方針違反 |
| `is_powered: bool` | `current_power: f32` | セマンティック変更 |
| イベントは `game_events.rs` に集約 | `power.rs` にも定義 | 重複 |

### ⚪ P3: 未実装機能（4件）

- FR-005: 無電力時の機械停止ロジック
- FR-008: 電力状態の視覚表示
- FR-009: 統計表示UI
- FR-013: 電線ブロック配置ロジック

---

## 2. 「実装後の修正が多い」原因分析

### 根本原因

| 原因 | 説明 | 証拠 |
|------|------|------|
| **仕様→実装の翻訳ミス** | 仕様書は明確だが実装が従っていない | PowerFuelSlot方針違反 |
| **コピペバグ** | 同じロジックを2回書いている | grid_calc.rs:177-190 |
| **統合テストなし** | ビルドエラーがそのまま残っている | UI module構文エラー |
| **イベント登録漏れ** | 定義だけして登録を忘れた | GameEventsExtPlugin |

### ツール使用の問題点

```
[Speckit仕様書] → [SwamTool実装] → [レビューなし] → [バグ混入]
                                    ↑
                              ここが抜けている
```

**SwamToolの特性**:
- 仕様書を読んで実装するが、**仕様の「注記」や「禁止事項」を見落とす**
- 複数ファイルにまたがる変更で**整合性チェックが弱い**
- **ビルド確認せずにコード生成**している可能性

---

## 3. 品質改善のための推奨フロー

### 現状のフロー（問題あり）
```
仕様作成 → 一括実装 → 動かない → 修正 → 修正 → ...
```

### 推奨フロー
```
仕様作成 → フェーズ分割 → 実装+ビルド確認 → テスト → 次フェーズ
              ↓
         1. コンポーネント定義
         2. システム骨組み
         3. イベント統合
         4. UI実装
         （各フェーズでビルド確認）
```

### SwamTool使用時のチェックリスト

```markdown
□ 実装前: 仕様の「注記」「禁止事項」を明示的に伝えたか
□ 実装後: cargo build 通るか
□ 実装後: 新しいイベントは Plugin に登録されているか
□ 実装後: 新しい Plugin は game.rs に登録されているか
□ 実装後: 仕様の「完了条件」を満たしているか
```

---

## 4. 修正計画

### Phase 1: ビルド復旧（P0修正）
1. `src/setup/ui/mod.rs` の構文エラー2件修正
2. `generator_tick.rs` の括弧重複削除
3. `grid_calc.rs` の重複変数削除
4. `cargo build` 通過確認

### Phase 2: 機能修正（P1修正）
1. `PowerGridChanged` を `GameEventsExtPlugin` に登録
2. `PowerSystemPlugin` の登録確認/追加
3. `grid_calc.rs` のキャストロジック修正

### Phase 3: 設計統一（P2修正）
1. `PowerFuelSlot` 削除、`Machine.fuel` 再利用に変更
2. `PowerConsumer` の定義を仕様に合わせる
3. イベント定義を `game_events.rs` に一本化

### Phase 4: 機能完成（P3対応）
1. 無電力時の機械停止ロジック実装
2. 電力UI実装
3. 電線配置ロジック実装

---

## 5. 良かった点

| 項目 | 評価 |
|------|------|
| 仕様書の品質 | ✅ 高品質（spec.md, data-model.md, plan.md, research.md） |
| アルゴリズム選択 | ✅ Union-Findは適切 |
| 設定ファイル | ✅ items.toml, machines.toml は正確 |
| テスト（部分的） | ✅ power.rs に基本テストあり |

---

---

## 6. Web検索による対策調査

### 業界のベストプラクティス

#### 制約はAIを制限しない、精度を高める

> "Constraints don't limit AI agents — they enable precision."
> — [Forge Code](https://forgecode.dev/blog/ai-agent-best-practices/)

> "Agents perform best when constrained. Freedom makes them creative. Production wants predictable."

#### 曖昧なプロンプト = 間違った結果

> "Vague prompts mean wrong results." Be specific about inputs, outputs, and constraints.

❌ 悪い例: `"You are a helpful coding assistant"`
✅ 良い例: `"You are a test engineer who writes tests for React components, follows these examples, and never modifies source code"`

— [Augment Code](https://www.augmentcode.com/blog/best-practices-for-using-ai-coding-agents)

#### コンテキストウィンドウは40-60%が最適

> Anthropic's research reveals optimal AI agent performance at 40–60% context window utilization, not 100%. Performance actually degrades as you approach maximum capacity.

— [Forge Code](https://forgecode.dev/blog/ai-agent-best-practices/)

**示唆**: 仕様書が長すぎると逆効果。重要な制約は最初の40%に入れるべき。

#### Plan-Act-Reflect ワークフロー

> The Plan–Act–Reflect workflow helps your AI coding agent think before it codes.
> 1. Plan (propose a plan before writing code)
> 2. Act (after reviewing the plan, code step by step with small modular scope)
> 3. Reflect

— [Medium](https://medium.com/@elisheba.t.anderson/building-with-ai-coding-agents-best-practices-for-agent-workflows-be1d7095901b)

#### 人間のレビューは必須

> Golden rule: AI agents can propose code, never own it.

> "I won't commit code I couldn't explain to someone else."

— [Augment Code](https://www.augmentcode.com/blog/best-practices-for-using-ai-coding-agents)

### 良い仕様書の書き方

[Addy Osmani](https://addyosmani.com/blog/good-spec/)による仕様書ベストプラクティス:

1. **最終目標だけでなく、背景の理由、追加の制約も説明する**
2. **曖昧なプロンプトは必ず失敗する** - 複雑なタスクを数語で指示しようとしない
3. **プロジェクト固有のガイドラインを構成ファイルで提供する**

### 指示無視問題への対策（OWASP 2025）

[OWASP](https://www.oligo.security/academy/owasp-top-10-llm-updated-2025-examples-and-mitigation-strategies)による対策:

1. **モデルの役割を明確に定義** - 特定のタスクへの厳格な遵守
2. **システムプロンプトに明確な指示を含める**
3. **外部コンテンツを分離** - プロンプトへの影響を最小化
4. **定期的な敵対的テスト**
5. **セマンティックフィルターで入力を検証**

---

## 7. 指示無視の詳細分析

### 無視された指示（証拠付き）

#### 🔴 Case 1: PowerFuelSlot禁止指示の完全無視

**仕様書の指示（3箇所で明示）**:

```markdown
# data-model.md 行10
新規PowerFuelSlot Componentは実装しない

# data-model.md 行34
個別のComponentとしては実装しない

# plan.md 行11-12
既存のMachineSlots.fuelを活用し、新規PowerFuelSlot Componentは実装しない
```

**実装結果（power.rs 行17-27）**:
```rust
pub struct PowerFuelSlot {  // ← 禁止されているのに実装！
    pub fuel: Option<(ItemId, u32)>,
    pub consumption_rate: f32,
    pub startup_delay: f32,
    pub startup_timer: f32,
}
```

**無視率**: 100%（3回明示した指示が完全に無視された）

#### 🟠 Case 2: フィールド定義の変更

**仕様**:
```
is_powered: bool  // バイナリ状態
```

**実装**:
```rust
current_power: f32  // 段階的な値に変更
```

**無視率**: 部分的（意図的な設計変更の可能性）

---

### 原因分析

#### 原因 1: 否定指示の埋没（最有力）

```markdown
### PowerFuelSlot                    ← セクション名が目立つ

燃料ベース発電機の燃料スロット。
既存のMachineSlots.fuelを活用するため、
個別のComponentとしては実装しない。   ← 否定指示が説明文に埋もれている

| 追加フィールド | Type | 用途 |       ← テーブルが続く
```

**問題**: AIは「PowerFuelSlot」というセクション名を見て「実装すべきもの」と解釈。
否定指示は文章の中に埋もれており、テーブルが続くことで「これらを実装する」という印象が強まる。

#### 原因 2: 「禁止」< 「やること」バイアス

LLMの一般的傾向:
- ✅ 「Xを実装する」→ 高い遵守率
- ❌ 「Xを実装しない」→ 低い遵守率（見落としやすい）

これはプロンプトエンジニアリングの既知の問題。

#### 原因 3: コンテキスト長の問題

仕様書が複数ファイル（spec.md + data-model.md + plan.md = 400行以上）に分散。
AIが全文を正確に保持できず、「セクション名」だけ記憶して詳細を見落とす。

---

### 対策（仕様書フォーマット改善）

#### 対策 1: 禁止事項を最上部に独立セクションで配置

```markdown
# 禁止事項（MUST NOT）

- ❌ PowerFuelSlot Componentを新規作成しない（既存Machine.fuelを使う）
- ❌ is_powered を f32 に変更しない（bool のまま）

---

# 以下、実装詳細...
```

#### 対策 2: セクション名に「NOT」を含める

```markdown
### PowerFuelSlot - DO NOT IMPLEMENT

このセクションは参考情報のみ。実装禁止。
```

#### 対策 3: 禁止事項をチェックリスト化

```markdown
## 実装前チェックリスト

SwamTool/AIに渡す前に確認:
- [ ] PowerFuelSlot を作らない（Machine.fuel使用）
- [ ] is_powered は bool のまま
- [ ] イベント定義は game_events.rs のみ
```

#### 対策 4: フェーズ分割 + 各フェーズ後のレビュー

```
Phase 1: コンポーネント定義
         ↓ レビュー「PowerFuelSlot作ってないか？」
Phase 2: システム実装
         ↓ レビュー「ビルド通るか？」
Phase 3: 統合
```

---

## 8. 対策案（Web検索結果ベース）

### A. 仕様書フォーマット改善

```markdown
# 禁止事項（MUST NOT）- 最上部に配置

❌ PowerFuelSlot Componentを新規作成しない
  → 代替: 既存Machine.fuelを使用
  → 確認: `grep -r "PowerFuelSlot" src/` が0件

❌ is_powered を f32 に変更しない
  → 理由: バイナリ状態で十分
  → 確認: `grep "is_powered.*f32" src/` が0件
```

### B. コンテキスト最適化

仕様書を**コンテキストウィンドウの40-60%**に収める:
- 現状: spec.md + data-model.md + plan.md = 400行以上
- 改善: 重要な制約を1ファイル100行以内に凝縮

### C. Plan-Act-Reflectの強制

```
Phase 1: Plan（計画のみ、コード書かない）
         ↓ 人間レビュー
Phase 2: Act（1ファイルずつ実装）
         ↓ cargo build 確認
Phase 3: Reflect（仕様との差異チェック）
```

### D. 検証スクリプト

```bash
#!/bin/bash
# scripts/verify-power-spec.sh

echo "=== 禁止パターンチェック ==="
if grep -rq "PowerFuelSlot" src/; then
  echo "❌ FAIL: PowerFuelSlot が存在"
  exit 1
fi

if grep -rq "current_power.*f32" src/components/; then
  echo "❌ FAIL: current_power: f32 が存在（is_powered: bool であるべき）"
  exit 1
fi

echo "✅ PASS: 禁止パターンなし"
```

---

## 9. 今回の対応

### 維持するファイル

```
opencode.json          # GLM-4.7モデル設定（SwarmTools/OpenCode用）
AGENTS.md              # OpenCode用の指示ファイル
```

`opencode.json` の内容:
```json
{
  "model": "zai-coding-plan/glm-4.7",
  "instructions": ["AGENTS.md"]
}
```

### ログ保存

今回の結果を比較用に保存:

```bash
# 保存先
mkdir -p docs/experiments/2026-01-30-glm-power-system

# 保存するファイル
cp -r specs/001-power-system-spec/ docs/experiments/2026-01-30-glm-power-system/specs/
cp src/components/power.rs docs/experiments/2026-01-30-glm-power-system/
cp -r src/systems/power/ docs/experiments/2026-01-30-glm-power-system/systems-power/

# 変更されたファイルのdiffも保存
git diff mods/base/items.toml > docs/experiments/2026-01-30-glm-power-system/items.diff
git diff mods/base/machines.toml > docs/experiments/2026-01-30-glm-power-system/machines.diff
git diff src/events/game_events.rs > docs/experiments/2026-01-30-glm-power-system/game_events.diff
git diff src/game_spec/machines.rs > docs/experiments/2026-01-30-glm-power-system/machines_spec.diff
git diff src/setup/ui/mod.rs > docs/experiments/2026-01-30-glm-power-system/ui_mod.diff
git diff src/ui/machine_ui.rs > docs/experiments/2026-01-30-glm-power-system/machine_ui.diff

# レビュー結果も保存
cp /home/bacon/.claude/plans/smooth-questing-volcano.md docs/experiments/2026-01-30-glm-power-system/review.md
```

### 削除対象

```bash
# 新規作成されたファイル（削除）
rm -rf specs/001-power-system-spec/  # 仕様書
rm src/components/power.rs           # 実装
rm -rf src/systems/power/            # 実装

# 変更されたファイル（git restore）
git restore mods/base/items.toml
git restore mods/base/machines.toml
git restore src/events/game_events.rs
git restore src/game_spec/machines.rs
git restore src/setup/ui/mod.rs
git restore src/ui/machine_ui.rs
```

---

## 10. 次のステップ

1. **ログ保存**: 今回の結果を `docs/experiments/` に保存
2. **削除**: 仕様書・実装を削除
3. **仕様書テンプレート作成**: 禁止事項セクション付きの新テンプレート
4. **再実装**: 新しいフォーマットで仕様書を作り直し（実装はしない）
