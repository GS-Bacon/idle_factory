# AI 3Dモデリング ベストプラクティス

## 1. AI生成の得意・不得意

| カテゴリ | 品質 | 備考 |
|----------|------|------|
| シンプルな小物 | ⭐⭐⭐⭐ | カップ、椅子など |
| スタイライズドキャラ | ⭐⭐⭐ | Low-polyなら良好 |
| **産業機械・メカ** | ⭐⭐ | **構造的精度が不足** |
| 有機的形状（人体等） | ⭐ | 破綻しやすい |

### なぜ機械が難しいか

1. **学習データの問題** - ネット上の3Dモデルは品質がバラバラ
2. **構造的整合性** - 動く部品、噛み合わせの理解が不十分
3. **詳細の欠落** - ネジ、パイプ、配線などの細部は再現困難

---

## 2. プロンプトの書き方

### 悪い例

```
採掘機を作って
```

### 良い例

```
Industrial mining drill machine, steampunk style,
metallic bronze and iron materials, cylindrical drill head,
compact body with visible gears, game-ready low-poly,
single object, clean topology
```

### 必須要素チェックリスト

| 要素 | 例 | 重要度 |
|------|-----|--------|
| **メインオブジェクト** | mining drill, excavator | 必須 |
| **スタイル** | steampunk, industrial, sci-fi | 必須 |
| **素材** | metal, iron, bronze, rusty steel | 必須 |
| **用途** | game-ready, low-poly, high-detail | 推奨 |
| **構造ヒント** | cylindrical body, visible gears | 推奨 |
| **単一指定** | single object | 推奨 |

### 避けるべきこと

- ❌ 複数オブジェクトを一度に（「机と椅子とランプ」）
- ❌ 詳細すぎる指定（5個以上の詳細）
- ❌ 矛盾した指定（「シンプルで複雑な」）
- ❌ 専門用語の乱用
- ❌ 抽象的な概念（具体的な視覚参照なし）

---

## 3. 複雑なオブジェクトの分割戦略

### 採掘機の例

**一発生成（失敗しやすい）:**
```
mining machine
```

**分割生成（推奨）:**
```
Step 1: "industrial machine base, boxy metal body, steel plates, low-poly"
Step 2: "industrial drill head, spiral metal bit, cylindrical, metallic"
Step 3: "mechanical arm, hydraulic pistons, metal joints"
Step 4: Blenderで結合・調整
```

### 分割の原則

1. **機能単位で分ける** - ベース、駆動部、作業部
2. **各パーツはシンプルに** - 1つの主要形状
3. **素材を統一** - 後で調整しやすい
4. **最後に手動調整** - 位置、スケール、接続

---

## 4. Hyper3D設定

| 設定 | 推奨値 | 用途 |
|------|--------|------|
| Quality | Medium (18k) | プロトタイプ |
| Quality | High (50k) | 本番用 |
| Mode | Rodin Detail | 機械・精密物 |
| HighPack | ON | 高品質テクスチャ |
| bbox_condition | [1,1,2] | 縦長オブジェクト |

### bbox_condition例

```python
# 横長（車両など）
bbox_condition = [2, 1, 0.5]

# 縦長（タワーなど）
bbox_condition = [1, 1, 3]

# 立方体（箱など）
bbox_condition = [1, 1, 1]
```

---

## 5. 代替戦略: Sketchfab

AI生成より既存モデルが適切な場合:

### 検索のコツ

```python
# 具体的なキーワード
query = "mining drill machine"  # ✓
query = "excavator industrial"  # ✓
query = "machine"  # ✗ 広すぎ

# フィルタ活用
downloadable = True  # 必須
```

### よく使うカテゴリ

| 用途 | 検索キーワード |
|------|----------------|
| 採掘機 | mining, drill, excavator |
| 工場機械 | factory, industrial, machinery |
| コンベア | conveyor, belt, transport |
| 炉 | furnace, smelter, forge |
| 発電機 | generator, power, turbine |

---

## 6. ワークフロー比較

### シンプルなオブジェクト

```
[Hyper3D一発生成] → [スケール調整] → 完了
```

### 中程度の複雑さ

```
[Hyper3D生成] → [Blenderで調整] → 完了
```

### 複雑な機械

```
[パーツ分割] → [各パーツ生成] → [Blender結合] → [手動調整] → 完了
```

### 高品質必須

```
[Sketchfab検索] → [ダウンロード] → [スケール調整] → 完了
```

---

## 7. トラブルシューティング

| 問題 | 原因 | 解決策 |
|------|------|--------|
| 形が崩れる | プロンプトが曖昧 | 具体的なスタイル・素材追加 |
| 複数オブジェクト化 | "single object"未指定 | 明示的に指定 |
| ディテール不足 | Quality設定が低い | High/Detail使用 |
| サイズがおかしい | target_size未指定 | 適切なサイズ指定 |
| タイムアウト | 処理が重い | 分割生成 |
| 機械が不正確 | AI限界 | Sketchfab使用 or 分割 |

---

## 8. ゲーム用チェックリスト

### 生成後の確認

- [ ] サイズが適切か（他オブジェクトとの比較）
- [ ] ポリゴン数が許容範囲か
- [ ] テクスチャが適用されているか
- [ ] 原点位置が適切か
- [ ] 回転が正しいか（Y-up）

### 修正コマンド例

```python
# スケール調整
bpy.ops.transform.resize(value=(0.5, 0.5, 0.5))

# 原点をジオメトリ中心に
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY')

# Y-up回転修正
obj.rotation_euler = (math.radians(90), 0, 0)
```

---

## Sources

- [Blender MCP公式](https://blender-mcp.com/)
- [Hyper3D公式](https://hyper3d.ai/)
- [Meshy AI - Prompt Engineering](https://www.meshy.ai/blog/meshy-5-text-to-3d)
- [3D AI Studio Guide](https://www.3daistudio.com/3d-generator-ai-comparison-alternatives-guide/what-to-write-in-text-prompts-for-3d)
