# プロジェクトロードマップ: Infinite Voxel Factory (仮)

## 1. ゲーム概要
Factorio、Satisfactory、Minecraft(Create Mod)の影響を受けた、3Dボクセル工場シミュレーションゲーム。敵やサバイバル要素（HP/空腹）は存在せず、純粋な「自動化」と「建築」を楽しむ。

## 2. 技術的制約・仕様
- **視点:** 3D一人称視点（ボクセル世界）
- **物理演算:**
  - アイテム搬送には物理演算を使用しない（軽量化のためデータ上の移動のみ）。
  - プレイヤーや動く機械にはアニメーションが必要。
- **言語/環境:** Rust / Bevy Engine
- **特徴:** 大量のアイテムがコンベア上を流れる（物理演算なし）、広大なボクセル世界
- **プレイヤー:** ジェットパックで飛行可能。

## 3. コアシステム要件
- **資源:** 採掘機から無限に湧く。枯渇なし。資源ノードは特定の場所に配置。
- **物流:** コンベア、アーム、パイプ、ドラム缶輸送。
- **動力:** 「回転速度」と「応力（ストレス）」の概念を持つ運動エネルギーシステム。負荷が許容量を超えると停止・故障する。
- **クラフト:**
  - Tier1: 手元クラフト
  - Tier2: 単体機械加工
  - Tier3: マルチブロック設備
- **ロジック:** 終盤はPython/Lua等のスクリプトを用いた、論理ゲート制御や不良品選別などの高度な自動化が必要。
- **建築:** ブループリント機能、建築ロボット、全ブロック空中設置可能。
- **ゴール:** 巨大な納品塔へ物資を送り、上空の宇宙ステーションを視覚的に完成させる。
- **アイテムレシピ:** 全開放。
- **マップ:** 無限生成。

---

## プロジェクトフェーズと機能リスト

### Phase 1: コアエンジンとMod基盤 (The Foundation)
目標: 何もない空間に、外部ファイルで定義されたブロックを表示し、マルチプレイの同期基盤を作る。

#### データ駆動アーキテクチャ
- [x] Asset Server拡張: assets/ フォルダ内の変更を監視し、ゲーム中に動的にリロードする機能（ホットリロード）。 **(完了)**
- [x] YAML Loader: serde_yaml を使用し、アイテム・ブロック・レシピ定義を読み込むパーサー。 **(完了)**
- [x] Dynamic Texture Atlas: フォルダにある複数のPNG画像を読み込み、実行時に1枚の巨大なテクスチャ（Atlas）に結合する処理。 **(完了)**

#### レンデリング (Voxel)
- [x] Chunk System: 32x32x32 単位のデータ管理。 **(完了)**
- [x] Meshing: ブロック定義（YAML）に基づいてテクスチャUV座標を決定し、メッシュを生成。 **(完了)**
- [x] Custom Shader Support: ユーザーが書いた .wgsl シェーダーをマテリアルとして適用するパイプライン。 **(完了)**

#### マルチプレイ基盤 (Networking)
- [x] Headless Server: グラフィック描画なしでロジックだけが動くサーバー用ビルド設定。 **(完了)**
- [x] Replication: サーバーの状態（ブロックの配置情報、プレイヤー位置）をクライアントに同期する仕組み（bevy_renet または lightyear ライブラリの導入）。 **(完了)**
- [x] Client Prediction: 自分の移動操作を即座に反映し、サーバーからの補正をスムーズに行う処理（ラグ軽減）。 **(完了)**

### Phase 2: ロジックと物流シミュレーション (The Simulation)
目標: サーバー側で工場が稼働し、クライアント側でそれが滑らかに見える状態。

#### シミュレーションループ
- [x] Fixed Timestep: 描画FPSに関係なく、サーバー側で正確に毎秒20回（20TPS）更新されるロジックループ。 **(完了)**
- [x] Deterministic Logic: 同じ入力なら必ず同じ結果になる計算ロジック（ズレ防止）。 **(完了)**

