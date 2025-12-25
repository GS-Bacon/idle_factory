# /generate-model スキル 批判的レビュー

**作成日**: 2025-12-25
**対象**: `/generate-model` スキル（モデリングトレーニングモード含む）

---

## 1. 重大な問題点

### 1.1 視覚的評価の欠如

**問題**: 現在の評価システムは数値メトリクスのみで、生成されたモデルの視覚的品質を確認していない。

**具体例**: 斧のモデルが「浮いて見える」問題が発生したが、接続性チェック（バウンディングボックス重複）は通過していた。

**原因**:
- `evaluator.py`の接続性チェックはバウンディングボックスの重複のみ
- パーツの向き・位置関係の妥当性は未検証
- 人間の目で見た「自然さ」を評価する仕組みがない

**影響度**: 致命的 - 数値上は成功でも実際には使えないモデルが量産される

### 1.2 参照モデルの未活用

**問題**: `challenges.yaml`に`reference_models`が定義されているが、実際のコード生成時に参照していない。

**具体例**:
```yaml
reference_models:
  - hammer
  - pickaxe
```
これらの参照モデルのコードパターンを学習に活用していない。

**影響度**: 高 - 成功パターンの蓄積が活かされていない

### 1.3 パーツ配向の未検証

**問題**: 刃や頭部の向きが正しいかどうかのチェックがない。

**具体例**: 斧の刃が垂直に立っているべきところ、水平になっていても検出されない。

**影響度**: 高 - 「それらしさ」を損なうモデルが生成される

### 1.4 単一イテレーションの盲目的実行

**問題**: モデル生成後に視覚確認なしで次の課題に進んでいる。

**理想のフロー**:
1. モデル生成
2. スクリーンショット取得
3. 視覚評価
4. 問題あれば修正
5. 合格なら次へ

**現状のフロー**:
1. モデル生成
2. 数値評価（接続性・三角形数のみ）
3. 次へ

---

## 2. 中程度の問題点

### 2.1 課題定義の曖昧さ

**問題**: `challenges.yaml`の`required_parts`が抽象的すぎる。

```yaml
required_parts:
  - handle
  - head
  - blade
```

**改善案**:
```yaml
required_parts:
  - name: handle
    shape: octagonal_prism
    material: wood
    orientation: Z  # Z軸方向（縦）
    position: bottom
  - name: blade
    shape: trapezoid
    material: iron
    orientation: X  # X軸方向（横に広がる）
    position: top
    facing: front   # 刃先が前方を向く
```

### 2.2 フィードバックの汎用性

**問題**: `feedback_generator.py`のテンプレートが一般的すぎて、具体的な修正箇所を特定できない。

**現状の出力例**:
```
接続されていないパーツがあります: blade
```

**理想の出力例**:
```
blade のZ位置 (0.15) がhandle上端 (0.12) から離れています。
修正: blade_z = handle_top_z - 0.003 (オーバーラップ)
```

### 2.3 スキルメモリの浅さ

**問題**: `skill_memory.json`は成功/失敗カウントのみで、具体的なコードパターンを学習していない。

**改善案**: 成功したモデルの実際のコードスニペットを保存し、次回生成時に参照する。

---

## 3. 軽微な問題点

### 3.1 評価関数の未実装

`evaluator.py`の`_evaluate_primitives()`が常に8.0を返す：
```python
def _evaluate_primitives(self) -> Tuple[float, Dict]:
    # TODO: 実装
    return 8.0, {"status": "not_implemented"}
```

### 3.2 材質チェックの甘さ

プリセット外の色が使われてもRGB近似で通過してしまう可能性。

---

## 4. 改善提案

### 4.1 視覚評価の導入（最優先）

モデル生成後にスクリーンショットを取得し、視覚的に評価するループを追加。

```
生成 → スクリーンショット → 視覚分析 → 問題あれば修正 → 再生成
```

**実装方法**:
1. `mcp__blender__get_viewport_screenshot()`でスクリーンショット取得
2. AIがスクリーンショットを確認
3. 問題があれば具体的な修正を指示

### 4.2 参照モデル読み込み

生成前に参照モデルのコードを読み込み、パターンを抽出：

```python
# 参照モデルからパターン抽出
def load_reference_patterns(model_name):
    code = read_file(f"tools/blender_scripts/{model_name}.py")
    # 寸法比、パーツ構成、接続方法を抽出
    return patterns
```

### 4.3 配向検証の追加

各パーツの主軸を計算し、期待する向きと比較：

```python
def validate_part_orientation(obj, expected_axis):
    """パーツの主軸が期待する方向かを検証"""
    dims = obj.dimensions
    primary_axis = max(enumerate(dims), key=lambda x: x[1])[0]
    axis_map = {'X': 0, 'Y': 1, 'Z': 2}
    return primary_axis == axis_map[expected_axis]
```

### 4.4 コードパターンDB

成功したモデルの寸法・構成をJSONで保存し、次回参照：

```json
{
  "axe": {
    "handle": {"length": 0.15, "radius": 0.012},
    "blade": {"width": 0.06, "height": 0.04, "orientation": "X"},
    "connection": {"blade_z": "handle_top - 0.003"}
  }
}
```

---

## 5. 実装優先順位

| 優先度 | 項目 | 工数 | 効果 |
|--------|------|------|------|
| 1 | 視覚評価ループ導入 | 低 | 高 |
| 2 | 課題定義の詳細化（orientation追加） | 低 | 高 |
| 3 | 参照モデルコード読み込み | 低 | 中 |
| 4 | パーツ配向検証 | 中 | 中 |
| 5 | コードパターンDB | 高 | 中 |

---

## 6. 即時実装する改善

### 6.1 generate-model.mdに視覚確認ステップを追加

トレーニングモード実行時に:
1. モデル生成後、必ずスクリーンショットを確認
2. 「それらしく見えるか」を視覚的に判定
3. 問題があれば修正して再生成

### 6.2 challenges.yamlにorientation情報を追加

パーツごとに期待する向きを明示。

### 6.3 skill_memory.jsonにコードパターンを保存

成功時に実際のコードの要点を記録。

---

## 7. 結論

現在の`/generate-model`スキルは**数値的には機能しているが、視覚的品質を保証できない**。

最優先で「視覚評価ループ」を導入し、生成→確認→修正のサイクルを回せるようにすべき。これにより「斧に見えない斧」のような問題を防止できる。
