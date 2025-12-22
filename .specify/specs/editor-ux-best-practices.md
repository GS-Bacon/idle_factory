# エディタUXベストプラクティス研究レポート

**作成日**: 2025-12-22
**目的**: ゲーム内エディタの設計指針を確立

---

## 1. 調査対象

| ツール | カテゴリ | 評価 |
|-------|---------|------|
| Unity Editor | ゲームエンジン | 業界標準 |
| Blender | 3Dモデリング | 習得後は高評価 |
| MagicaVoxel | ボクセルエディタ | 初心者に優しい |
| Unreal Blueprint | ノードエディタ | 非プログラマーに人気 |
| LDtk | レベルエディタ | UX重視設計 |

---

## 2. 優れたエディタの共通原則

### 2.1 Unity公式の4原則

> 「良いデザインとは、形・機能・美しさの調和である」
> — [Unity Blog](https://blog.unity.com/engine-platform/evolving-the-unity-editor-ux)

| 原則 | 説明 |
|------|------|
| **Modern（現代的）** | プロフェッショナルで信頼感のある外観 |
| **Familiar（親しみやすい）** | 業界標準に準拠した一貫性 |
| **Accessible（アクセシブル）** | 幅広いユーザー層に対応 |
| **Efficient（効率的）** | 一般的なタスクに最適化されたワークフロー |

### 2.2 MagicaVoxelの成功要因

> 「インターフェースは圧倒せず、邪魔にならず、すべての機能に手が届く」
> — [Slant Review](https://www.slant.co/options/5450/~magicavoxel-review)

**成功の鍵**:
- 左クリック = 追加、右クリック = カメラ回転（シンプル）
- ホバーでツールチップ表示
- 適切なデフォルト値
- 軽量で高速

### 2.3 Unreal Blueprintの成功要因

> 「非プログラマーでもビジュアルでゲームロジックを構築できる」
> — [Epic Games](https://dev.epicgames.com/documentation/en-us/unreal-engine/blueprints-visual-scripting-in-unreal-engine)

**成功の鍵**:
- 色分けされたワイヤーとポート（型が一目瞭然）
- 実行中のフロー可視化（デバッグ支援）
- ドラッグ&ドロップで接続
- C++との相互運用（上級者への道）

---

## 3. ベストプラクティス詳細

### 3.1 ナビゲーション

| プラクティス | 説明 | 例 |
|-------------|------|---|
| **予測可能な配置** | 機能は予測できる場所に | ファイル→編集→表示の順序 |
| **浅いメニュー** | サブメニューは2階層まで | 深すぎると迷う |
| **複数アクセス経路** | 同じ機能に複数の方法で到達 | メニュー + ショートカット + ツールバー |

### 3.2 キーボードショートカット

> 「優れたショートカットは発見可能で、記憶しやすく、競合しない」
> — [Knock Blog](https://knock.app/blog/how-to-design-great-keyboard-shortcuts)

| プラクティス | 説明 |
|-------------|------|
| **発見可能性** | メニューにショートカットを表示 |
| **一貫性** | C=コピー、V=ペーストなど標準に準拠 |
| **OS対応** | macOS/Windows/Linuxで適切な修飾キー |
| **競合回避** | ブラウザ/OSのショートカットと衝突しない |
| **学習曲線** | Cmd+単一キー（3.2回の練習で習得）を優先 |

**避けるべき組み合わせ**:
- `Alt+文字`（アクセスキーと競合）
- `Ctrl+Alt`（一部言語でAltGrと解釈）
- 特殊文字（@ {} [] \ ~ | ^ ' < >）

### 3.3 Undo/Redo

> 「Undoは実験を促す。試してダメならやり直せる安心感」
> — [Wayline](https://www.wayline.io/blog/undo-redo-crucial-level-design)

| プラクティス | 説明 |
|-------------|------|
| **深い履歴** | 数十〜数百ステップ戻れる |
| **細かい粒度** | ブラシストローク単位でUndo |
| **関係性の理解** | 削除Undo時に依存関係も復元 |
| **視覚的フィードバック** | 現在の履歴位置を表示 |
| **メモリ最適化** | 差分圧縮、状態スナップショット |

### 3.4 ノードエディタ

> 「データは上から下へ流れる。入力は上、出力は下」
> — [Dev.to](https://dev.to/cosmomyzrailgorynych/designing-your-own-node-based-visual-programming-language-2mpg)

| プラクティス | 説明 |
|-------------|------|
| **データフロー方向** | 上→下 または 左→右で統一 |
| **色分け** | 型ごとに色を統一（Float=緑、Bool=赤など） |
| **無効接続の防止** | 互換性のないポート同士は繋げない |
| **実行可視化** | 実行中のノードをハイライト |
| **60fps維持** | ズーム/パンをスムーズに |

### 3.5 ツールチップとヘルプ

| プラクティス | 説明 |
|-------------|------|
| **ホバーで表示** | ボタンの上で機能説明 |
| **ショートカット併記** | 「保存 (Ctrl+S)」のように |
| **遅延表示** | 即座ではなく0.5秒後に表示 |
| **コンテキスト依存** | 現在の状態に応じた説明 |

---

## 4. ツール別の学ぶべき点

### 4.1 MagicaVoxel（初心者向け設計）

```
学ぶべき点:
├── シンプルな操作体系（左クリック/右クリック）
├── 軽量で即座に起動
├── ホバーで全機能の説明表示
├── Model View / World Viewの明確な分離
└── 適切なデフォルト値
```

### 4.2 LDtk（レベルエディタ）

```
学ぶべき点:
├── 20年以上の経験に基づく設計
├── 「気持ちよさ」を最優先
├── プラットフォーマー/トップダウンに特化
├── すべてのUI細部が慎重に設計
└── モダンでユーザーフレンドリー
```

### 4.3 Unreal Blueprint（ノードエディタ）

```
学ぶべき点:
├── 色分けによる型の識別
├── 実行フローの可視化
├── ドラッグ&ドロップの直感性
├── 段階的な学習パス（初心者→上級者）
└── コードとの相互運用
```

### 4.4 Unity Editor（総合エディタ）

```
学ぶべき点:
├── 業界標準への準拠
├── ドッキング可能なパネル
├── Inspector/Hierarchyの分離
├── Play Mode での即時テスト
└── 一貫したビジュアルデザイン
```

---

## 5. 視覚デザインの原則

### 5.1 シンプルさ

> 「良いUIは透明で、邪魔にならない。使っていて意識しないのが理想」
> — [Unity Blog](https://blog.unity.com/engine-platform/evolving-the-unity-editor-ux)

| 原則 | 説明 |
|------|------|
| **最小限のUI** | コンテンツに集中できる |
| **グラデーション排除** | 気が散る装飾を減らす |
| **深さの維持** | コントロールの区別は保つ |
| **余白の活用** | 詰め込みすぎない |

### 5.2 フィードバック

| 操作 | フィードバック |
|------|---------------|
| クリック | ボタンの視覚変化 + 音 |
| ドラッグ | ドラッグ中の視覚表示 |
| 成功 | 緑のハイライト / チェックマーク |
| エラー | 赤のハイライト / 警告音 |
| 処理中 | プログレスバー / スピナー |

### 5.3 状態表示

| 状態 | 表現 |
|------|------|
| 選択中 | 青い枠線 / ハイライト |
| 無効 | グレーアウト |
| エラー | 赤い枠線 / アイコン |
| 変更あり | アスタリスク（*） |
| 保存済み | チェックマーク |

---

## 6. 学習曲線の設計

### 6.1 段階的開示（Progressive Disclosure）

```
初心者モード:
├── 基本ツールのみ表示
├── ガイド付きチュートリアル
└── ヘルプを積極的に表示

中級者モード:
├── 全ツール表示
├── ショートカット案内
└── ヒントは控えめ

上級者モード:
├── カスタマイズ可能
├── 高度な機能解放
└── ヘルプは要求時のみ
```

### 6.2 ゲーミフィケーション

> 「Blenderの学習曲線をゲーム化できないか？」
> — [UX Collective](https://uxdesign.cc/blender-start-here-gamifying-an-unapproachable-ui-d03d5ae29752)

| 手法 | 説明 |
|------|------|
| **チュートリアルクエスト** | 段階的なタスクで学習 |
| **バッジ/実績** | スキル習得の可視化 |
| **プログレスバー** | 学習進捗の表示 |
| **ヒント報酬** | 新機能発見で通知 |

---

## 7. 本作エディタへの提言

### 7.1 必須実装

| 項目 | 理由 |
|------|------|
| **ホバーツールチップ** | MagicaVoxel方式、全ボタンに説明 |
| **Undo/Redo（深い履歴）** | 100ステップ以上 |
| **ショートカット表示** | メニューに併記 |
| **即時フィードバック** | 全操作に視覚/聴覚反応 |
| **ドラッグ&ドロップ** | 直感的な操作 |

### 7.2 強く推奨

| 項目 | 理由 |
|------|------|
| **レシピノードエディタ** | Blueprint風の視覚的編集 |
| **プレビュー機能** | 編集結果を即座に確認 |
| **検索機能** | 大量のアイテム/レシピに対応 |
| **フィルタリング** | カテゴリ別表示 |
| **インポート/エクスポート** | 外部ツールとの連携 |

### 7.3 差別化ポイント

| 項目 | 説明 |
|------|------|
| **AI連携** | 視覚的指示でAIが理解しやすい |
| **ゲーム内統合** | 別アプリではなくゲーム内で編集 |
| **即時テスト** | 編集→プレイの即座な切り替え |

---

## 参考文献

- [Unity Blog - Evolving the Unity Editor UX](https://blog.unity.com/engine-platform/evolving-the-unity-editor-ux)
- [Unity UX Essentials](https://unityeditordesignsystem.unity.com/fundamentals/ux-essentials)
- [Slant - MagicaVoxel Review](https://www.slant.co/options/5450/~magicavoxel-review)
- [Epic Games - Blueprints Visual Scripting](https://dev.epicgames.com/documentation/en-us/unreal-engine/blueprints-visual-scripting-in-unreal-engine)
- [LDtk Level Editor](https://ldtk.io/)
- [Knock - How to Design Great Keyboard Shortcuts](https://knock.app/blog/how-to-design-great-keyboard-shortcuts)
- [Wayline - Undo/Redo in Level Design](https://www.wayline.io/blog/undo-redo-crucial-level-design)
- [Dev.to - Node-based Visual Programming](https://dev.to/cosmomyzrailgorynych/designing-your-own-node-based-visual-programming-language-2mpg)

---

*このレポートは、エディタソフトウェアのUX調査に基づいています。*