#### 物流システム (Items)
- [x] Grid Data: ボクセルとは別の「搬送データ層」。** (完了)** (`src/gameplay/grid.rs`の`SimulationGrid`リソースがグリッドベースの機械配置の基盤を形成。)
- [x] Item Entity Optimization: アイテムを1つずつEntityにせず、軽量な構造体として管理し、描画時のみインスタンシング描画（Instanced Rendering）を行う高速化。** (完了)** (`src/gameplay/items.rs`の`update_visual_items`システムがアイテムの効率的な視覚表現を管理。)
- [x] Inventory System: 入れる/出す/スタックする等の基本操作。** (完了)** (`src/gameplay/grid.rs`の`ItemSlot`がインベントリ管理に使用され、`src/gameplay/machines/conveyor.rs`の`tick_conveyors`がアイテムの移動、衝突、転送を処理。)

#### デバッグ機能
- [x] Debug Overlay: F3キー等で座標、FPS、Chunk読み込み状況、メモリ使用量を表示。 **(完了)**
- [x] Debug Mode: 衝突判定の可視化、Tickごとのステップ実行、全アイテム取得などのチート機能。 **(完了)**

### Phase 3: 動力とマルチブロック (Power & Machines)
目標: YAMLで定義された複雑な機械が動き、エネルギーを消費する。

#### エネルギー計算
- [x] Graph Network: 接続された機械をグラフ構造として認識するアルゴリズム。(ただし、`src/gameplay/grid.rs`の`MachineInstance`に`power_node: Option<Entity>`フィールドが存在し、将来の統合が計画されている。)
- [x] Stress/Speed Propagation: 回転速度と応力の伝播計算。

#### マルチブロック
- [x] Structure Validator: ブロック配置パターンが定義通りかチェックする機能。**(完了)** (`src/gameplay/multiblock.rs`の`StructureValidator`がパターンマッチングとバリデーションを実装。`MultiblockPattern`でYAML定義可能なパターンをサポート。)
- [x] Master/Slave System: 判定の委譲処理。**(完了)** (`src/gameplay/multiblock.rs`の`MultiblockMaster`/`MultiblockSlave`コンポーネントと`FormedMultiblocks`リソースで管理。イベント駆動で形成・破壊を検知。)

#### インタラクション
- [x] GUI Framework: インベントリ画面、機械の設定画面。**(完了)** (`src/ui/machine_ui.rs`に`MachineUiPlugin`を実装。Assemblerの右クリックでUI表示、レシピ選択、インベントリ表示をサポート。Bevy State機械でUI状態管理。)

### Phase 4: 高度な自動化とスクリプティング (Automation)
目標: ユーザーがゲーム内でコードを書き、論理回路を組む。

#### スクリプト統合
- [ ] Lua VM (mlua): ゲーム内にLua仮想マシンを組み込み。
- [ ] Sandbox API: ユーザーコードからゲームをクラッシュさせないための制限付きAPI（read_sensor(), set_output() 等）。

#### 論理回路
- [ ] Signal System: 0/1 または数値信号を伝達するケーブルシステム。

### Phase 5: 最適化と配布 (Polish & Build)
目標: 巨大工場でも重くならず、簡単にModが作れる環境。

#### 高速化 (Optimization)
- [ ] Multithreading: 地形生成、物流計算、レンダリングの並列化徹底。
- [ ] LOD (Level of Detail): 遠くの機械や地形を簡略化して描画。

#### Modding SDK
- [ ] Example Mod: 「バニラ（デフォルト）」の要素自体を1つのModとして実装し、お手本にする。
- [ ] Hot Reloading: ゲーム起動中にテクスチャやYAMLを書き換えて即反映。

---

## 技術的詳細な提案

### 1. コンベアシステムのデータ構造
数万個のアイテムをカクつきなく描画・処理するためには、Bevy ECS (Entity Component System) の特性を最大限に活用し、チャンク単位での最適化、およびインスタンシング描画を組み合わせるのが最適です。

-   **`ConveyorBelt` Entity:** 各コンベアセグメントは`Entity`として表現され、`ConveyorComponent`を持つ。これには、コンベアの種類、向き、占有状態、および前後のコンベアへの参照（グラフ構造を形成するため）などが含まれます。
-   **`Item` Component:** アイテム自体は`ItemType`（YAML定義からロード）と、コンベア上での正確な位置 (`position_on_belt: f32`) を持つ軽量なコンポーネントとして存在します。それぞれのアイテムエンティティは、親コンベアエンティティの子として管理されることもあります。
-   **`ConveyorGrid` Resource:** グローバルなリソースとして、ワールドのグリッド上にどのコンベアが存在するかを効率的にクエリできる構造（例: `HashMap<IVec3, Entity>`）を保持します。
-   **`ItemBatch` Component / Instanced Rendering:** 個々のアイテムをそれぞれエンティティとして描画するのではなく、特定の`ItemType`のアイテムが多数存在するコンベア上で、それらをまとめて一つの描画コールで描画するインスタンシングを適用します。これにより、ドローコールの数を大幅に削減できます。アイテムは、その`ItemType`とワールド座標を持つ構造体の配列としてメモリに保持され、描画時にGPUに一括で送られます。
-   **チャンク単位の処理:** ワールドを小さなチャンクに分割し、各チャンク内のアイテムの移動、描画、物理演算（物理演算は使用しないが、境界チェックなど）を並行して処理します。これは特にアイテムの`Fixed Timestep`更新時に効果を発揮します。

