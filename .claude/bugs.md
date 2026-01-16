# よくあるバグと対策

**重要**: 新機能実装時、このセクションのパターンに該当する場合は対応するテストも追加すること。

## バグ修正手順

**重要**: バグ修正タスクを受けたら、まずログを解析してから修正に着手する。
**絶対禁止**: 憶測での修正。「多分これが原因」で修正してはいけない。

### 1. ログ取得

**ネイティブ版**:
```bash
./run.sh                    # ゲーム起動
./show-logs.sh              # 最新ログ
./show-logs.sh errors       # エラーのみ
./show-logs.sh events       # BLOCK, MACHINE, QUEST
```

**WASM版**:
```bash
node capture-wasm-logs.js   # 30秒キャプチャ
./show-logs.sh wasm
```

### 2. 画面検証

```bash
./verify-native.sh          # スクリーンショット撮影
# screenshots/verify/native_*.png を確認
```

## バグパターン一覧

### 1. 地面が透ける（黒い穴）
- **原因**: メッシュのワインディング順序が間違っている
- **検出**: `cargo test test_mesh_winding_order`
- **対策**: 面定義の頂点順序を修正

### 2. 機械設置時に地面が透ける
- **原因**: 機械設置時に`set_block()`で偽のブロックを登録
- **対策**: 機械はエンティティなので`set_block()`を呼ばない

### 3. エンティティ破壊時に子エンティティが残る
- **原因**: 親削除時に子を削除していない
- **対策**: 破壊時に関連エンティティもdespawn

### 4. チャンク境界でブロックが消える
- **原因**: 隣接チャンク情報なしでメッシュ生成
- **対策**: `generate_mesh_with_neighbors`で隣接情報を渡す

### 5. ブロック操作時のフリーズ
- **原因**: チャンク再生成パターンが不統一
- **対策**: 同じ再生成パターンを使用

### 6. レイキャスト判定漏れ
- **原因**: 新機械追加時にレイキャスト判定を追加し忘れ
- **対策**: 新機械追加時は必ず判定を更新

### 7. 機械破壊時にアイテム消失
- **原因**: 破壊時にスロット内容をインベントリに返却していない
- **対策**: despawn前に全スロットを返却

### 8. モード別UI表示制御漏れ
- **原因**: CreativeModeチェックなしで常に表示
- **対策**: マーカーコンポーネント追加、モードチェック

### 9. UI表示中のポインターロック
- **原因**: canvasクリック時にUI状態をチェックせず
- **対策**: `isGameUIOpen()`をチェック

### 10. UI表示中に操作が効く
- **原因**: システムがInventoryOpenをチェックしていない
- **対策**: InputState.allows_*()でチェック

### 11. プレビューと実際の動作が異なる
- **原因**: プレビューと実処理で異なるロジック
- **対策**: 同じ関数を呼び出す

### 12. ESCでUI閉じた後にポインターロック解除
- **原因**: ブラウザはESCでポインターロック解除する
- **対策**: JS側で自動再ロック（50ms後）

### 13. チャンク処理で長時間フリーズ
- **原因**: 複数チャンクが同時に完了
- **対策**: 1フレームの処理数を制限（MAX_CHUNKS_PER_FRAME=2）

### 14. チャンク再読み込みで変更リセット
- **原因**: チャンクアンロード時にデータ削除
- **対策**: modified_blocksで変更を永続化

