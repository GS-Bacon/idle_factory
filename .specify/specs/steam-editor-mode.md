# Steam特別モード＆エディタ仕様

**ステータス**: 決定済み（実装待ち）

---

## 1. エディタの位置づけ

### コンセプト
- Minecraftのように外部からハックするのではなく、**最初から公式MOD対応**
- AI開発において、アニメーションやレシピツリーの指示が困難 → **視覚的エディタ**で解決

### 対象ユーザー
| ユーザー | 用途 |
|---------|------|
| 開発者 | 公式コンテンツ制作、Steam実績管理 |
| MOD作者 | アイテム・レシピ・クエスト追加 |
| 一般プレイヤー | 簡単なカスタマイズ |

---

## 2. MOD＆プロファイルシステム（決定）

### ディレクトリ構造

```
profiles/
  vanilla/                    ← 公式コンテンツ（Steam配布）
    profile.yaml
    data/
      items.yaml              ← エディタで直接編集
      recipes.yaml
      quests.yaml
      achievements.yaml       ← Steam実績（開発者モード）
    assets/
      icons/
      models/

  my_custom_mod/              ← ユーザー作成MOD
    profile.yaml
    data/
      items.yaml
      recipes.yaml

mods/                         ← サードパーティMOD（ダウンロード）
  steel_age/
    1.0.0/
      mod.yaml
      data/
    1.2.0/
      mod.yaml
      data/

config/
  active_profile.yaml         ← 起動時に読み込み

saves/
  slot_1/
    profile: vanilla
    world.dat
```

### 重要な区別
| 種類 | 場所 | 用途 |
|-----|------|------|
| profiles/ | ローカル編集可能 | エディタで直接編集 |
| mods/ | ダウンロード専用 | 外部MODをバージョン管理 |

### profile.yaml

```yaml
name: "Heavy Industry"
version: "1.0.0"
mods:
  - id: steel_age
    version: "1.2.0"
  - id: power_plus
    version: ">=1.0.0"
server_origin: null  # or "192.168.1.100:7777"
```

### active_profile.yaml

```yaml
profile: "industrial"
```

---

## 3. プロファイル切り替え（疑似ホットリロード）

### アーキテクチャ

```
┌────────────────────────────────────────────────────┐
│ Factory Data Architect (Tauri) ← フロントエンド     │
│                                                    │
│  [Editor Tab] [Play ▼]                             │
│               └─ vanilla                           │
│                  industrial ← 選択                  │
│                                                    │
│  ┌──────────────────────────────────────────────┐  │
│  │         Bevy Game (子プロセス)                │  │
│  │         profile: industrial                  │  │
│  └──────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────┘
```

### 切り替えフロー

```
[ユーザー: プロファイル変更]
         ↓
[Tauri: ゲームに終了シグナル送信]
         ↓
[Bevy: 状態保存 → 終了]
         ↓
┌─────────────────────────┐
│ ローディング画面        │
│ ████████░░░░ 60%        │
│ "industrial をロード中" │
└─────────────────────────┘
         ↓
[Tauri: 新プロファイルでゲーム起動]
         ↓
[Bevy: 起動完了 → Tauriに通知]
         ↓
[ローディング画面非表示]
```

### メリット
- ユーザーからは**ホットリロードに見える**
- 実際は完全再起動なので**MOD競合リスク低**
- ランチャー不要（Tauriエディタがラッパー）

---

## 3.5 プロファイル直接編集（エクスポート不要）

### 従来のワークフロー（廃止）
```
エディタで編集 → [📤 Export] → assets/data/
```

### 新ワークフロー
```
┌─────────────────────────────────────────────────────────┐
│ [Items] [Recipes] [Quests]    Target: [vanilla ▼] [▶]  │
│                                       └─ vanilla       │
│                                          my_mod        │
│                                          industrial    │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  編集内容は自動的にターゲットプロファイルに保存          │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### データフロー
```
[エディタで編集]
       ↓ 自動保存
profiles/{target}/data/
       ├── items.yaml
       ├── recipes.yaml
       └── quests.yaml
       ↓ ゲーム起動時
