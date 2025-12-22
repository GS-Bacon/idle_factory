# MOD API Design Skill

MODシステムの設計・実装を支援するスキルです。

## 参照ドキュメント

- `.specify/specs/mod-development-best-practices.md`

---

## Factorioモデル（3段階データライフサイクル）

```
1. Settings Stage
   ├── MOD設定の読み込み
   └── グローバル設定の確定

2. Data Stage
   ├── プロトタイプ定義（ブロック、アイテム、レシピ）
   ├── データ修正
   └── 最終データ確定

3. Runtime Stage
   ├── ゲームロジック実行
   ├── イベント処理
   └── スクリプト実行
```

---

## MODマニフェスト

```yaml
# mod.yaml
name: "awesome-machines"
version: "1.0.0"
title: "Awesome Machines"
author: "ModAuthor"
description: "Adds powerful new machines"

# 依存関係
dependencies:
  - name: "base"
    version: ">=0.1.0"
  - name: "power-plus"
    version: "^2.0.0"
    optional: true

# API互換性
api_version: "1.0"

# ロード順序
load_order: 100  # デフォルト: 1000

# 許可されるAPI
permissions:
  - "read_world"
  - "modify_entities"
  - "register_items"
```

---

## APIバウンダリ

### 許可するAPI

```rust
// MODが使用可能なAPI
pub trait ModApi {
    // データ登録
    fn register_block(&mut self, block: BlockDefinition);
    fn register_item(&mut self, item: ItemDefinition);
    fn register_recipe(&mut self, recipe: RecipeDefinition);

    // データ読み取り
    fn get_block(&self, id: &str) -> Option<&BlockDefinition>;
    fn get_item(&self, id: &str) -> Option<&ItemDefinition>;

    // イベント登録
    fn on_event<E: Event>(&mut self, handler: impl Fn(&E));

    // ワールド読み取り
    fn query_entities(&self, filter: EntityFilter) -> Vec<EntityRef>;
}
```

### 禁止するAPI

```rust
// MODから隠蔽するAPI
// - ファイルシステム直接アクセス
// - ネットワーク直接アクセス
// - メモリ直接操作
// - 他MODの内部状態
// - セーブデータ直接操作
```

---

## サンドボックス

### Lua サンドボックス

```rust
fn create_sandbox() -> Lua {
    let lua = Lua::new();

    // 危険な関数を無効化
    lua.globals().set("os", Value::Nil)?;
    lua.globals().set("io", Value::Nil)?;
    lua.globals().set("load", Value::Nil)?;
    lua.globals().set("loadfile", Value::Nil)?;
    lua.globals().set("require", Value::Nil)?;
    lua.globals().set("dofile", Value::Nil)?;

    // 安全なAPIを提供
    let api = lua.create_table()?;
    api.set("register_block", lua.create_function(register_block)?)?;
    api.set("register_item", lua.create_function(register_item)?)?;
    lua.globals().set("game", api)?;

    lua
}
```

### リソース制限

```rust
struct ModResourceLimits {
    max_memory: usize,          // 64MB
    max_cpu_time_per_tick: u64, // 1ms
    max_entities: usize,        // 10000
    max_registered_items: usize, // 1000
}

fn check_limits(mod_id: &str, usage: &ResourceUsage) -> Result<(), LimitError> {
    if usage.memory > LIMITS.max_memory {
        return Err(LimitError::MemoryExceeded(mod_id.to_string()));
    }
    if usage.cpu_time > LIMITS.max_cpu_time_per_tick {
        return Err(LimitError::CpuTimeExceeded(mod_id.to_string()));
    }
    Ok(())
}
```

---

## イベントシステム

```rust
#[derive(Event)]
pub struct OnBlockPlaced {
    pub block_id: String,
    pub position: IVec3,
    pub player: Option<PlayerId>,
}

#[derive(Event)]
pub struct OnMachineOutput {
    pub machine: Entity,
    pub item: ItemStack,
}

// MODからの登録
// game.on_event("on_block_placed", function(event)
//     if event.block_id == "my_mod:special_block" then
//         -- カスタム処理
//     end
// end)
```

---

## バージョニング

### セマンティックバージョニング

```
Major.Minor.Patch

Major: 破壊的変更
Minor: 後方互換の機能追加
Patch: バグ修正
```

### 非推奨化プロセス

```rust
// v1.0
pub fn old_function() { }

// v1.1
#[deprecated(since = "1.1.0", note = "Use new_function instead")]
pub fn old_function() { }
pub fn new_function() { }

// v2.0
pub fn new_function() { }
// old_function は削除
```

### マイグレーション

```rust
fn migrate_mod_data(data: ModData, from: Version, to: Version) -> ModData {
    let mut data = data;

    if from < Version::new(1, 1, 0) {
        data = migrate_1_0_to_1_1(data);
    }
    if from < Version::new(2, 0, 0) {
        data = migrate_1_x_to_2_0(data);
    }

    data
}
```

---

## MODロード順序

```rust
fn resolve_load_order(mods: &[ModManifest]) -> Vec<ModId> {
    let mut graph = DependencyGraph::new();

    for mod_manifest in mods {
        graph.add_node(mod_manifest.id.clone());

        for dep in &mod_manifest.dependencies {
            if !dep.optional {
                graph.add_edge(&dep.name, &mod_manifest.id);
            }
        }
    }

    // トポロジカルソート
    graph.topological_sort()
}
```

---

## マルチプレイヤー対応

### 同期要件

```rust
// 全クライアントで同一MODが必要
fn validate_mods(server_mods: &[ModInfo], client_mods: &[ModInfo]) -> Result<(), ModMismatch> {
    for server_mod in server_mods {
        let client_mod = client_mods.iter()
            .find(|m| m.id == server_mod.id);

        match client_mod {
            None => return Err(ModMismatch::Missing(server_mod.id.clone())),
            Some(m) if m.version != server_mod.version => {
                return Err(ModMismatch::VersionMismatch(server_mod.id.clone()));
            }
            _ => {}
        }
    }
    Ok(())
}
```

### MOD設定同期

```rust
// サーバー側のMOD設定が権威
fn sync_mod_settings(server_settings: &ModSettings) {
    apply_settings(server_settings);
}
```

---

## チェックリスト

### API設計

- [ ] 明確なバウンダリがあるか
- [ ] 危険な操作が禁止されているか
- [ ] イベントシステムがあるか
- [ ] ドキュメントがあるか

### サンドボックス

- [ ] ファイルアクセスが制限されているか
- [ ] ネットワークアクセスが制限されているか
- [ ] リソース制限があるか

### バージョニング

- [ ] セマンティックバージョニングを使用しているか
- [ ] 非推奨化プロセスがあるか
- [ ] マイグレーションパスがあるか

### 互換性

- [ ] 依存関係解決があるか
- [ ] MODロード順序が決定的か
- [ ] マルチプレイヤー同期があるか

---

*このスキルはMOD APIの品質を確保するためのガイドです。*