### 2. 動力ネットワークの管理
「回転速度」と「応力」をネットワーク全体に伝播させるためのグラフ探索アルゴリズムとクラス設計は、以下のように考えられます。

-   **クラス/コンポーネント設計:**
    -   **`PowerNode` Component:** 全ての動力機械（`PowerSource`, `PowerConsumer`, `Shaft`）が持つコンポーネント。ノードID、接続先のノードIDのリストを持つ。
    -   **`PowerSource` Component:** `PowerNode`に加えて、`capacity: f32`（許容応力）、`current_speed: f32`（現在の回転速度）を持つ。
    -   **`PowerConsumer` Component:** `PowerNode`に加えて、`stress_impact: f32`（必要応力）、`is_active: bool`（稼働中か）を持つ。
    -   **`Shaft` Component:** `PowerNode`に加えて、`stress_resistance: f32`（応力抵抗、Shaft自体の強度）を持つ。
    -   **`PowerNetwork` Resource:** ワールドに存在する全ての動力ノードとその接続関係を管理するグローバルリソース。これはグラフデータ構造（例: `petgraph`クレートを使用）として実装されます。
    -   **`NetworkGroup` Component:** 接続された機械のグループ（Connected Component）ごとに割り当てられるIDと、そのグループ全体の`total_stress: f32`、`source_capacity: f32`、`is_overstressed: bool`などの状態を保持する。

-   **アルゴリズム:**
    1.  **ネットワーク構築:** 機械が配置・破壊されるたびに、`PowerNetwork`グラフを更新します。隣接する機械（`Shaft`を介して）をエッジとして追加します。
    2.  **グループ検出:** `PowerNetwork`グラフに対して幅優先探索 (BFS) または深さ優先探索 (DFS) を実行し、接続されている全てのノードのセット（Connected Component）を検出します。各セットに一意の`NetworkGroup` IDを割り当てます。
    3.  **応力計算:** 各`NetworkGroup`について、以下の計算を`Fixed Timestep`ごとに実行します。
        -   `total_stress = Sum(PowerConsumer.stress_impact for active consumers in group)`
        -   `source_capacity = Sum(PowerSource.capacity for sources in group)`
        -   `is_overstressed = (total_stress > source_capacity)`
    4.  **状態伝播:**
        -   `is_overstressed`が`true`の場合、その`NetworkGroup`内の全ての`PowerConsumer`の`is_active`を`false`に設定し、`PowerSource`の`current_speed`を`0.0`に設定します。
        -   `is_overstressed`が`false`の場合、`PowerSource`から`PowerConsumer`へ`current_speed`を伝播させます。これは再度BFS/DFSで伝播可能ですが、単純な場合はグループ内で一律に適用することも可能です。`PowerConsumer`は`is_active`が`true`の場合のみ`stress_impact`を計算に含めます。

### 3. ボクセル描画
大規模建築に耐えうるボクセル描画アプローチとして、以下を組み合わせます。

-   **Greedy Meshing:**
    -   チャンクデータを元に、隣接する同じテクスチャの面を結合して大きなクワッド（四角形）を生成するアルゴリズムです。これにより、頂点数とインデックス数を大幅に削減し、GPUへのデータ転送量を減らします。
    -   メッシュ生成は各チャンクで行われ、チャンクデータが変更された場合のみ再生成されます。これは非同期タスクとしてバックグラウンドスレッドで実行されるべきです。
