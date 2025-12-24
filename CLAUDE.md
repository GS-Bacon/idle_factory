# Claude Code メモリ

## 作業ルール

| ルール | 詳細 |
|--------|------|
| ログ | 作業中は出力禁止、終了時に短く報告 |
| テスト | 実装/変更時はテスト作成、Clippy後にテスト実行 |
| コミット | 適宜実行、日本語メッセージ、プッシュは指示待ち |
| 中断禁止 | 確認を求めず最後まで完了させる |
| 記録 | 完了/失敗時は`changelog.md`に追記 |
| パターン | 仕様変更/実装前に`patterns-compact.md`確認 |
| UI実装 | UI作成/修正時は`ui-design-rules.md`に従う |
| 動作確認 | 指示時は自動操作でテスト、クラッシュ時はログ解析 |
| 3Dモデル | 「XXXのモデルを作成」指示でサブエージェント起動 |

## 3Dモデル生成ルール

「〇〇のモデルを作成して」という指示を受けたら:

1. **サブエージェント起動** (Task tool, subagent_type: general-purpose)
2. **プロンプト内容**:
```
docs/style-guide.json を読み込み、以下のモデルのBlender Pythonスクリプトを生成せよ。

【モデル】{ユーザー指定のモデル名}
【カテゴリ】{machine/item/structureを推定}

【必須要件】
- スタイルガイドのgeometry.primitives.allowed_shapesのみ使用
- 円/球/円柱はcurve_substitutionsで八角形に置換
- grid.snap_unit (0.0625) でスナップ
- polygon_budgetの上限を守る
- materials.presetsから適切な素材選択
- mechanical_partsの形状仕様に従う
- originの位置規則に従う
- 可動部があればbonesとanimation規則に従う

【出力】
tools/blender_scripts/{model_name}.py にスクリプトを書き出し、
使用方法をユーザーに報告せよ。
```

3. **サブエージェント完了後**: 結果をユーザーに報告

## 参照ファイル

| 優先度 | ファイル | 用途 |
|--------|----------|------|
| 必須 | `API_REFERENCE.md` | 関数・構造体一覧 |
| 状況 | `.specify/memory/constitution.md` | プロジェクト原則 |
| 状況 | `.specify/memory/changelog.md` | 開発履歴 |
| 状況 | `.specify/memory/patterns-compact.md` | 52パターン |
| 状況 | `.specify/memory/ui-design-rules.md` | UIデザインルール |
| 状況 | `docs/style-guide.json` | 3Dモデルスタイルガイド |
| 状況 | `.specify/specs/index-compact.md` | 全仕様集約 |
| 詳細時 | `.specify/specs/*.md`, `src/**/*.rs` | 個別レポート/ソース |

**原則**: 圧縮版優先、詳細は必要時のみ

## README.md

**含める**: 概要、AI駆動開発の特徴、エディタ/MOD、実装済み機能、予定、技術スタック
**含めない**: ビルド手順、環境構築、構成詳細、操作方法、連絡先
**冒頭に記載**: 「🤖 AI開発プロジェクト」
