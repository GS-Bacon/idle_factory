# WASM Mod API リファレンス

> **注意**: このAPIは開発中。製品版までは破壊的変更あり。

## 1. 機械操作

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

## 2. ワールド操作

```
get_block(x: i32, y: i32, z: i32) -> u32
place_block(x: i32, y: i32, z: i32, block_id: u32) -> i32
remove_block(x: i32, y: i32, z: i32) -> i32
get_machine_at(x: i32, y: i32, z: i32) -> u64
get_conveyor_at(x: i32, y: i32, z: i32) -> u64
is_position_loaded(x: i32, y: i32, z: i32) -> i32
```

## 3. インベントリ操作

```
get_player_slot(player: u64, slot: u32) -> (u32, u32)
add_to_player(player: u64, item_id: u32, count: u32) -> i32
take_from_player(player: u64, item_id: u32, count: u32) -> i32
```

## 4. イベント

```
subscribe_event(event_type: u32) -> u32  // subscription_id
unsubscribe_event(subscription_id: u32)
emit_custom_event(name_ptr: u32, name_len: u32, data_ptr: u32, data_len: u32)
```

Mod側エクスポート:
```
mod_on_event(event_type: u32, data_ptr: u32, data_len: u32)
```

## 5. ネットワーク（電力/液体/信号用）

```
create_network(network_type: u32) -> u32  // network_id
network_add_node(network: u32, entity: u64)
network_remove_node(network: u32, entity: u64)
network_connect(network: u32, node_a: u64, node_b: u64)
network_disconnect(network: u32, node_a: u64, node_b: u64)
network_get_neighbors(network: u32, node: u64, out_ptr: u32) -> u32  // count
network_find_connected(network: u32, start: u64, out_ptr: u32) -> u32
```

## 6. カスタムコンポーネント

```
set_entity_data(entity: u64, key_ptr: u32, key_len: u32, data_ptr: u32, data_len: u32)
get_entity_data(entity: u64, key_ptr: u32, key_len: u32, out_ptr: u32) -> i32
has_entity_data(entity: u64, key_ptr: u32, key_len: u32) -> i32
remove_entity_data(entity: u64, key_ptr: u32, key_len: u32)
```

## 7. UI

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

## 8. ワールド生成

```
register_biome(config_ptr: u32, config_len: u32) -> u32
register_ore_rule(biome: u32, ore_config_ptr: u32, ore_config_len: u32)
register_structure(config_ptr: u32, config_len: u32) -> u32
```

## 9. レジストリ

```
register_item(config_ptr: u32, config_len: u32) -> u32
register_machine(config_ptr: u32, config_len: u32) -> u32
register_recipe(config_ptr: u32, config_len: u32) -> u32
get_item_info(item_id: u32, out_ptr: u32) -> i32
get_machine_info(machine_id: u32, out_ptr: u32) -> i32
```

## 10. ユーティリティ

```
get_tick() -> u64
get_delta_time() -> f32
log_info(msg_ptr: u32, msg_len: u32)
log_warn(msg_ptr: u32, msg_len: u32)
log_error(msg_ptr: u32, msg_len: u32)
storage_set(key_ptr: u32, key_len: u32, value_ptr: u32, value_len: u32)
storage_get(key_ptr: u32, key_len: u32, out_ptr: u32) -> i32
```

## 11. カメラ・描画

```
create_camera(x: f32, y: f32, z: f32, rx: f32, ry: f32, rz: f32) -> u32
set_camera_position(camera_id: u32, x: f32, y: f32, z: f32)
set_camera_rotation(camera_id: u32, rx: f32, ry: f32, rz: f32)
get_camera_texture(camera_id: u32) -> u32  // texture_id
destroy_camera(camera_id: u32)
create_image_widget(panel: u32, texture_id: u32) -> u32
```

## 12. Entity移動・乗り物

```
move_entity(entity: u64, x: f32, y: f32, z: f32)
set_entity_velocity(entity: u64, vx: f32, vy: f32, vz: f32)
get_entity_position(entity: u64) -> (f32, f32, f32)
get_entity_velocity(entity: u64) -> (f32, f32, f32)
attach_player_to_entity(player: u64, entity: u64)
detach_player(player: u64)
```

## 13. プレイヤー制御

```
teleport_player(player: u64, x: f32, y: f32, z: f32)
get_player_position(player: u64) -> (f32, f32, f32)
get_player_rotation(player: u64) -> (f32, f32)
set_player_rotation(player: u64, rx: f32, ry: f32)
```

## 14. 環境制御

```
set_time_of_day(hour: f32)
get_time_of_day() -> f32
set_weather(weather_type: u32)
get_weather() -> u32
```

## 15. サウンド

```
play_sound(sound_id: u32, x: f32, y: f32, z: f32, volume: f32) -> u32
play_music(music_id: u32, loop_flag: i32) -> u32
stop_sound(handle: u32)
register_sound(path_ptr: u32, path_len: u32) -> u32
```

## 16. Entity/Mob生成

```
spawn_entity(type_ptr: u32, type_len: u32, x: f32, y: f32, z: f32) -> u64
despawn_entity(entity: u64)
set_entity_ai(entity: u64, ai_type: u32, config_ptr: u32, config_len: u32)
get_entities_in_range(x: f32, y: f32, z: f32, radius: f32, out_ptr: u32) -> u32
```

---

## エラーコード

| コード | 意味 |
|--------|------|
| 0 | 成功 |
| -1 | 一般エラー |
| -2 | Entity not found |
| -3 | Position not loaded |
| -4 | Permission denied |
| -5 | Rate limit exceeded |
| -6 | Invalid parameter |

## 制限事項

| 項目 | 上限 |
|------|------|
| ホスト関数呼び出し | 10,000回/tick |
| メモリ使用量 | 64MB |
| 実行時間 | 5ms/tick |
| ブロック変更 | 1,000個/tick |
| Entity変更 | 1,000個/tick |

---

## Mod必須エクスポート

```rust
fn mod_init() -> i32          // 0=OK, 負数=エラー
fn mod_cleanup()
fn mod_tick(tick: u64)
fn mod_on_event(event_type: u32, data_ptr: u32, data_len: u32)
fn mod_alloc(size: u32) -> u32
fn mod_free(ptr: u32, size: u32)
```

---

*最終更新: 2026-01-30*