-   **Frustum Culling & Occlusion Culling:**
    -   **Frustum Culling:** カメラの視錐台に入っていないチャンクやメッシュは描画しません。
    -   **Occlusion Culling:** 既に描画されたオブジェクトによって隠されているオブジェクトは描画しません。これは通常GPU側で処理されますが、CPU側で簡易的なカリングを行うことも可能です（例: 高層ビルの背後にあるチャンクを描画しない）。
-   **Level of Detail (LOD):**
    -   カメラから遠いチャンクは、より粗いメッシュや、完全に統合されたメッシュ（例: 遠景では個々のブロックが見えない一枚岩のようなメッシュ）で描画します。
    -   遠くのアイテムや小さな機械は、より単純なモデルに置き換えるか、またはアイコンとして描画します。
-   **Instanced Rendering (for small, repeated objects):**
    -   コンベア上のアイテムや、同じモデルを多数配置する小さな機械などは、インスタンシングで描画します。これにより、1つのモデルデータをGPUに一度送り、その変換行列（位置、回転、スケール）の配列だけを更新することで、多数のオブジェクトを効率的に描画できます。
-   **Custom Shader Support (.wgsl):**
    -   BevyのPBRパイプラインを拡張し、`Shader`アセットとして.wgslファイルをロード、マテリアルに適用できるようにします。これにより、パフォーマンス最適化のためのカスタムライティングやテクスチャリング、または特殊効果（例: 動力グリッドの可視化）などを実装できます。

### 4. 「回転動力システム」のコアロジックのコード例 (Rust/Bevy)
以下は、Kinetic Power Systemの主要なコンポーネントとシステム（ロジック）のドラフトコードです。

