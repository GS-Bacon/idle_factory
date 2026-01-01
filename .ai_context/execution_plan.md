# リファクタリング実行計画

**作成日**: 2026-01-01
**連携**: Claude + Gemini (最新コンテキスト使用)

---

## レビュー結果サマリー

### 評価スコア

| 観点 | Claude | Gemini |
|------|--------|--------|
| アーキテクチャ | 8/10 | 7/10 |
| コード品質 | 8/10 | 7/10 |
| Bevyパターン | 8/10 | 8/10 |
| ツール・CI | - | 10/10 |
| **総合** | **8.0/10** | **7.5/10** |

### 共通の良い点

1. **Plugin分割進捗** - MachineSystemsPlugin, UIPlugin, SavePlugin作成済み
2. **モジュール分割完了** - 1000行超ファイルがゼロに
3. **unwrap()削減** - 72箇所→10箇所
4. **周辺ツール充実** - scripts/, E2Eテスト自動化

---

## 優先度別タスク

### 🔴 高優先度（今回実行）

#### 1. command_ui.rs 分割（826行→3ファイル）

**Gemini提案の分割案**:
```
src/systems/command/
├── mod.rs          # 再エクスポート
├── ui.rs           # UI層: トグル、描画更新
├── executor.rs     # ロジック層: パース、イベント発火
└── handlers.rs     # システム層: イベントハンドラ
```

**分割対象**:
- `ui.rs`: command_input_toggle, command_input_handler, keycode_to_char
- `executor.rs`: execute_command, parse_item_name
- `handlers.rs`: handle_teleport_event, handle_look_event, 他全ハンドラ

### 🟡 中優先度（将来検討）

#### 2. world/mod.rs 分割（627行）
- chunk.rs: チャンク管理
- access.rs: ブロックアクセス
- mesh.rs: メッシュ生成連携

#### 3. inventory_ui.rs 整理（649行）
- UI構築とロジックの分離

#### 4. InputContext統合
- 6つの独立リソース → 1つのInputContextに統合
- InventoryOpen, InteractingFurnace, InteractingCrusher等

### 🟢 低優先度（見送り）

#### 見送り項目と理由

| 項目 | Gemini判断 | 理由 |
|------|-----------|------|
| PlayerPlugin作成 | **不要** | main.rs 219行は十分コンパクト |
| WorldPlugin作成 | **不要** | 同上 |
| InteractionPlugin作成 | **不要** | 同上 |
| SystemSet導入 | **不要** | 現状の.after()で問題なし |
| main.rs追加Plugin化 | 400-500行超えたら検討 | 現状219行 |

---

## Claude vs Gemini 見解比較

### 一致点
- command_ui.rs分割が最優先
- プロジェクトは健全な状態
- ツール・自動化が充実

### 相違点

| 項目 | Claude初期提案 | Gemini最新判断 | 採用 |
|------|---------------|---------------|------|
| PlayerPlugin | 作成推奨 | 不要 | **Gemini** |
| InteractionPlugin | 作成推奨 | 不要 | **Gemini** |
| main.rsダイエット | 必要 | 219行で十分 | **Gemini** |
| command_ui分割 | 2-3ファイル | 3ファイル | **Gemini** |

**採用理由**: Geminiは最新のmain.rs行数（219行）を正確に把握した上で判断

---

## 実行計画

### Phase 1: command_ui.rs 分割（今回実行）

**作業ステップ**:
1. `src/systems/command/` ディレクトリ作成
2. `handlers.rs` 作成 - 全イベントハンドラを移動
3. `executor.rs` 作成 - execute_command, parse_item_name移動
4. `ui.rs` 作成 - UI制御システム移動
5. `mod.rs` で再エクスポート
6. `src/systems/mod.rs` のインポート更新
7. main.rsのインポート更新
8. テスト実行 (`cargo test && cargo clippy`)

**期待効果**:
- 826行 → 3ファイル（各約250-300行）
- デバッグ機能の見通し改善
- E2Eテスト関連コードの整理

### Phase 2: 完了後レビュー

- Geminiに分割結果をレビュー依頼
- 追加の改善提案があれば検討

---

## 更新予定（CLAUDE.md）

実行完了後、タスクリストを更新:
```markdown
| command_ui.rs 分割 | ✅ | 826行→3ファイル |
```

---

## 承認待ち

**実行対象**: Phase 1（command_ui.rs分割）

「実行」と言っていただければ開始します。
