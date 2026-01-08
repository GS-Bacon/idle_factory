# Mod仕様書

> **ステータス**: M2実装前の設計ドキュメント
> **最終更新**: 2026-01-08

## 概要

このゲームは3層のMod構造を持つ。Factorioレベルの拡張性を目指す。

---

## 1. Modの種類

| レイヤー | 形式 | 用途 | 状態 |
|---------|------|------|------|
| **Data Mod** | TOML | アイテム/機械/レシピ定義 | ✅ 実装済み |
| **Script Mod** | WebSocket | イベント監視、外部連携 | ✅ 基盤あり |
| **Core Mod** | WASM | 新ロジック追加 | ❌ M2で実装 |

---

## 2. Modでできること

| 機能 | Data | Script | Core |
|------|:----:|:------:|:----:|
| **コンテンツ追加** | | | |
| アイテム追加 | ✅ | - | ✅ |
| 機械追加（既存ロジック） | ✅ | - | ✅ |
| レシピ追加 | ✅ | - | ✅ |
| **状態アクセス** | | | |
| イベント監視 | - | ✅ | ✅ |
| 機械状態読み取り | - | ✅ | ✅ |
| 機械状態変更 | - | ❌ | ✅ |
| インベントリ操作 | - | ❌ | ✅ |
| **ワールド操作** | | | |
| ブロック読み取り | - | ❌ | ✅ |
| ブロック配置/削除 | - | ❌ | ✅ |
| ワールド生成ルール | - | - | ✅ |
| バイオーム追加 | ✅ | - | ✅ |
| **新システム** | | | |
| 新ロジック（電力等） | - | - | ✅ |
| ネットワーク計算 | - | - | ✅ |
| カスタムコンポーネント | - | - | ✅ |
| **UI** | | | |
| HUD追加 | - | - | ✅ |
| カスタムパネル | - | - | ✅ |
| **カメラ・描画** | | | |
| 監視カメラ | - | - | ✅ |
| レンダーテクスチャ | - | - | ✅ |
| **Entity・乗り物** | | | |
| Entity移動 | - | - | ✅ |
| 乗り物（乗車/下車） | - | - | ✅ |
| テレポート | - | - | ✅ |
| **環境** | | | |
| 昼夜サイクル | - | - | ✅ |
| 天候変更 | - | - | ✅ |
| **サウンド** | | | |
| 効果音再生 | - | - | ✅ |
| BGM再生 | - | - | ✅ |
| **Mob/NPC** | | | |
| Entity生成/削除 | - | - | ✅ |
| AI制御 | - | - | ✅ |

---

## 3. ホスト関数一覧（WASM API）

### 3.1 機械操作

```
get_machine_state(entity: u64) -> i32
set_machine_enabled(entity: u64, enabled: i32)
get_machine_progress(entity: u64) -> f32
set_machine_progress(entity: u64, progress: f32)
get_machine_slot(entity: u64, slot_id: u32) -> (u32, u32)  // item_id, count
add_to_slot(entity: u64, slot_id: u32, item_id: u32, count: u32) -> i32
take_from_slot(entity: u64, slot_id: u32, count: u32) -> i32
transfer_item(from: u64, to: u64, item_id: u32, count: u32) -> i32
```

### 3.2 ワールド操作

```
get_block(x: i32, y: i32, z: i32) -> u32
place_block(x: i32, y: i32, z: i32, block_id: u32) -> i32
remove_block(x: i32, y: i32, z: i32) -> i32
get_machine_at(x: i32, y: i32, z: i32) -> u64
get_conveyor_at(x: i32, y: i32, z: i32) -> u64
is_position_loaded(x: i32, y: i32, z: i32) -> i32
```

### 3.3 インベントリ操作

```
get_player_slot(player: u64, slot: u32) -> (u32, u32)
add_to_player(player: u64, item_id: u32, count: u32) -> i32
take_from_player(player: u64, item_id: u32, count: u32) -> i32
```

### 3.4 イベント

```
subscribe_event(event_type: u32) -> u32  // subscription_id
unsubscribe_event(subscription_id: u32)
emit_custom_event(name_ptr: u32, name_len: u32, data_ptr: u32, data_len: u32)
```

Mod側エクスポート:
```
mod_on_event(event_type: u32, data_ptr: u32, data_len: u32)
```

### 3.5 ネットワーク（電力/液体/信号用）