```rust
// src/gameplay/power.rs

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

// --- Components ---

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PowerNode {
    pub id: u32, // Unique ID for this node
    pub group_id: Option<u32>, // Which network group it belongs to
}

#[derive(Component, Debug)]
pub struct PowerSource {
    pub capacity: f32, // Max stress this source can handle
    pub current_speed: f32, // Current rotation speed (e.g., RPM)
}

#[derive(Component, Debug)]
pub struct PowerConsumer {
    pub stress_impact: f32, // Stress demand when active
    pub is_active: bool, // Whether the consumer is currently trying to draw power
    pub current_speed_received: f32, // Actual speed received from the network
}

#[derive(Component, Debug)]
pub struct Shaft {
    pub stress_resistance: f32, // Resistance to breaking under stress
}

// --- Resources ---

/// Represents the overall power network as a graph
#[derive(Resource, Default)]
pub struct PowerNetworkGraph {
    // Adjacency list: NodeId -> Set of connected NodeIds
    pub adjacencies: HashMap<u32, HashSet<u32>>,
    pub node_entity_map: HashMap<u32, Entity>, // Map node ID to its Bevy Entity
    pub next_node_id: u32, // Simple counter for unique node IDs
}

/// Represents a connected group of power machines
#[derive(Resource, Default)]
pub struct PowerNetworkGroups {
    pub groups: HashMap<u32, NetworkGroup>, // Group ID -> NetworkGroup
    pub next_group_id: u32,
}

#[derive(Debug, Default)]
pub struct NetworkGroup {
    pub nodes: HashSet<u32>, // All node IDs in this group
    pub total_stress_demand: f32,
    pub total_source_capacity: f32,
    pub is_overstressed: bool,
    pub ideal_speed: f32, // Ideal operating speed for this group
}

// --- Systems ---

/// System to initialize PowerNodes when a power-related entity is spawned.
pub fn spawn_power_node_system(
    mut commands: Commands,
    mut graph: ResMut<PowerNetworkGraph>,
    query_new_sources: Query<Entity, (With<PowerSource>, Without<PowerNode>)>,
    query_new_consumers: Query<Entity, (With<PowerConsumer>, Without<PowerNode>)>,
    query_new_shafts: Query<Entity, (With<Shaft>, Without<PowerNode>)>,
) {
    for entity in query_new_sources.iter().chain(query_new_consumers.iter()).chain(query_new_shafts.iter()) {
        let node_id = graph.next_node_id;
        commands.entity(entity).insert(PowerNode { id: node_id, group_id: None });
        graph.node_entity_map.insert(node_id, entity);
        graph.next_node_id += 1;
        // Connections would be handled by another system based on spatial data
    }
}

/// System to update the PowerNetworkGraph based on entity positions and connections.
/// This would be triggered by building/destroying power-related structures.
/// This is a placeholder for actual spatial connection logic.
pub fn update_power_graph_system(
    mut graph: ResMut<PowerNetworkGraph>,
    // This query would need to check entity positions and link them
    // For a voxel game, this usually involves checking adjacent grid cells
    // This example is simplified and doesn't contain actual spatial logic.
    mut has_changed: Local<bool>, // A flag to indicate if connections changed
) {
    // In a real game, this would re-evaluate connections based on grid positions.
    // For now, let's just make a dummy connection for demonstration.
    if !*has_changed && graph.node_entity_map.len() >= 2 {
        let mut node_ids: Vec<u32> = graph.node_entity_map.keys().cloned().collect();
        node_ids.sort(); // Ensure consistent ordering
        if let (Some(id1), Some(id2)) = (node_ids.get(0), node_ids.get(1)) {
            graph.adjacencies.entry(*id1).or_default().insert(*id2);
            graph.adjacencies.entry(*id2).or_default().insert(*id1);
            info!("Dummy: Connected node {} and {}", id1, id2);
            *has_changed = true;
        }
    }
}


/// System to detect connected components and create/update NetworkGroups.
pub fn detect_network_groups_system(
    mut power_network: ResMut<PowerNetworkGraph>,
    mut power_groups: ResMut<PowerNetworkGroups>,
    mut query_nodes: Query<&mut PowerNode>,
) {
    power_groups.groups.clear(); // Clear existing groups
    let mut visited: HashSet<u32> = HashSet::new();
    let mut current_group_id = power_groups.next_group_id;

    let all_node_ids: Vec<u32> = power_network.adjacencies.keys().cloned().collect();

    for &start_node_id in all_node_ids.iter() {
        if !visited.contains(&start_node_id) {
            let mut q: Vec<u32> = Vec::new();
            q.push(start_node_id);
            visited.insert(start_node_id);

            let mut new_group = NetworkGroup::default();
            new_group.nodes.insert(start_node_id);

            let mut head = 0;
            while head < q.len() {
                let node_id = q[head];
                head += 1;

                // Update PowerNode component with new group_id
                if let Some(entity) = power_network.node_entity_map.get(&node_id) {
                    if let Ok(mut power_node) = query_nodes.get_mut(*entity) {
                        power_node.group_id = Some(current_group_id);
                    }
                }

                if let Some(neighbors) = power_network.adjacencies.get(&node_id) {
                    for &neighbor_id in neighbors.iter() {
                        if !visited.contains(&neighbor_id) {
                            visited.insert(neighbor_id);
                            q.push(neighbor_id);
                            new_group.nodes.insert(neighbor_id);
                        }
                    }
                }
            }
            power_groups.groups.insert(current_group_id, new_group);
            current_group_id += 1;
        }
    }
    power_groups.next_group_id = current_group_id;
}


/// Fixed timestep system to calculate stress and update power states.
pub fn calculate_power_states_system(
    power_network: Res<PowerNetworkGraph>, // Read-only access to graph
    mut power_groups: ResMut<PowerNetworkGroups>,
    query_sources: Query<(&PowerNode, &PowerSource)>,
    query_consumers: Query<(&PowerNode, &PowerConsumer)>,
    mut query_sources_mut: Query<&mut PowerSource>,
    mut query_consumers_mut: Query<&mut PowerConsumer>,
) {
    for (group_id, group) in power_groups.groups.iter_mut() {
        let mut current_total_stress = 0.0;
        let mut current_total_capacity = 0.0;

        // Sum stress and capacity for this group
        for &node_id in group.nodes.iter() {
            if let Some(entity) = power_network.node_entity_map.get(&node_id) {
                if let Ok((_node, consumer)) = query_consumers.get(*entity) {
                    if consumer.is_active {
                        current_total_stress += consumer.stress_impact;
                    }
                }
                if let Ok((_node, source)) = query_sources.get(*entity) {
                    current_total_capacity += source.capacity;
                }
            }
        }

        group.total_stress_demand = current_total_stress;
        group.total_source_capacity = current_total_capacity;
        group.is_overstressed = current_total_stress > current_total_capacity;
        group.ideal_speed = if group.is_overstressed { 0.0 } else { 1.0 }; // Example: 1.0 for full speed, 0.0 for stopped

        // Propagate state to individual machines
        for &node_id in group.nodes.iter() {
            if let Some(entity) = power_network.node_entity_map.get(&node_id) {
                // Update consumers
                if let Ok(mut consumer) = query_consumers_mut.get_mut(*entity) {
                    consumer.current_speed_received = group.ideal_speed;
                    // If overstressed, consumers might automatically deactivate or slow down
                    if group.is_overstressed {
                        consumer.is_active = false; // Example: Force deactivation
                    }
                }
                // Update sources
                if let Ok(mut source) = query_sources_mut.get_mut(*entity) {
                    source.current_speed = group.ideal_speed;
                }
            }
        }
    }
}
```

