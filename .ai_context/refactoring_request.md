# リファクタリング計画依頼

## 現在の状態
- 総コード: 12,811行
- 最大ファイル: command_ui.rs (826行)
- 残タスク: PlayerPlugin作成、InteractionPlugin作成、command_ui.rs分割

## 完了済み
- block_operations.rs 分割 ✅
- ui_setup.rs 分割 ✅  
- targeting.rs 分割 ✅
- MachineSystemsPlugin, UIPlugin, SavePlugin 作成 ✅

## main.rs の現状 (507行)
以下のシステムがまだmain.rsに残っている:
- player_look, player_move, toggle_cursor_lock, tick_action_timers
- block_break, block_place
- spawn_chunk_tasks, receive_chunk_meshes, unload_distant_chunks
- quest系システム

## 依頼
1. PlayerPlugin の設計案（どのシステム/リソースを移動すべきか）
2. InteractionPlugin の設計案
3. command_ui.rs 分割案（826行→どう分けるか）
4. 優先順位の提案

具体的なコード例があると助かります。
