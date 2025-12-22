# Steam販売準備レポート

## 概要

本レポートは、Infinite Voxel FactoryをSteamで販売するために必要な機能と準備事項を調査・整理したものです。

---

## 1. Steamworks SDK統合

### 1.1 推奨ライブラリ

| クレート | 説明 | バージョン |
|---------|------|-----------|
| [bevy-steamworks](https://crates.io/crates/bevy-steamworks) | Bevy用Steamworks統合プラグイン | 0.15 |
| [steamworks](https://docs.rs/steamworks) | 基盤となるRust Steamworksバインディング | 自動同梱 |

```toml
# Cargo.tomlに追加
[dependencies]
bevy-steamworks = { version = "0.15", optional = true }

[features]
default = []
steam = ["bevy-steamworks"]
```

### 1.2 初期化コード

```rust
use bevy::prelude::*;
use bevy_steamworks::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SteamworksPlugin::new(AppId(YOUR_APP_ID)))
        .run();
}
```

**注意**: Steam未インストール環境でもゲームが起動できるよう、featureフラグで分離すること。

---

## 2. 実装が必要なSteam機能

### 2.1 実績システム（Achievements）

**優先度**: 高

| 必要な作業 | 説明 |
|-----------|------|
| 実績定義 | Steamworksパートナーサイトで実績を定義 |
| 実績トラッキング | ゲーム内進捗を監視するシステム |
| 実績解除ロジック | 条件達成時にSteam APIを呼び出し |
| UIオーバーレイ対応 | 実績解除通知の表示 |

**推奨実績例**:
- 初めての機械設置
- 初めてのクエスト完了
- 1000個のアイテム生産
- 全レシピ解放
- フェーズ完了（Phase 1-5）

**現状の基盤**:
- ✅ クエストシステム（`src/gameplay/quest.rs`）が存在
- ❌ 実績専用のトラッキングシステムなし

### 2.2 クラウドセーブ（Steam Cloud）

**優先度**: 高

| 方式 | 説明 | 推奨度 |
|-----|------|-------|
| Steam Auto-Cloud | コード変更不要、ファイルパス指定のみ | ⭐推奨 |
| Steam Cloud API | 細かい制御が可能 | 高度なケース向け |

**現状**:
- ✅ セーブシステム存在（`src/core/save_system.rs`）
- ✅ JSON形式でローカル保存
- ❌ Steam Cloud連携なし

**対応方法**:
Steamworksパートナーサイトで以下を設定:
```
saves/slot_*/metadata.json
saves/slot_*/world.dat
```

### 2.3 リーダーボード（Leaderboards）

**優先度**: 中

工場建設ゲームに適したリーダーボード:
- 総生産量
- 工場効率（アイテム/分）
- クエスト完了時間
- 使用機械数

**現状**:
- ❌ 統計追跡システムなし
- 新規実装が必要

### 2.4 リッチプレゼンス（Rich Presence）

**優先度**: 中

Steamフレンドリストに表示される詳細ステータス:
- 「Phase 3をプレイ中」
- 「鉄インゴットを生産中」
- 「クエスト: 初めての自動化」

### 2.5 Steam Overlay対応

**優先度**: 高（自動）

- Steamオーバーレイ表示中のゲーム一時停止
- Shift+Tab対応

```rust
// オーバーレイ表示中かチェック
fn check_overlay(client: Res<Client>) {
    if client.utils().is_overlay_enabled() {
        // オーバーレイ対応処理
    }
}
```

---

## 3. Steam Deck対応

### 3.1 互換性要件

| 要件 | 現状 | 対応 |
|-----|------|-----|
| 解像度 1280x800/720 | ❓ 未確認 | ウィンドウリサイズ対応必要 |
| コントローラー完全対応 | ❌ キーボード専用 | 大幅な作業必要 |
| オンスクリーンキーボード | ❌ 未対応 | Steamworks API使用 |
| テキスト入力対応 | ✅ ワールド名入力あり | Steam API連携必要 |

### 3.2 コントローラー対応（大規模作業）

**現状の入力システム** (`src/core/input.rs`):
- キーボードのみ対応
- ゲームパッド対応なし

**必要な作業**:
1. `Gamepad` リソースの追加
2. スティック入力→カメラ/移動変換
3. ボタンマッピング（A=決定、B=キャンセル等）
4. UI操作のコントローラー対応
5. アイコン切り替え（キーボード/コントローラー）

### 3.3 Linux/SteamOS ビルド

Bevy は Steam Deck（Linux）をネイティブサポート:
- 安定して90 FPS動作の報告あり
- ただしライブラリバージョンに注意

**推奨**: [bevy_steamos_docker](https://github.com/paul-hansen/bevy_steamos_docker) でビルド

---

## 4. ストア要件

### 4.1 必須素材

| 素材 | サイズ | 備考 |
|-----|-------|-----|
| カプセル画像（横長） | 460x215 | ストアページメイン |
| カプセル画像（縦長） | 374x448 | ライブラリ表示 |
| ヘッダー画像 | 460x215 | ストアページ上部 |
| ライブラリカプセル | 600x900 | Steamライブラリ |
| スクリーンショット | 1920x1080推奨 | 最低5枚 |
| トレーラー動画 | 1080p推奨 | 1本以上推奨 |

### 4.2 Coming Soonページ

- リリース2週間前までに公開必須
- ウィッシュリスト獲得のため早期公開推奨

### 4.3 レビュープロセス

1. ストアページレビュー（先に提出）
2. ゲームビルドレビュー（後で提出）
3. 審査期間：数日〜1週間

---

## 5. 現プロジェクトの対応状況

### 5.1 既存機能の活用

| 機能 | ファイル | Steam機能との連携 |
|-----|---------|------------------|
| クエストシステム | `quest.rs` | 実績トリガーに活用可能 |
| セーブシステム | `save_system.rs` | クラウドセーブ基盤として利用 |
| プレイヤーステータス | `player_stats.rs` | 統計トラッキングに活用可能 |
| ホットリロード | `hot_reload.rs` | 開発中のみ、本番では無効化 |

### 5.2 新規実装が必要

| 機能 | 優先度 | 工数見積 |
|-----|-------|---------|
| 実績システム | 高 | 中 |
| 統計トラッキング | 高 | 中 |
| コントローラー対応 | 中（Deck対応時は高） | 大 |
| リーダーボード | 低 | 小 |
| リッチプレゼンス | 低 | 小 |

---

## 6. 推奨実装順序

### Phase A: 基盤整備（必須）

1. **Steam feature flag追加**
   - `bevy-steamworks`をoptional依存に追加
   - Steam未インストール環境での起動確保

2. **実績システム実装**
   - `AchievementEvent` イベント追加
   - `AchievementTracker` リソース追加
   - クエスト完了時に実績チェック

3. **統計システム実装**
   - `GameStatistics` リソース追加
   - 生産数、プレイ時間等を追跡

### Phase B: ストア準備

4. **素材作成**
   - スクリーンショット撮影
   - カプセル画像作成
   - トレーラー制作

5. **ストアページ作成**
   - Coming Soonページ公開
   - ウィッシュリスト獲得開始

### Phase C: 拡張機能（オプション）

6. **コントローラー対応**
   - Steam Deck「Verified」取得に必要

7. **リーダーボード実装**
8. **リッチプレゼンス実装**

---

## 7. コスト

| 項目 | 費用 |
|-----|------|
| Steamworks登録料 | $100（1回） |
| App ID取得 | 無料（登録後） |
| 年間費用 | なし |

---

## 8. 参考資料

- [Steamworks公式ドキュメント](https://partner.steamgames.com/doc/home)
- [bevy-steamworks GitHub](https://github.com/HouraiTeahouse/bevy_steamworks)
- [Steam Deck互換性要件](https://partner.steamgames.com/doc/steamdeck/compat)
- [実績ガイド](https://partner.steamgames.com/doc/features/achievements/ach_guide)
- [クラウドセーブガイド](https://kb.heathen.group/steam/features/cloud-save)
- [リーダーボードドキュメント](https://partner.steamgames.com/doc/features/leaderboards)

---

## 9. 次のアクション

1. [ ] Steamworksパートナーアカウント作成
2. [ ] App ID取得
3. [ ] `bevy-steamworks` をプロジェクトに追加（optional）
4. [ ] 実績定義の設計
5. [ ] `AchievementTracker` システム実装
6. [ ] 統計トラッキングシステム実装
7. [ ] ストア素材の準備開始

---

*作成日: 2025-12-22*
*最終更新: 2025-12-22*
