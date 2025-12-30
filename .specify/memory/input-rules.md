# 入力マトリクス（入力系変更時の必須確認）

入力系を変更するときは、このマトリクスを確認して全状態での動作を検討する。

## 状態一覧
| 状態 | リソース | 説明 |
|------|----------|------|
| 通常 | - | ゲームプレイ中 |
| Tutorial | `TutorialShown` | チュートリアル表示中 |
| Inventory | `InventoryOpen` | Eキーでインベントリ開いている |
| FurnaceUI | `InteractingFurnace` | 精錬炉UI開いている |
| CrusherUI | `InteractingCrusher` | 粉砕機UI開いている |
| Command | `CommandInputState` | T or /でコマンド入力中 |
| Paused | `CursorLockState.paused` | ESCでポーズ中 |

## 入力と状態の関係
| 入力 | 通常 | Tutorial | Inventory | FurnaceUI | CrusherUI | Command | Paused |
|------|------|----------|-----------|-----------|-----------|---------|--------|
| WASD/Space/Shift | 移動 | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Mouse Move | 視点 | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Left Click | 破壊 | 閉じる | スロット | スロット | スロット | ✗ | 復帰 |
| Right Click | 設置/UI | 閉じる | ✗ | - | - | ✗ | ✗ |
| Wheel | HB選択 | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| 1-9 | HB選択 | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| E | Inv開く | ✗ | 閉じる | 閉じる | 閉じる | ✗ | ✗ |
| ESC | ポーズ | ✗ | 閉じる | 閉じる | 閉じる | 閉じる | - |
| T or / | Cmd開く | ✗ | ✗ | ✗ | ✗ | 入力 | ✗ |
| Q | 報酬 | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| F3 | デバッグ | デバッグ | デバッグ | デバッグ | デバッグ | デバッグ | デバッグ |
| Any key | - | 閉じる | - | - | - | - | - |

✗ = 無効、- = 該当なし

## 入力ハンドラーと状態チェック

主要ハンドラーがチェックする状態:

| ハンドラー | チェックする状態 |
|------------|-----------------|
| player_move | Tutorial, Inventory, Furnace, Crusher, Command, Paused |
| player_look | Tutorial, Inventory, Furnace, Crusher, Command, Paused |
| block_break | Furnace, Inventory, Paused |
| block_place | Inventory, Paused |
| select_block_type | Command, Inventory |
| furnace_interact | Inventory, Command, Paused |
| crusher_interact | Inventory, Furnace, Command, Paused |
| inventory_toggle | Furnace, Crusher, Command, Paused |
| tutorial_dismiss | - (TutorialShown=falseの時のみ動作) |

## 入力系変更時のチェックリスト
- [ ] このマトリクスで全状態の動作を確認した
- [ ] 必要な状態チェックを追加した
- [ ] 関連するハンドラーを確認した（grep で同じキー/ボタンを処理している箇所）

## 変更前grepルール（必須）

入力系を変更する前に、同じキー/ボタンを処理している箇所を確認する。

```bash
# Eキーを変更する場合
grep -n "KeyCode::KeyE" src/main.rs

# 左クリックを変更する場合
grep -n "MouseButton::Left" src/main.rs

# 右クリックを変更する場合
grep -n "MouseButton::Right" src/main.rs

# マウスホイールを変更する場合
grep -n "MouseWheel\|mouse_wheel" src/main.rs
```

**理由**: 同じキーを複数のハンドラーが処理している場合、1つだけ変更すると不整合が起きる。