### 5. C# (Unity) 環境におけるユーザーコードの安全な実行 (Rust/Bevyへの適用)
質問はC# (Unity) 環境向けですが、現在のプロジェクトがRust/Bevyであるため、Rust環境でのユーザーコード実行に最適なライブラリや手法を提案します。

-   **Lua VM (mlua):** `mlua`クレートはRustでLuaJITを安全に埋め込むための堅牢なソリューションです。サンドボックス環境を提供し、ユーザーコードからゲームのクラッシュを防ぐのに適しています。
    -   **安全性:** `mlua`はLuaサンドボックスを提供し、不要なファイルI/Oやネットワークアクセスを制限できます。提供するAPIを明示的にホワイトリスト化することで、ゲームエンジンへの不正なアクセスを防ぎます。
    -   **パフォーマンス:** LuaJITは非常に高速なJITコンパイラを備えており、十分なパフォーマンスを提供します。
    -   **統合:** RustコードからLua関数を呼び出したり、LuaコードからRust関数を呼び出したりするための強力なFFI (Foreign Function Interface) を提供します。
-   **Rhai:** Rhaiは、Rustで書かれた軽量な埋め込みスクリプトエンジンです。Luaほど強力なJITコンパイラはありませんが、よりRustフレンドリーなAPIとRustエコシステムとの統合のしやすさが魅力です。安全性も高く、サンドボックス化が容易です。
    -   ゲームロジックの一部をユーザーに公開するシンプルなスクリプティングには非常に適しています。
-   **WebAssembly (Wasm):** より高度なサンドボックスとパフォーマンスを求めるなら、ユーザーがRustやC++で書いたコードをWasmにコンパイルし、ゲーム内で実行するという選択肢もあります。これは実装が複雑になりますが、最高の安全性とパフォーマンスを提供できます。

**推奨:** 複雑なロジックや高度なスクリプティングを想定しているため、`mlua`が最もバランスの取れた選択肢となるでしょう。より軽量なものを求めるならRhaiも良いです。

### 6. ゲーム内オブジェクトをスクリプトにAPIとして公開するための設計パターン (Rust/Bevy/mluaの例)

`mlua`を使用して、ゲーム内の`RobotArm`や`Sensor`をLuaスクリプトに公開する設計パターンを以下に示します。

