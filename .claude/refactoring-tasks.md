# リファクタリングタスクリスト

## 概要

2026-01-01 アーキテクチャレビュー結果に基づくタスクリスト。
総合評価: **7/10** - 良い基盤だが、スケーリングに課題あり。

## 🔴 優先度: 高（次の機能追加前に対応）

### 1. block_operations.rs の分割（1,001行 → 3ファイル）

**現状**: UIとロジックが混在、1000行超過でルール違反

**分割案**:
```
src/systems/
├── block_operations/
│   ├── mod.rs           # pub use 再エクスポート
│   ├── placement.rs     # ブロック設置ロジック (~300行)
│   ├── breaking.rs      # ブロック破壊ロジック (~400行)
│   └── utils.rs         # 共通ユーティリティ (~300行)
```

**作業内容**:
- [ ] ディレクトリ `src/systems/block_operations/` 作成
- [ ] ブロック設置関連を `placement.rs` に移動
- [ ] ブロック破壊関連を `breaking.rs` に移動
- [ ] 共通関数（レイキャスト等）を `utils.rs` に移動
- [ ] `mod.rs` で再エクスポート
- [ ] main.rs のインポート更新
- [ ] テスト確認

**推定作業時間**: 1-1.5時間

---

### 2. ui_setup.rs の分割（977行 → 3ファイル）

**現状**: 全UI生成が1ファイルに集中

**分割案**:
```
src/setup/
├── ui/
│   ├── mod.rs           # pub use 再エクスポート
│   ├── base.rs          # 基本レイアウト (~200行)
│   ├── machine_ui.rs    # 機械UI生成 (~400行)
│   └── inventory_ui.rs  # インベントリUI (~300行)
```

**作業内容**:
- [ ] ディレクトリ `src/setup/ui/` 作成
- [ ] 機械UI（Furnace, Crusher, Miner）を `machine_ui.rs` に移動
- [ ] インベントリ・ホットバーUIを `inventory_ui.rs` に移動
- [ ] 基本レイアウトを `base.rs` に残す
- [ ] `mod.rs` で再エクスポート
- [ ] main.rs のインポート更新
- [ ] テスト確認

**推定作業時間**: 1.5時間

---

### 3. MachineSystemsPlugin の作成

**現状**: main.rs に40+の `add_systems` 呼び出しが散在

**新規ファイル**:
```
src/plugins/
├── mod.rs
└── machines.rs          # MachineSystemsPlugin
```

**作業内容**:
- [ ] `src/plugins/` ディレクトリ作成
- [ ] `MachineSystemsPlugin` 実装
  - furnace.rs のシステム登録
  - crusher.rs のシステム登録
  - miner.rs のシステム登録
  - conveyor.rs のシステム登録
- [ ] 関連リソースの init_resource をプラグインに移動
- [ ] main.rs を `.add_plugins(MachineSystemsPlugin)` に変更
- [ ] テスト確認

**推定作業時間**: 1時間

---

## 🟡 優先度: 中（3機能以内に対応）

### 4. InputContext リソースの統合

**現状**: 6つの独立したリソースが入力状態を管理
```rust
init_resource::<InventoryOpen>()
init_resource::<InteractingFurnace>()
init_resource::<InteractingCrusher>()
init_resource::<InteractingMiner>()
init_resource::<CursorLockState>()
init_resource::<CommandBarState>()
```

**提案**:
```rust
#[derive(Resource, Default)]
pub struct InputContext {
    pub allows_movement: bool,
    pub allows_block_actions: bool,
    pub allows_camera: bool,
    pub current_ui: Option<ActiveUI>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ActiveUI {
    Inventory,
    Furnace(Entity),
    Crusher(Entity),
    Miner(Entity),
    CommandBar,
}
```

**作業内容**:
- [ ] `InputContext` リソース定義
- [ ] `ActiveUI` enum 定義
- [ ] フレーム開始時に `InputContext` を計算するシステム追加
- [ ] 各システムを `InputContext` 参照に変更
- [ ] 古いリソースを削除
- [ ] テスト確認

**推定作業時間**: 2時間

---

### 5. ワイルドカードインポートの削除

**現状**: `use components::*` で79定義をインポート

**作業内容**:
- [ ] main.rs の `use components::*` を明示的インポートに変更
- [ ] 他ファイルのワイルドカードを確認・修正
- [ ] コンパイル確認

**推定作業時間**: 30分

---

### 6. targeting.rs の分割（759行）

**分割案**:
```
src/systems/
├── targeting/
│   ├── mod.rs
│   ├── raycast.rs       # レイキャスト処理
│   ├── highlight.rs     # ハイライト表示
│   └── guide_marker.rs  # ガイドマーカー
```

**推定作業時間**: 1時間

---

### 7. machines/ モジュールの整理

**現状**: architecture.md と実装が不一致
- `machines/mod.rs` は18行のプレースホルダ
- 実際の定義は `components/machines.rs`（636行）