[Bevy: プロファイル読み込み]
```

### 開発者モード（Steam特別モード）
- ターゲット: `vanilla`（デフォルトプロファイル）
- `vanilla`は公式コンテンツとして配布
- Steam実績もvanillaプロファイル内で管理

### MODユーザーモード
- ターゲット: 任意のカスタムプロファイル
- 新規プロファイル作成可能
- vanillaを参照しつつ追加コンテンツを編集

### メリット
- **エクスポート操作が不要**
- **編集対象が明確**（どのプロファイルを編集中か表示）
- **即座にテスト可能**（▶ボタンで対象プロファイルを起動）
- **Steam版とMOD版で同じUI**（ターゲットが違うだけ）

---

## 4. エディタモード分離

| モード | 対象 | タブ |
|-------|------|------|
| 通常モード | MODユーザー/プレイヤー | Items, Recipes, Quests, Multiblock, Biome, Sounds |
| 開発者モード | 開発者のみ | 上記 + **Steam**, **Build** |

### 切り離し方式

```toml
# Cargo.toml
[features]
default = []
developer-mode = []  # 開発者モード有効化
```

または環境変数:
```
FACTORY_DEVELOPER_MODE=1
```

---

## 5. Steam特別モード（開発者モード専用）

### 対象機能
- Steam実績の定義・管理
- 統計定義
- ビルド設定

### 実績エディタUI案

```
┌─────────────────────────────────────────────────────────┐
│ [Items] [Recipes] [Quests] ... [🔧Steam]                │
├─────────────────────────────────────────────────────────┤
│ ┌──────────────┐  ┌─────────────────────────────────┐  │
│ │ 実績一覧     │  │ 実績エディタ                    │  │
│ │              │  │                                 │  │
│ │ ✓ first_step │  │ ID: first_machine               │  │
│ │ ○ producer   │  │ 名前(ja): 初めての一歩          │  │
│ │ ○ master     │  │ 名前(en): First Step            │  │
│ │              │  │ タイプ: [通常 ▼]                │  │
│ │ [+ 新規]     │  │ トリガー: [クエスト完了 ▼]      │  │
│ │              │  │   クエストID: tutorial_1        │  │
│ └──────────────┘  │ アイコン: [📁選択]              │  │
│                   └─────────────────────────────────┘  │
├─────────────────────────────────────────────────────────┤
│ [📤 Steamworksにエクスポート]  [✓ バリデーション]        │
└─────────────────────────────────────────────────────────┘
```

---

## 6. 実績システム仕様（決定）

### 6.1 実績の種類

| 種類 | 説明 | 用途例 |
|-----|------|-------|
| ✅ 通常実績 | 条件達成で即解除 | 初めての機械設置、チュートリアル完了 |
| ✅ プログレス実績 | 段階的に進捗表示（0-100%） | アイテム1000個生産、機械100台設置 |
| ✅ 隠し実績 | 解除まで内容非表示 | 隠しエリア発見、イースターエッグ |

### 6.2 トリガー条件

| トリガー | 説明 | パラメータ |
|---------|------|-----------|
| ✅ クエスト完了 | 特定クエストの完了 | `quest_id: String` |
| ✅ アイテム生産数 | 累計生産量 | `item_id: String, count: u64` |
| ✅ 機械設置数 | 累計設置数 | `machine_id: Option<String>, count: u32` |
| ✅ プレイ時間 | 累計プレイ時間 | `hours: u32` |
| ✅ フェーズ到達 | 特定フェーズへの到達 | `phase: u32` |
| ✅ カスタム条件 | 複合条件やスクリプト | `condition: String` |

### 6.3 エクスポート形式

**方針**: エディタ内部はJSON、Steamworksへはエクスポート時にVDF変換

```
[エディタで編集]
      ↓
  achievements.json  ← 内部形式
      ↓
[エクスポート機能]
      ↓
  achievements.vdf   ← Steamworks形式