```
create_network(network_type: u32) -> u32  // network_id
network_add_node(network: u32, entity: u64)
network_remove_node(network: u32, entity: u64)
network_connect(network: u32, node_a: u64, node_b: u64)
network_disconnect(network: u32, node_a: u64, node_b: u64)
network_get_neighbors(network: u32, node: u64, out_ptr: u32) -> u32  // count
network_find_connected(network: u32, start: u64, out_ptr: u32) -> u32
```

### 3.6 カスタムコンポーネント

```
set_entity_data(entity: u64, key_ptr: u32, key_len: u32, data_ptr: u32, data_len: u32)
get_entity_data(entity: u64, key_ptr: u32, key_len: u32, out_ptr: u32) -> i32
has_entity_data(entity: u64, key_ptr: u32, key_len: u32) -> i32
remove_entity_data(entity: u64, key_ptr: u32, key_len: u32)
```

### 3.7 UI

```
create_hud_element(type: u32, config_ptr: u32, config_len: u32) -> u32
update_hud_element(element_id: u32, data_ptr: u32, data_len: u32)
remove_hud_element(element_id: u32)
create_panel(config_ptr: u32, config_len: u32) -> u32
add_panel_widget(panel: u32, widget_config_ptr: u32, widget_config_len: u32) -> u32
update_widget(widget_id: u32, data_ptr: u32, data_len: u32)
show_panel(panel_id: u32)
hide_panel(panel_id: u32)
```

### 3.8 ワールド生成

```
register_biome(config_ptr: u32, config_len: u32) -> u32
register_ore_rule(biome: u32, ore_config_ptr: u32, ore_config_len: u32)
register_structure(config_ptr: u32, config_len: u32) -> u32
```

### 3.9 レジストリ

```
register_item(config_ptr: u32, config_len: u32) -> u32
register_machine(config_ptr: u32, config_len: u32) -> u32
register_recipe(config_ptr: u32, config_len: u32) -> u32
get_item_info(item_id: u32, out_ptr: u32) -> i32
get_machine_info(machine_id: u32, out_ptr: u32) -> i32
```

### 3.10 ユーティリティ

```
get_tick() -> u64
get_delta_time() -> f32
log_info(msg_ptr: u32, msg_len: u32)
log_warn(msg_ptr: u32, msg_len: u32)
log_error(msg_ptr: u32, msg_len: u32)
storage_set(key_ptr: u32, key_len: u32, value_ptr: u32, value_len: u32)
storage_get(key_ptr: u32, key_len: u32, out_ptr: u32) -> i32
```

### 3.11 カメラ・描画

```
create_camera(x: f32, y: f32, z: f32, rx: f32, ry: f32, rz: f32) -> u32
set_camera_position(camera_id: u32, x: f32, y: f32, z: f32)
set_camera_rotation(camera_id: u32, rx: f32, ry: f32, rz: f32)
get_camera_texture(camera_id: u32) -> u32  // texture_id
destroy_camera(camera_id: u32)
create_image_widget(panel: u32, texture_id: u32) -> u32
```

### 3.12 Entity移動・乗り物

```
move_entity(entity: u64, x: f32, y: f32, z: f32)
set_entity_velocity(entity: u64, vx: f32, vy: f32, vz: f32)
get_entity_position(entity: u64) -> (f32, f32, f32)
get_entity_velocity(entity: u64) -> (f32, f32, f32)
attach_player_to_entity(player: u64, entity: u64)
detach_player(player: u64)
```

### 3.13 プレイヤー制御

```
teleport_player(player: u64, x: f32, y: f32, z: f32)
get_player_position(player: u64) -> (f32, f32, f32)
get_player_rotation(player: u64) -> (f32, f32)
set_player_rotation(player: u64, rx: f32, ry: f32)
```

### 3.14 環境制御

```
set_time_of_day(hour: f32)
get_time_of_day() -> f32
set_weather(weather_type: u32)
get_weather() -> u32
```

### 3.15 サウンド

```
play_sound(sound_id: u32, x: f32, y: f32, z: f32, volume: f32) -> u32
play_music(music_id: u32, loop_flag: i32) -> u32
stop_sound(handle: u32)
register_sound(path_ptr: u32, path_len: u32) -> u32
```

### 3.16 Entity/Mob生成

```
spawn_entity(type_ptr: u32, type_len: u32, x: f32, y: f32, z: f32) -> u64
despawn_entity(entity: u64)
set_entity_ai(entity: u64, ai_type: u32, config_ptr: u32, config_len: u32)
get_entities_in_range(x: f32, y: f32, z: f32, radius: f32, out_ptr: u32) -> u32
```