**選択肢**:
A) architecture.md を現状に合わせて更新
B) 実装を architecture.md に合わせて再構成

**推奨**: B（長期的にスケールしやすい）

**作業内容**:
- [ ] `components/machines.rs` を `machines/` に移動
- [ ] 機械ごとにファイル分割
- [ ] インポート更新
- [ ] テスト確認

**推定作業時間**: 1.5時間

---

## 🟢 優先度: 低（MVP後に対応）

### 8. UIPlugin の作成

**作業内容**:
- [ ] `src/plugins/ui.rs` 作成
- [ ] UI関連システムをプラグインに移動
- [ ] UI関連リソースをプラグインに移動

---

### 9. SavePlugin の作成

**作業内容**:
- [ ] `src/plugins/save.rs` 作成
- [ ] save_systems.rs のシステムをプラグインに移動

---

### 10. システム順序の明示化

**作業内容**:
- [ ] 依存関係のあるシステムに `.before()/.after()` 追加
- [ ] コメントで順序の理由を記載

---

### 11. テスト追加

**不足しているテスト**:
- [ ] 機械チェーン（Miner → Conveyor → Furnace）
- [ ] Save/Load の状態保持
- [ ] Block break → item drop → pickup
- [ ] クエスト完了フロー
- [ ] クリエイティブモード切替

---

## 🟠 優先度: バグ修正・クリーンアップ

### 12. unwrap() の安全化

**作業内容**:
- [ ] vox_loader.rs:290 - unwrap()をOptionハンドリングに修正
- [ ] block_operations.rs:450 - unwrap()をOptionハンドリングに修正
- [ ] その他のunwrap()を調査・修正（72箇所）

**推定作業時間**: 2-3時間

---

### 13. 未使用コード削除

**作業内容**:
- [ ] Direction::opposite()
- [ ] Conveyor::calculate_shape*
- [ ] CreativeViewMode
- [ ] CreativeCategory

**推定作業時間**: 30分

---

## 🔵 優先度: テスト・品質向上

### 14. 自動バグ検出

**Lv1: Fuzzing**:
- [ ] ランダム入力でクラッシュ検出スクリプト作成

**Lv2: シナリオテスト**:
- [ ] 採掘機→コンベア→精錬炉→インゴット確認
- [ ] e2e_state.jsonに機械状態・インベントリ内容を追加
- [ ] assert関数追加（assert_inventory_contains, assert_machine_working等）

**推定作業時間**: 3-4時間

---

### 15. E2E改善（ゲーム内コマンド）

**作業内容**:
- [ ] /testコマンド追加（ゲーム内テストシナリオ実行）
- [ ] /assertコマンド追加（インベントリ・機械状態の検証）
- [ ] /spawn_lineコマンド追加（機械ラインを一発配置）

**推定作業時間**: 2時間

---

### 16. ログ改善

**作業内容**:
- [ ] ゲームイベントログ追加（機械動作、プレイヤー操作、アイテム移動）
- [ ] 構造化ログ形式（JSON）に変更
- [ ] チャンク生成ログのノイズ削減（DEBUGレベルに下げる）

**推定作業時間**: 1.5時間

---

### 17. ログ活用（AI連携）

**作業内容**:
- [ ] ログサマリー生成スクリプト（AIで自然言語要約）
- [ ] 異常検出ルール追加（機械停止、コンベア詰まり等）

**推定作業時間**: 2時間

---

### 18. コンベア左右逆問題

**作業内容**:
- [ ] /debug_conveyorコマンド追加（pos, direction, shape, inputs, outputs, items出力）
- [ ] L字配置自動テスト（左曲がり/右曲がりのアイテム座標追跡）
- [ ] 「左」の定義統一（コード/モデル/アニメーションで同じ意味に）

**推定作業時間**: 2時間

---

## 🟣 優先度: 将来（v0.2以降）

### 19. リプレイシステム
- [ ] 操作記録・巻き戻し・早送り対応

### 20. ブループリント
- [ ] 範囲選択保存・ペースト・エクスポート/インポート

### 21. 電力システム
- [ ] 発電機・電線・電力消費・過負荷制御

### 22. 流体パイプ
- [ ] ポンプ・パイプ・タンク・流量計算

### 23. マルチプレイ基盤
- [ ] WebSocket同期・プレイヤー位置・ブロック操作同期

### 24. Modding API
- [ ] Lua/WASM埋め込み・イベントフック

### 25. ビジュアルプログラミング
- [ ] ノードグラフで機械の振る舞い定義

---

## 進捗トラッキング

