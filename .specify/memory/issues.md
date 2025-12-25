# 課題管理

未解決の課題・TODO・改善案を記録。

---

## 未解決

### 無限ワールド生成の改善
**日付**: 2025-12-25
**優先度**: 中
**カテゴリ**: ゲームプレイ

**現状**:
- Perlinノイズによる基本的な地形生成は完了
- プレイヤー周囲4チャンク（128ブロック）を自動生成
- 非同期処理でスムーズな生成

**今後の実装予定**:

| 優先度 | 機能 | 詳細 |
|--------|------|------|
| 高 | スポーン位置最適化 | 地形高さを検出してプレイヤーを安全な位置にスポーン |
| 高 | 草ブロック追加 | 地表を草で覆う（現在はdirtのみ） |
| 中 | 地形バリエーション | 砂、砂利、水、溶岩などのブロック追加 |
| 中 | バイオーム | 地域ごとの異なる地形（森、砂漠、山岳など） |
| 中 | 洞窟生成 | 3Dノイズによる地下洞窟システム |
| 中 | 鉱石分布 | 深さに応じた鉱石の自動配置 |
| 低 | 構造物生成 | 村、ダンジョン、遺跡などの自動配置 |
| 低 | 木・植物 | 地表に木や草花を自動配置 |

**技術的改善**:
- チャンク生成の優先度（プレイヤー視線方向を優先）
- メッシュ最適化（Greedy Meshing）
- チャンクのシリアライズ/デシリアライズ（セーブ対応）

---

### Blender MCP使用時の注意点
**日付**: 2025-12-25
**優先度**: 中
**カテゴリ**: ツール

**問題1: MCPでの関数定義消失**
- `mcp__blender__execute_blender_code` は各実行が独立
- `exec(open("_base.py").read())` で読み込んだ関数は次の実行で消える
- **解決策**: 1回のexecute_blender_codeに必要な関数定義を全て含める

**問題2: スクリーンショットが真っ黒**
- `get_viewport_screenshot`が黒画像を返すことがある
- **解決策**: レンダリングしてファイル出力 → Readで確認
```python
bpy.ops.render.render(write_still=True)
```

**問題3: マテリアルエラー**
- `Principled BSDF`ノードが見つからないことがある
- **解決策**: `node.type == 'BSDF_PRINCIPLED'` でイテレーション検索

**現状の回避策（起動）**:
1. Blenderを手動で起動: `DISPLAY=:10 blender --python tools/blender_autostart_mcp.py &`
2. Blender内でMCPサーバーを起動: N→サイドバー→MCP→Start Server

---

### 仕様と実装の乖離（Steam特別モード＆エディタ）
**日付**: 2025-12-25
**優先度**: 高
**カテゴリ**: 仕様/実装

`steam-editor-mode.md` の仕様と実際の実装に多数の乖離が発見された。

#### 1. 実績システム（未実装）
**仕様**: 第6章で詳細定義
- 実績の種類: 通常、プログレス、隠し実績
- トリガー条件: 6種類（クエスト完了、アイテム生産数、機械設置数、プレイ時間、フェーズ到達、カスタム）
- データ構造: achievements.yaml
- Steam連携: achievements.vdf へのエクスポート

**実装状態**:
- `AchievementTracker` リソースなし
- achievements.yaml ファイルなし
- Steam API連携（bevy-steamworks）なし

#### 2. 統計システム（未実装）
**仕様**: 第7章で定義
- 追跡対象: total_items_produced, total_machines_placed, total_playtime_seconds, current_phase
- ローカル保存: stats.yaml

**実装状態**:
- stats.yaml ファイルなし
- 統計トラッキング機構なし

#### 3. 開発者モード（未実装）
**仕様**: 第4章で定義
- `developer-mode` フィーチャー または `FACTORY_DEVELOPER_MODE` 環境変数で有効化
- 有効時に [Steam], [Build] タブを追加表示

**実装状態**:
- Cargo.toml に `developer-mode` フィーチャーなし
- 環境変数チェックなし
- Steamタブ、Buildタブの実装なし

#### 4. プロファイルディレクトリ構造（不完全）
**仕様**:
```
profiles/vanilla/
  ├── profile.yaml
  ├── data/
  │   ├── items.yaml
  │   ├── recipes.yaml
  │   ├── quests.yaml
  │   └── achievements.yaml
  └── assets/
      ├── icons/
      └── models/
```

**実装状態**:
- `profiles/vanilla/` と `profiles/industrial/` に `profile.yaml` のみ存在
- `data/` ディレクトリなし
- `assets/` ディレクトリなし
- items.yaml, recipes.yaml, quests.yaml 不在

#### 5. エディタSteamタブ（未実装）
**仕様**: 第5章で実績エディタUI案を提示
**実装状態**: Tauriエディタに Items, Recipes, Localization のみ、Steam/Build タブなし

#### 6. MOD APIのSemVer検証（未実装）
**仕様**: 第8章でSemVerサポートを規定
**実装状態**:
- version フィールドは String のみ
- ">= 1.0.0" 形式の依存関係が記述されても評価されない
- 非推奨化・マイグレーション機能なし