---

## 4. 禁止事項（セキュリティ）

| 禁止 | 理由 |
|------|------|
| ファイルシステムアクセス | セキュリティ |
| ネットワーク直接アクセス | セキュリティ |
| 任意メモリアクセス | WASMサンドボックス |
| シェルコマンド実行 | セキュリティ |
| 他Modのメモリ直接アクセス | 隔離 |

---

## 5. 制限事項（パフォーマンス）

| 項目 | 上限 |
|------|------|
| ホスト関数呼び出し | 10,000回/tick |
| メモリ使用量 | 64MB |
| 実行時間 | 5ms/tick |
| ブロック変更 | 1,000個/tick |
| Entity変更 | 1,000個/tick |

---

## 6. Mod必須エクスポート

```rust
// 初期化・終了
fn mod_init() -> i32          // 0=OK, 負数=エラー
fn mod_cleanup()

// メインループ
fn mod_tick(tick: u64)

// イベントハンドラ
fn mod_on_event(event_type: u32, data_ptr: u32, data_len: u32)

// メモリ管理
fn mod_alloc(size: u32) -> u32
fn mod_free(ptr: u32, size: u32)
```

---

## 7. サンプルMod案

| Mod | 内容 | 使用API |
|-----|------|---------|
| 電力システム | 発電機→電線→機械 | 3.1, 3.5, 3.7 |
| 液体システム | パイプ→タンク→機械 | 3.1, 3.5, 3.7 |
| 信号システム | センサー→ワイヤー→ON/OFF | 3.1, 3.5 |
| 建設ロボット | ブループリント自動建設 | 3.2, 3.16 |
| 新バイオーム | 砂漠、雪原、火山 | 3.8 |
| 監視カメラ | 工場の遠隔監視 | 3.7, 3.11 |
| 列車・乗り物 | 長距離輸送 | 3.12, 3.16 |
| テレポーター | 拠点間ワープ | 3.13 |
| 天候システム | 雨で機械効率変化 | 3.1, 3.14 |
| 敵Mob | 工場を襲う敵とタレット | 3.15, 3.16 |

---

## 8. 実装フェーズ

| フェーズ | API | 内容 |
|---------|-----|------|
| **M2初期** | 3.1〜3.6, 3.9〜3.10 | 機械、ワールド、インベントリ、イベント、ネットワーク、レジストリ、ユーティリティ |
| **M2後半** | 3.7〜3.8 | UI、ワールド生成 |
| **M3以降** | 3.11〜3.16 | カメラ、乗り物、プレイヤー、環境、サウンド、Mob |

**方針**: 最小限で始めて、必要になったら追加

---

## 9. API安定性ガイドライン

> **注意**: 製品版まで公式Modのみ対応。後方互換性は気にしない。

| 方針 | 詳細 |
|------|------|
| 破壊的変更 | ✅ OK（公式Modは同時更新） |
| 関数削除 | ✅ OK |
| シグネチャ変更 | ✅ OK |
| バージョニング | 不要 |

製品版リリース時にAPI凍結を検討。

---

## 10. エラーハンドリング

### エラーコード規約

| コード | 意味 |
|--------|------|
| 0 | 成功 |
| -1 | 一般エラー |
| -2 | Entity not found |
| -3 | Position not loaded |
| -4 | Permission denied |
| -5 | Rate limit exceeded |
| -6 | Invalid parameter |

### 動作規約

| 状況 | 動作 |
|------|------|
| 存在しない関数を呼ぶ | エラーコード返却（クラッシュしない） |
| 無効なEntity ID | エラーコード返却 |
| 範囲外の座標 | エラーコード返却 |
| 制限超過 | エラーコード返却 + ログ警告 |
| Modがパニック | **そのModだけ停止、ゲーム継続** |

---

## 11. ディレクトリ構造

```
mods/
├── base/                  # Data Mod（TOML）- バニラコンテンツ
│   ├── items.toml
│   ├── machines.toml
│   └── recipes.toml
└── sample_power_mod/      # Core Mod（WASM）- 電力システム例
    ├── mod.toml           # Mod定義（依存関係等）
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs
    └── build.sh
```

### mod.toml形式

```toml
[mod]
id = "power_system"
name = "Power System"
version = "1.0.0"
api_version = 1

[dependencies]
base = ">=1.0.0"
```

---

*最終更新: 2026-01-08*