| # | タスク | 優先度 | 状態 | 完了日 |
|---|--------|--------|------|--------|
| 1 | block_operations分割 | 🔴高 | ⬜ | - |
| 2 | ui_setup分割 | 🔴高 | ⬜ | - |
| 3 | MachineSystemsPlugin | 🔴高 | ⬜ | - |
| 4 | InputContext統合 | 🟡中 | ⬜ | - |
| 5 | ワイルドカード削除 | 🟡中 | ⬜ | - |
| 6 | targeting分割 | 🟡中 | ⬜ | - |
| 7 | machines/整理 | 🟡中 | ⬜ | - |
| 8 | UIPlugin作成 | 🟢低 | ⬜ | - |
| 9 | SavePlugin作成 | 🟢低 | ⬜ | - |
| 10 | システム順序明示化 | 🟢低 | ⬜ | - |
| 11 | テスト追加 | 🟢低 | ⬜ | - |
| 12 | unwrap()安全化 | 🟠バグ | ⬜ | - |
| 13 | 未使用コード削除 | 🟠バグ | ⬜ | - |
| 14 | 自動バグ検出 | 🔵テスト | ⬜ | - |
| 15 | E2E改善 | 🔵テスト | ⬜ | - |
| 16 | ログ改善 | 🔵テスト | ⬜ | - |
| 17 | ログ活用 | 🔵テスト | ⬜ | - |
| 18 | コンベア左右問題 | 🔵テスト | ⬜ | - |
| 19 | CI/CD改善 | 🟢インフラ | ⬜ | - |
| 20 | Cargo.toml最適化 | 🟢インフラ | ⬜ | - |
| 21 | Clippy警告修正 | 🟢品質 | ⬜ | - |
| 22 | フォント最適化 | 🟢最適化 | ⬜ | - |
| 23 | WASMサイズ最適化 | 🟢最適化 | ⬜ | - |
| 24 | バイナリサイズ削減 | 🟢最適化 | ⬜ | - |
| 25 | 依存関係棚卸し | 🟢調査 | ⬜ | - |

---

## 🟢 優先度: インフラ・ツール（並列作業向き）

### 19. CI/CD改善

**対象ファイル**: `.github/workflows/`（競合リスクなし）

**作業内容**:
- [ ] clippy警告をエラー扱いに（`-D warnings`）
- [ ] WASMビルドの自動化
- [ ] リリースタグ時の自動バイナリ生成
- [ ] PRごとのテスト実行

**推定作業時間**: 1時間

---

### 20. Cargo.toml最適化

**対象ファイル**: `Cargo.toml`（競合リスク低）

**作業内容**:
- [ ] 不要な依存削除の調査
- [ ] feature flagsの整理
- [ ] dev-dependencies の最適化

**推定作業時間**: 30分

---

### 21. Clippy警告修正

**対象ファイル**: 複数（警告箇所による）

**現状の警告（8件）**:
- [ ] `meshes` and `materials` fields never read
- [ ] `load_all_vox_models` is never used
- [ ] `load_vox_recursive` is never used
- [ ] function has too many arguments (14/7)
- [ ] very complex type（2件）
- [ ] `impl` can be derived
- [ ] `map_or` can be simplified

**推定作業時間**: 30分

---

### 22. フォント最適化

**対象ファイル**: `assets/fonts/`（競合リスクなし）

**現状**:
- assets/fonts/: 16MB（全アセットの94%）
- NotoSansJP が重い

**作業内容**:
- [ ] 使用文字のみのサブセット化
- [ ] 軽量フォントへの置き換え検討
- [ ] WOFF2圧縮

**推定作業時間**: 1時間

---

### 23. WASMサイズ最適化

**対象ファイル**: `Cargo.toml`（プロファイル設定）

**現状**:
- WASM: 37MB（圧縮後7.5MB）

**作業内容**:
- [ ] wasm-opt による最適化
- [ ] LTO設定の見直し
- [ ] panic=abort の検討
- [ ] 不要feature の除外

**推定作業時間**: 1時間

---

### 24. バイナリサイズ削減

**現状**:
- debug: 243MB
- release: 未計測

**作業内容**:
- [ ] release ビルドサイズ計測
- [ ] strip symbols
- [ ] codegen-units = 1 の検討

**推定作業時間**: 30分

---

### 25. 依存関係棚卸し（調査のみ）

**作業内容**:
- [ ] 409個の依存の必要性調査
- [ ] ライセンス一覧生成（cargo-license）
- [ ] 重複依存の確認

**推定作業時間**: 1時間

---

## 参考: 推定総作業時間

| 優先度 | 合計時間 |
|--------|----------|
| 🔴 高（リファクタリング） | 3.5-4時間 |
| 🟡 中（リファクタリング） | 5時間 |
| 🟢 低（リファクタリング） | 未定 |
| 🟠 バグ修正 | 2.5-3.5時間 |
| 🔵 テスト・品質向上 | 10.5-13.5時間 |
| 🟢 インフラ・ツール | 1.5時間 |
| 🟢 品質・最適化 | 4時間 |
| 🟣 将来（v0.2以降） | 未定 |

**推奨**: 🔴高優先度のタスク1-3を一気に実施（半日作業）

**並列作業向き**: 🟢インフラ・最適化（#19-25）は他タスクと競合しにくい