#### 7. プロファイル切り替え（未実装）
**仕様**: 第3章で「疑似ホットリロード」フローを規定
- Tauriからゲームに終了シグナル送信
- ローディング画面表示
- 新プロファイルでゲーム再起動

**実装状態**:
- `ProfileChangedEvent` は定義されているが、実際のフロー未実装
- Tauriエディタからゲームプロセスへの通信なし
- 子プロセス管理なし

#### 8. active_profile.yaml の起動時読み込み（未実装）
**仕様**: ゲーム起動時に `config/active_profile.yaml` を読み込んでプロファイルを復元
**実装状態**:
- ファイルは存在するが読み込み処理がない
- `ProfileManager::scan_profiles()` を呼ぶだけ

#### 9. ProfileSelect画面の機能不足
**仕様**: プロファイル選択後、そのプロファイルが有効化されるべき
**実装状態**:
- `ProfileList` がハードコード（vanilla のみ）
- `ProfileManager` からの実際のプロファイル取得が行われていない

**対応優先順位**:
1. プロファイルディレクトリ構造を仕様通りに整備
2. 実績・統計システムの基本実装
3. active_profile.yaml の起動時読み込み
4. エディタの Steam/Build タブ実装
5. 疑似ホットリロード機構

---

### 仕様の矛盾・曖昧性
**日付**: 2025-12-25
**優先度**: 中
**カテゴリ**: 仕様

#### 1. GameModeとコンセプトの矛盾
**問題**:
- `index-compact.md`: 「特徴: サバイバル要素なし、純粋な自動化ゲーム」
- 実装: `GameMode::Survival` がデフォルト、PlayerHealth, FallDamage 等が実装済み

**要検討**: Survival/Creative の位置づけを明確化（両方サポート？Survival廃止？）

#### 2. Luaサンドボックスの詳細仕様不明
**問題**:
- 「Lua 5.4（サンドボックス）」「os/io/load/require無効化」と記載
- Luaスクリプトの入力元（ゲーム内 or MODファイル）が不明
- サンドボックス化の実装コードが見つからない

#### 3. プロファイル管理の責任分担が曖昧
**問題**:
- アイテム・レシピは `profiles/vanilla/data/items.yaml` なのか、エディタ側管理なのか
- エディタでの編集対象プロファイルの決定方法が不明確

#### 4. マルチプレイ仕様の未定義
**問題**:
- profile.yaml に `server_origin` フィールドが定義
- しかしマルチプレイ通信の詳細仕様がない
- プロファイル・MODのサーバー間同期方式が不明

**要検討**: マルチプレイ実装時に仕様を明確化

---

### 中優先度システム実装（ゲームリサーチより）
**日付**: 2025-12-25
**優先度**: 中
**カテゴリ**: ゲームプレイ

高優先度システム（9件）は実装完了。以下は今後実装予定の中優先度機能:

| 機能 | 参考ゲーム | 詳細 |
|------|------------|------|
| ヘッドリフト・ポンプシステム | Satisfactory | パイプの高さ制限、ポンプで揚程追加 |
| 発電機ティアシステム | Satisfactory/Factorio | 発電機のグレード・効率段階 |
| クリーチャー自動化 | Palworld | パルのような生き物を工場労働に活用 |
| カラールーティング列車 | Shapez2 | 色ベースの信号で列車を分岐 |
| グローバル信号送受信 | Shapez2 | ワイヤレスでシグナル伝達 |

**必要な3Dモデル（中優先度）**:
- pump.glb - 揚水ポンプ
- coal_generator.glb - 石炭発電機
- fuel_generator.glb - 燃料発電機
- nuclear_reactor.glb - 原子炉
- train_engine.glb - 列車機関車
- cargo_wagon.glb - 貨物車両
- train_station.glb - 列車駅
- color_router.glb - カラールーター
- signal_transmitter.glb - 信号送信機
- signal_receiver.glb - 信号受信機
- creature_worker.glb - ワーカークリーチャー
- creature_pen.glb - クリーチャー飼育場
- creature_feeder.glb - 自動エサやり機
- pal_sphere.glb - パルスフィア風捕獲アイテム
- breeding_station.glb - 繁殖ステーション
- creature_transport.glb - クリーチャー輸送機

**関連ファイル**:
- `.specify/memory/feature-roadmap-from-research.md` - 詳細なロードマップ

---

## 完了

### ビルド時SIGSEGV問題
**日付**: 2025-12-24
**優先度**: 高
**カテゴリ**: ビルド

**問題**: rustc SIGSEGVでビルド失敗

**原因**: メモリ不足（VSCode + Claude Code複数プロセスで約4GB使用）

**対策**:
- 開発環境の拡張（RAM増設/スワップ増量）検討
- 使用していないプロセスの終了
- 状況に応じてDocker/リモートビルド

---

*新しい課題は「未解決」セクションに追加*