```

**理由**:
- JSONはエディタで扱いやすい
- VDFはSteamworksパートナーサイトにアップロード可能
- 両対応で柔軟性を確保

### 6.4 実績データ構造

```yaml
# achievements.yaml
achievements:
  - id: first_machine
    type: normal
    i18n_key: achievement.first_machine
    icon: icons/first_machine.png
    hidden: false
    trigger:
      type: machine_placed
      count: 1

  - id: iron_producer
    type: progress
    i18n_key: achievement.iron_producer
    icon: icons/iron_producer.png
    hidden: false
    trigger:
      type: item_produced
      item_id: iron_ingot
      count: 1000
    progress_stat: iron_ingot_produced  # Steam統計と連携

  - id: secret_area
    type: normal
    i18n_key: achievement.secret_area
    icon: icons/secret.png
    hidden: true
    trigger:
      type: custom
      condition: "visited_secret_zone_alpha"
```

### 6.5 ゲーム内実績トラッカー

```rust
// 概念的な構造
pub struct AchievementTracker {
    pub achievements: HashMap<String, AchievementDef>,
    pub progress: HashMap<String, AchievementProgress>,
}

pub struct AchievementProgress {
    pub unlocked: bool,
    pub unlock_time: Option<DateTime>,
    pub current_value: u64,  // プログレス実績用
    pub target_value: u64,
}

// イベント
pub enum AchievementEvent {
    ProgressUpdated { id: String, current: u64, target: u64 },
    Unlocked { id: String },
}
```

---

## 7. 統計システム仕様（決定）

Steam実績と連携する統計システム:

### 7.1 追跡する統計

| 統計ID | 説明 | 型 |
|-------|------|---|
| `total_items_produced` | 総アイテム生産数 | INT |
| `total_machines_placed` | 総機械設置数 | INT |
| `total_playtime_seconds` | 総プレイ時間（秒） | INT |
| `current_phase` | 現在のフェーズ | INT |
| `{item_id}_produced` | アイテム別生産数 | INT |

### 7.2 統計の保存

```yaml
# stats.yaml（ローカル）
stats:
  total_items_produced: 12345
  total_machines_placed: 42
  total_playtime_seconds: 3600
  current_phase: 3
  iron_ingot_produced: 5000
  copper_ingot_produced: 3000
```

---

## 8. エディタUX原則 (E1-E6)

### E1. 即時プレビュー
- 編集内容は即座に反映（リアルタイム更新）
- 保存ボタン不要（自動保存）
- ゲーム内での見た目をプレビュー可能

### E2. 非破壊編集
- **Undo**: 100ステップ以上の履歴
- **履歴ブランチ**: 複数の編集パスを保持
- **元データ保持**: 元ファイルを破壊しない

### E3. 制約視覚化
- 有効/無効な配置領域を色で表示
- レシピ依存関係の可視化
- エラー箇所のハイライト

### E4. スマートデフォルト
- コンテキストに応じた初期値
- 直前の設定を記憶
- テンプレートからの複製

### E5. 一括操作
- 複数選択 → 同時編集
- 検索 → 一括置換
- フィルタリング → 一括適用

### E6. 参照整合性
- 削除時に使用箇所を警告
- 依存アイテムの自動検出
- 循環参照のチェック

### MOD APIバージョニング (M3)
- **SemVer**: Major.Minor.Patch
- **非推奨化**: 1バージョン猶予後に削除
- **マイグレーション**: 自動変換パス提供

---

## 9. 次のステップ

1. ✅ 実績の種類を決定
2. ✅ トリガー条件を決定
3. ✅ エクスポート形式を決定
4. **エディタUI実装**
   - 実績一覧パネル
   - 実績エディタフォーム
   - VDFエクスポート機能
5. **ゲーム側実装**
   - AchievementTrackerリソース
   - 統計トラッキングシステム
   - Steam API連携（bevy-steamworks）
6. **エディタUX改善** (E1-E6)
   - Undo/Redo履歴実装
   - 参照整合性チェック
   - 一括操作対応

---

*作成日: 2025-12-22*
*最終更新: 2025-12-22（エディタUX原則追加）*