```rust
// src/gameplay/scripting.rs

use bevy::prelude::*;
use mlua::prelude::*; // For Lua integration

// --- Components for scripting interaction ---

#[derive(Component)]
pub struct ScriptedRobotArm {
    // Other RobotArm data...
    pub item_holding: Option<Entity>,
}

#[derive(Component)]
pub struct ScriptedSensor {
    pub value: f32,
    pub detected_entity: Option<Entity>,
}

// --- Lua API Exposure ---

/// Exposes a RobotArm's functions to Lua
struct LuaRobotArm {
    entity: Entity,
    // You might also need a World reference or direct access to a Bevy SystemParam
    // For simplicity, we'll assume a system handles the actual Bevy commands
}

impl LuaUserData for LuaRobotArm {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("grab", |_, this, _: ()| {
            // This would trigger a Bevy event or command to attempt a grab action
            // The actual grab logic runs in a Bevy system, not directly here.
            info!("Lua: RobotArm {:?} attempted to grab an item.", this.entity);
            Ok(true) // Return success/failure
        });

        methods.add_method("release", |_, this, _: ()| {
            info!("Lua: RobotArm {:?} attempted to release an item.", this.entity);
            Ok(true)
        });

        methods.add_method_get("is_holding_item", |_, this| {
            // You would query the Bevy World here to get the actual state
            // For a real game, this might involve a custom Bevy SystemParam or an exclusive system
            // For this example, let's just return a dummy value
            Ok(false) // Dummy value
        });
    }
}

/// Exposes a Sensor's functions to Lua
struct LuaSensor {
    entity: Entity,
}

impl LuaUserData for LuaSensor {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_get("read", |_, this| {
            // Query Bevy World for sensor value
            Ok(123.45f32) // Dummy value
        });
    }
}

// --- Bevy System for Lua Script Execution ---

#[derive(Component)]
pub struct LuaScript(pub String); // Store the script code

pub fn run_lua_scripts_system(
    mut commands: Commands,
    // lua_assets: Res<Assets<LuaScriptAsset>>, // Assuming LuaScriptAsset is loaded via AssetLoader
    script_query: Query<(Entity, &LuaScript, Option<&ScriptedRobotArm>, Option<&ScriptedSensor>)>,
    // Need a way to get a World reference or send events/commands from Lua
    // This part is complex and often involves custom Bevy plugins/systems
) {
    for (entity, lua_script_component, robot_arm_opt, sensor_opt) in script_query.iter() {
        let lua = Lua::new();

        // Register game objects (RobotArm, Sensor) to Lua
        if let Some(_) = robot_arm_opt {
            let lua_robot_arm = LuaRobotArm { entity };
            lua.globals().set("robot_arm", lua_robot_arm)
                .expect("Failed to set robot_arm global");
        }
        if let Some(_) = sensor_opt {
            let lua_sensor = LuaSensor { entity };
            lua.globals().set("sensor", lua_sensor)
                .expect("Failed to set sensor global");
        }

        // Execute the script
        match lua.load(&lua_script_component.0).eval::<()>() {
            Ok(_) => info!("Lua script for {:?} executed successfully.", entity),
            Err(e) => error!("Lua script error for {:?}: {}", entity, e),
        }
    }
}

// --- Example Lua Script (for assets/scripts/robot_logic.lua) ---
/*
function update()
    if sensor.read() > 100 then
        robot_arm.grab()
    else
        robot_arm.release()
    end
end
*/
```
**設計パターン:**
1.  **`LuaUserData` Trait:** `mlua`の`LuaUserData`トレイトを実装することで、Rustの構造体（`LuaRobotArm`, `LuaSensor`）をLuaスクリプトからオブジェクトとしてアクセスできるようにします。
2.  **イベント/コマンドシステム:** Luaスクリプトから直接Bevy Worldを変更することはできません。Luaスクリプト内の関数呼び出し（例: `robot_arm.grab()`) は、実際にはBevyのイベントを送信するか、Bevyのコマンドキューにコマンドを追加するRustのラッパー関数を呼び出します。これにより、Bevyの変更検出システムとデータフローを維持し、安全な変更を保証します。
3.  **データ同期:** Luaスクリプトがゲームオブジェクトの状態（例: `sensor.read()`）を読み取る場合、これはBevy Worldから最新のデータを取得してLuaに返すRustの関数を呼び出します。
4.  **アセットパイプライン:** Luaスクリプト自体も`LuaScriptAsset`としてBevyのアセットシステムを通じてロードされるべきです。

### 7. 宇宙ステーションのプログレスシステム設計

プレイヤーの達成感を最大化する視覚演出と共に、進捗を管理するシステムを設計します。

#### 1. データ構造
-   **`SpaceStationProgress` Resource:** グローバルなBevyリソースとして、宇宙ステーション全体の進捗を管理します。
    ```rust
    // src/gameplay/space_station.rs

    use bevy::prelude::*;
    use std::collections::HashMap;

    #[derive(Resource, Default)]
    pub struct SpaceStationProgress {
        pub total_delivered_items: HashMap<String, u64>, // ItemType -> count
        pub progress_points: u64, // Overall progress points
        pub current_stage: u32, // Current visual stage of the space station model
        pub max_stages: u32, // Total number of visual stages
        pub stage_thresholds: HashMap<u32, u64>, // Stage -> points needed for next stage
        pub delivered_items_since_last_launch: HashMap<String, u64>, // For rocket launch animation
    }

    #[derive(Component)]
    pub struct DeliveryTower; // Tag component for the delivery tower entity
    #[derive(Component)]
    pub struct SpaceStation; // Tag component for the space station entity
    #[derive(Component)]
    pub struct Rocket; // Tag component for the rocket entity
    ```

