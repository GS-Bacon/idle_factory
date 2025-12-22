# Graphics & Rendering Skill

グラフィックス・レンダリング最適化を支援するスキルです。

## 参照ドキュメント

- `.specify/specs/graphics-implementation-best-practices.md`
- `.specify/specs/graphics-implementation-antipatterns.md`

---

## ボクセルレンダリング

### Greedy Meshing

```rust
fn greedy_mesh(chunk: &Chunk) -> Mesh {
    let mut quads = Vec::new();

    for axis in [Axis::X, Axis::Y, Axis::Z] {
        for slice in chunk.slices(axis) {
            // 同じブロックタイプの連続領域を結合
            let merged = merge_adjacent_faces(slice);
            quads.extend(merged);
        }
    }

    build_mesh_from_quads(quads)
}
```

### チャンクメッシュ更新

```rust
// 変更されたチャンクのみ再メッシュ
fn remesh_dirty_chunks(
    mut chunks: Query<(&Chunk, &mut Mesh), Changed<Chunk>>,
) {
    for (chunk, mut mesh) in &mut chunks {
        *mesh = greedy_mesh(chunk);
    }
}
```

---

## LOD (Level of Detail)

### 距離ベースLOD

```rust
#[derive(Component)]
enum ChunkLod {
    Full,    // 0-64m: 完全メッシュ
    Medium,  // 64-128m: 簡略化
    Low,     // 128-256m: 最小
    Icon,    // 256m+: ビルボード
}

fn update_lod(
    camera: Query<&Transform, With<Camera>>,
    mut chunks: Query<(&Transform, &mut ChunkLod)>,
) {
    let cam_pos = camera.single().translation;

    for (transform, mut lod) in &mut chunks {
        let distance = cam_pos.distance(transform.translation);
        *lod = match distance {
            d if d < 64.0 => ChunkLod::Full,
            d if d < 128.0 => ChunkLod::Medium,
            d if d < 256.0 => ChunkLod::Low,
            _ => ChunkLod::Icon,
        };
    }
}
```

---

## インスタンシング

### 大量オブジェクト描画

```rust
// 1回の描画コールで数千個レンダリング
fn instance_render(
    items: Query<&Transform, With<ConveyorItem>>,
) {
    let transforms: Vec<Mat4> = items.iter()
        .map(|t| t.compute_matrix())
        .collect();

    // GPUインスタンシング
    draw_instanced(item_mesh, &transforms);
}
```

### バッチング戦略

| オブジェクトタイプ | 戦略 |
|-------------------|------|
| コンベアアイテム | インスタンシング |
| 機械 | 静的バッチング |
| パーティクル | GPUパーティクル |
| 地形 | Greedy Meshing |

---

## カリング

### フラスタムカリング

```rust
fn frustum_cull(
    camera: &Camera,
    objects: &[AABB],
) -> Vec<usize> {
    let frustum = camera.frustum();

    objects.iter()
        .enumerate()
        .filter(|(_, aabb)| frustum.intersects(aabb))
        .map(|(i, _)| i)
        .collect()
}
```

### オクルージョンカリング

```rust
// 簡易オクルージョン：前フレームの可視性を使用
fn hierarchical_culling(
    chunks: &[Chunk],
    last_visible: &HashSet<ChunkId>,
) -> HashSet<ChunkId> {
    // 前回見えていたチャンクと隣接チャンクをチェック
    // ...
}
```

---

## シェーダー最適化

### プリコンパイル

```rust
// 起動時にシェーダーをコンパイル
fn precompile_shaders(app: &mut App) {
    let shaders = [
        "terrain.wgsl",
        "machine.wgsl",
        "particle.wgsl",
        "ui.wgsl",
    ];

    for shader in shaders {
        compile_shader(shader);
    }
}
```

### シェーダースタッター防止

| 対策 | 説明 |
|------|------|
| ウォームアップ | ロード画面で全シェーダーを1回描画 |
| パイプラインキャッシュ | コンパイル済みシェーダーを保存 |
| 条件分岐削減 | if/elseをバリアント化 |

---

## テクスチャ管理

### アトラス

```rust
struct TextureAtlas {
    texture: Handle<Image>,
    tile_size: Vec2,
    tiles: HashMap<BlockId, UVRect>,
}

fn get_uv(atlas: &TextureAtlas, block: BlockId) -> UVRect {
    atlas.tiles[&block]
}
```

### ミップマップ

| テクスチャタイプ | ミップマップ |
|------------------|--------------|
| 地形 | 必須 |
| UI | 不要 |
| スプライト | 状況による |

---

## パフォーマンス目標

| 指標 | 目標 |
|------|------|
| FPS | 60以上 |
| フレーム時間 | 16.67ms以下 |
| 描画コール | 500以下 |
| GPUメモリ | 2GB以下 |

### プロファイリング

```rust
fn profile_frame() {
    // CPU時間
    let cpu_start = Instant::now();
    // ... システム処理
    let cpu_time = cpu_start.elapsed();

    // GPU時間（Bevyの診断機能）
    // app.add_plugins(FrameTimeDiagnosticsPlugin);
}
```

---

## 工場ゲーム向け最適化

### 大規模工場

| 問題 | 対策 |
|------|------|
| 数千の機械 | LOD + カリング |
| 数万のアイテム | インスタンシング |
| 広大な地形 | チャンクストリーミング |
| 複雑なUI | UIキャッシング |

### コンベアアイテム

```rust
// 見える範囲のアイテムのみ描画
fn cull_conveyor_items(
    camera: &Camera,
    items: &[ConveyorItem],
) -> Vec<&ConveyorItem> {
    let visible_area = camera.visible_world_rect();

    items.iter()
        .filter(|item| visible_area.contains(item.position))
        .collect()
}
```

---

## チェックリスト

- [ ] Greedy Meshingを使用しているか
- [ ] LODシステムがあるか
- [ ] インスタンシングを活用しているか
- [ ] フラスタムカリングがあるか
- [ ] シェーダープリコンパイルがあるか
- [ ] 描画コールが500以下か
- [ ] 60 FPS維持できるか

---

*このスキルはグラフィックス最適化の品質を確保するためのガイドです。*