### 15. カーソル制御の競合（複数システム問題）
- **原因**: 複数のシステムが同時にカーソル制御、実行順序で競合
- **検出**: `./scripts/cursor-log.sh --filter`で`release_cursor`直後に`lock_cursor`
- **対策**: PostUpdateで実行する専用システム`sync_cursor_to_ui_state`で一元管理
- **参考**: [Bevy Cheatbook](https://bevy-cheatbook.github.io/window/mouse-grab.html)

## 実装時チェックリスト

- [ ] メッシュ変更 → ワインディング順序テスト
- [ ] 機械追加 → レイキャスト判定、破壊時クリーンアップ、アイテム返却
- [ ] 子エンティティ持ち → 破壊時に子もdespawn
- [ ] チャンク操作 → 境界テスト
- [ ] モード専用UI → モードチェック、マーカー追加
- [ ] UI追加 → `set_ui_open_state`呼び出し
- [ ] ESCで閉じるUI → JS側で自動再ロック確認
- [ ] 毎フレーム処理 → バッチ処理に制限
- [ ] カーソル制御 → **絶対に直接制御しない**。UIStateを変更すれば`sync_cursor_to_ui_state`が自動制御

## 修正済みバグ（インベントリUI関連）

### BUG-UI-1: スプライト画像が小さくて見づらい
- **状態**: ✅ 修正済み（コミット89ed225）
- **詳細**: アイテムスロット内のスプライト画像サイズが小さすぎる
- **対策**: SPRITE_SIZE を 42px → 52px に変更

### BUG-UI-2: クリエイティブカタログのアイコンがスプライト未適用
- **状態**: ✅ 修正済み（コミット89ed225）
- **詳細**: クリエイティブカタログ（上部）のアイテムアイコンにスプライトが適用されていない
- **対策**: `update_creative_catalog_sprites` システム追加

### BUG-UI-3: アイテムスロットにアイテムの色が表示されている
- **状態**: ✅ 修正済み（コミット89ed225）
- **詳細**: スロット背景にアイテムの色が表示され、見た目がおかしい
- **対策**: スプライトがある場合は SLOT_BG 固定色を使用

### BUG-UI-4: ドラッグ中にアイテムが表示されない
- **状態**: ✅ 修正済み（コミット89ed225）
- **詳細**: ドラッグ&ドロップ中にアイテムが見えない
- **対策**: HeldItemImage コンポーネント追加、`update_held_item_display` でスプライト表示

## テスト追加ルール

| 変更タイプ | 必須テスト |
|-----------|-----------|
| 新機能 | 正常系 + エッジケース |
| バグ修正 | 再発防止テスト |
| リファクタリング | 既存テストでOK |

---

## 作業中バグ

### BUG-16: LocalPlayerとPlayerInventory未初期化
**状態**: ✅ 修正済み

**症状**:
- ホットバー選択（数字キー1-9）が機能しない
- テスト入力注入（Hotbar1-9等）が失敗する
- セーブ/ロードが正しく動作しない可能性

**原因**:
`src/setup/player.rs`の`setup_player()`でプレイヤーエンティティは作成されるが:
- `LocalPlayer`リソースが設定されていない
- `PlayerInventory`コンポーネントが追加されていない

**影響範囲**:
- `select_block_type()` - ホットバー選択
- `update_hotbar_ui()` - ホットバーUI更新
- `handle_load_event()` - セーブデータロード
- すべてのインベントリ関連システム

**対策**:
```rust
// setup_player() で以下を追加
let player_entity = commands.spawn((
    Player,
    PlayerPhysics::default(),
    PlayerInventory::default(),  // 追加
    Transform::from_xyz(8.0, 12.0, 20.0),
    Visibility::default(),
)).id();
commands.insert_resource(LocalPlayer(player_entity));  // 追加
```

**テストファイル**: `tests/scenarios/hotbar_select.toml`

---

### BUG-10: Windows起動時カーソル制御問題
**状態**: ✅ 修正済み

**症状**: 起動時にいきなりカーソルを吸収してしまう

**期待動作**: 一時停止メニューから始め、Resumeでゲーム開始

**原因**:
- `UIState::default()` がスタック空（Gameplay状態）で初期化されていた
- `CursorLockState::default()` が `paused: false` で初期化されていた

**対策**:
- `src/components/ui_state.rs`: `default()` が `vec![UIContext::PauseMenu]` で初期化するように変更
- `src/components/player.rs`: `CursorLockState::default()` が `paused: true` で初期化するように変更
- テスト用に `UIState::new_empty()` メソッドを追加

**テストファイル**: `tests/scenarios/startup_pause_menu.toml`

---

### BUG-17: カーソル制御の競合（複数システム問題）
**状態**: ✅ 修正済み

**症状**: ESCやEでUI開いてもカーソルが表示されない（Windowsで顕著）

**原因**:
- 複数のシステムが同時にカーソル制御していた
- `update_pause_ui`、`update_inventory_visibility`、`machines/generic.rs`等
- 実行順序が不定で、`release_cursor()`直後に`lock_cursor()`が呼ばれる競合

**ログでの確認方法**:
```bash
./scripts/cursor-log.sh --filter
# release_cursor直後にlock_cursorが呼ばれていたら競合
```

**対策（ベストプラクティス）**:
1. **PostUpdate**で実行される専用システム`sync_cursor_to_ui_state`を作成
2. UIStateを**唯一の真実のソース**として使用
3. 他のシステムからカーソル直接制御を削除

**変更ファイル**:
- `src/systems/cursor.rs` - `sync_cursor_to_ui_state`追加
- `src/plugins/game.rs` - PostUpdateで登録
- `src/systems/player.rs` - `update_pause_ui`からカーソル制御削除

**テストファイル**: `tests/scenarios/bug_esc_cursor_release.toml`, `tests/scenarios/ui_cursor_lock.toml`

**参考**: [Bevy Cheatbook - Mouse Grab](https://bevy-cheatbook.github.io/window/mouse-grab.html)

---

### BUG-11: EキーでインベントリとポーズメニューUIが両方表示
**状態**: テスト作成済み、追加検証不要かも

**症状**: Eキーを押すとインベントリと一時停止メニューが同時に表示される

**調査結果**:
- `ui_inventory_handler` はPauseMenu中にEキーを無視するコードがある（正常）
- テスト `tests/scenarios/pause_inventory_exclusive.toml` でPauseMenu中にEキーが効かないことを確認済み
- Windows固有の問題か、BUG-10が原因でUI状態が乱れている可能性あり

**テストファイル**: `tests/scenarios/pause_inventory_exclusive.toml`

**残作業**:
1. BUG-10修正後にWindowsで再確認

---

## 修正済みバグ詳細

### BUG-15: コンベアcorner_left/corner_rightモデルの左右逆問題（再発3回）

**症状**: 左に曲がるはずのコンベアが右に曲がるモデルを表示

**根本原因**: `tools/voxel_generator.py` でモデル形状の定義が逆だった

| 関数名 | 誤った定義 | 正しい定義 |
|--------|-----------|-----------|
| `create_conveyor_corner_left()` | 右へ曲がる形状 | **左**へ曲がる形状 |
| `create_conveyor_corner_right()` | 左へ曲がる形状 | **右**へ曲がる形状 |

**なぜ再発したか**:
1. 過去の修正が「ロジック側でマッピングを入れ替え」と「モデル側で形状を入れ替え」の両方で行われた
2. 一方を修正すると他方との整合性が崩れた
3. docstringと実際のコードが一致していなかった

**正しい対応**:
- `voxel_generator.py` のモデル生成コード自体を修正
- モデル名 = シェイプ名 = 曲がる方向 という一貫性を維持
- `get_conveyor_model()` でのマッピング入れ替えは**禁止**

**確認方法**:
```bash
# モデル再生成後
python3 tools/voxel_generator.py corner_left
python3 tools/voxel_generator.py corner_right
DISPLAY=:10 blender --background --python tools/vox_to_gltf.py -- assets/models/machines/conveyor/corner_left.vox assets/models/machines/conveyor/corner_left.glb
DISPLAY=:10 blender --background --python tools/vox_to_gltf.py -- assets/models/machines/conveyor/corner_right.vox assets/models/machines/conveyor/corner_right.glb
```

**関連テスト**: `test_conveyor_corner_left_direction`, `test_conveyor_corner_right_direction`, `test_corner_left_all_directions`, `test_corner_right_all_directions`