-   **`StationPart` Component:** 宇宙ステーションを構成する個々の3Dモデルパーツにアタッチ。どのステージで表示されるか、どのアイテムの納品に対応するかなどの情報を持つ。
    ```rust
    #[derive(Component)]
    pub struct StationPart {
        pub required_stage: u32, // Stage at which this part becomes visible/active
        pub part_model_handle: Handle<Scene>, // Handle to its 3D model
        // ... other rendering/positioning data
    }
    ```

#### 2. マネージャー各クラスの設計
BevyのSystemを使ってこれらを管理します。

-   **`DeliverySystem`:** 納品塔にアイテムが届いた際に`SpaceStationProgress`を更新します。
    -   アイテムが納品塔に搬入されると、`SpaceStationProgress.total_delivered_items`と`progress_points`を更新します。
    -   `progress_points`が`stage_thresholds`を超えた場合、`current_stage`をインクリメントし、`RocketLaunchEvent`を発生させます。
-   **`StationVisualUpdateSystem`:** `SpaceStationProgress.current_stage`の変化に応じて、`SpaceStation`エンティティの子として`StationPart`エンティティの表示/非表示を切り替えます。
    -   例えば、`current_stage`が`required_stage`以上の`StationPart`を有効化（レンダリング可能にする）。
    -   `StationPart`は`SceneBundle`または複数の`MeshBundle`で構成され、それぞれにモデルハンドルが設定されます。
-   **`RocketLaunchSystem`:** `RocketLaunchEvent`を受け取り、ロケットの発射アニメーション、ステーションへのドッキング、新パーツの追加演出を制御します。

#### 3. 視覚演出のアイディア

-   **納品塔からロケット発射:**
    1.  **イベントトリガー:** `SpaceStationProgress.current_stage`が更新されたときに`RocketLaunchEvent`を発生させます。このイベントには、納品されたアイテムのリストや現在のステージなどの情報を含めます。
    2.  **ロケットモデルの登場:** 納品塔の近くに、発射準備中のロケット3Dモデルをスポーンさせます。
    3.  **カウントダウンとエフェクト:** カウントダウンUIを表示し、蒸気、火花、音などのエフェクトを発生させます。
    4.  **発射アニメーション:** ロケットが納品塔からゆっくりと上昇し、加速しながら空へ飛び立つアニメーションを再生します。パーティクルエフェクトで炎や煙を表現します。カメラはロケットを追尾しても良いでしょう。
    5.  **「旅」の表現:** ロケットが宇宙へ向かう途中で、地上の工場や雲が遠ざかり、星々が見え始めるような背景の切り替え演出を加えます。長すぎないように、加速してワープするような演出も有効です。

-   **宇宙ステーションへのドッキングとパーツ増加:**
    1.  **ステーションへの接近:** ロケットが空の宇宙ステーションに接近するアニメーション。
    2.  **ドッキング演出:** ロケットがステーションの指定されたドッキングポートに滑らかに接続するアニメーション。接続時には小さな爆発エフェクトやライトの点滅などで合体感を強調します。
    3.  **パーツの出現:** ドッキングが完了すると、新しくアンロックされた`StationPart`の3Dモデルが、ステーションの対応する位置に突然出現するか、あるいは組み立てられるような短いアニメーション（例: 光のエフェクトと共にフェードイン、またはアームがパーツを取り付ける）で表示されます。
    4.  **全体像の更新:** ステーション全体が少しずつ大きくなり、複雑になっていく様子をプレイヤーに示します。新しいパーツが表示されるたびに、ステーションの新しい部分が機能し始めるような短い音響効果や視覚的なフィードバックを与えます。
    5.  **進捗表示:** 画面のどこかに、ステーションの現在の進捗（例: 「Stage 3/10: 生命維持装置モジュール建設中」）や、次に必要なアイテム、到達目標などを表示するUIを追加します。

**達成感の最大化:**
-   **明確な目標提示:** 各ステージで何が建設され、それがステーションのどの部分に対応するのかを明確に示します。
-   **段階的な報酬:** 新しいパーツが出現するだけでなく、それが新しい機能（例えば、特定のアイテムの生産ブーストや、新しい研究ツリーのアンロックなど）をもたらすことで、ゲームプレイ上の報酬と結びつけます。
-   **壮大なスケール感:** ロケットの発射やステーションの拡大を、画面いっぱいの演出でプレイヤーに体験させ、ゲームの壮大さを感じさせます。
-   **音響効果:** ロケットの轟音、パーツの組み立て音、完了時のファンファーレなど、適切な音響効果で演出を盛り上げます。
