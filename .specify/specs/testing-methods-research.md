# ゲーム/GUIアプリのテスト手法研究レポート

**作成日**: 2025-12-22
**目的**: 本プロジェクト（Bevy + Tauri）に適用可能なテスト手法の調査

---

## 1. 調査対象

| カテゴリ | ツール/手法 | 適用対象 |
|---------|------------|---------|
| ゲームE2E | GameDriver, Test.AI | ゲーム本体 |
| Bevyテスト | leafwing-input-manager, World直接操作 | Bevy ECS |
| Tauriテスト | WebdriverIO, Selenium | エディタUI |
| パターン | リプレイ、プロパティベース | 全般 |

---

## 2. ゲーム特有のテスト課題

### 2.1 従来のE2Eテストとの違い

> 「ゲームは非決定的で、同じ入力でも毎回違う結果になる」
> — Game Developer Conference 2023

| 課題 | Webアプリ | ゲーム |
|------|----------|--------|
| **決定性** | 高い（DOM操作は予測可能） | 低い（物理演算、AI、乱数） |
| **状態量** | 少ない（ページ単位） | 膨大（ワールド全体） |
| **フレーム依存** | なし | あり（60fps同期） |
| **入力方式** | クリック/タイプ | 連続入力（WASD押しっぱなし） |

### 2.2 工場ゲーム特有の課題

| 課題 | 説明 |
|------|------|
| **時間経過** | 生産完了まで待つ必要がある |
| **大規模状態** | 数百〜数千の機械が同時稼働 |
| **連鎖反応** | 1つの変更が全体に波及 |
| **長時間プレイ** | メモリリーク、パフォーマンス劣化 |

---

## 3. 適用可能なツール

### 3.1 Bevyテスト（ゲーム本体）

#### leafwing-input-manager
```rust
// 入力をモック可能にするクレート
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    Move,
    Jump,
    PlaceMachine,
}

// テストでの入力シミュレーション
fn test_player_movement(world: &mut World) {
    // 仮想入力を注入
    let mut action_state = ActionState::<Action>::default();
    action_state.press(&Action::Move);

    // システム実行
    app.update();

    // 結果検証
    assert!(player_moved(world));
}
```

**利点**:
- 入力レイヤーを抽象化、テストで差し替え可能
- キーボード/マウス/ゲームパッドを統一的に扱える
- 本番コードの変更が最小限

#### World直接操作テスト
```rust
#[test]
fn test_machine_production() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, production_system);

    // 初期状態セットアップ
    let machine = app.world.spawn((
        Machine::new(MachineType::Furnace),
        Inventory::with_items(vec![Item::IronOre]),
    )).id();

    // 100フレーム経過をシミュレート
    for _ in 0..100 {
        app.update();
    }

    // 生産完了を検証
    let inventory = app.world.get::<Inventory>(machine).unwrap();
    assert!(inventory.contains(Item::IronIngot));
}
```

**利点**:
- ECSの強みを活かしたユニットテスト
- フレームごとの細かい検証が可能
- 外部依存なし

### 3.2 Tauriテスト（エディタUI）

#### WebdriverIO + Tauri Driver
```javascript
// wdio.conf.js
const { spawn } = require('child_process');

exports.config = {
    runner: 'local',
    specs: ['./test/specs/**/*.ts'],
    capabilities: [{
        'tauri:options': {
            application: './src-tauri/target/release/idle_factory_editor',
        }
    }],
    services: ['tauri'],
};

// test/specs/editor.test.ts
describe('Editor', () => {
    it('should create new recipe', async () => {
        // ボタンクリック
        await $('#new-recipe-btn').click();

        // 入力
        await $('#recipe-name').setValue('Iron Ingot');
        await $('#input-item').selectByVisibleText('Iron Ore');

        // 保存
        await $('#save-btn').click();

        // 検証
        const recipes = await $$('.recipe-item');
        expect(recipes).toHaveLength(1);
    });
});
```

**利点**:
- 標準的なWebテストツールが使える
- CSSセレクタでUI要素を特定
- スクリーンショット、動画記録可能

### 3.3 統合テスト（ゲーム + エディタ）

```rust
// エディタで作成したデータをゲームで読み込むテスト
#[test]
fn test_editor_game_integration() {
    // 1. エディタでレシピ作成（Tauri API経由）
    let recipe = create_recipe_via_editor("test_recipe.yaml");

    // 2. ゲームにデータ読み込み
    let mut app = create_game_app();
    app.world.insert_resource(recipe);

    // 3. ゲーム内で動作検証
    spawn_machine_with_recipe(&mut app, "test_recipe");

    for _ in 0..100 {
        app.update();
    }

    assert!(production_completed(&app));
}
```

---

## 4. テストパターン

### 4.1 リプレイシステム

#### 入力ベースリプレイ
```rust
#[derive(Serialize, Deserialize)]
struct InputRecord {
    frame: u64,
    action: PlayerAction,
}

#[derive(Serialize, Deserialize)]
struct Replay {
    seed: u64,  // 乱数シード
    inputs: Vec<InputRecord>,
}

// 記録
fn record_input(mut replay: ResMut<Replay>, input: Res<Input<Action>>, frame: Res<FrameCount>) {
    for action in input.get_just_pressed() {
        replay.inputs.push(InputRecord {
            frame: frame.0,
            action: action.clone(),
        });
    }
}

// 再生
fn replay_input(replay: Res<Replay>, frame: Res<FrameCount>, mut input: ResMut<Input<Action>>) {
    for record in &replay.inputs {
        if record.frame == frame.0 {
            input.press(record.action.clone());
        }
    }
}
```

**用途**:
- バグ再現（ユーザーからリプレイファイル受け取り）
- 回帰テスト（同じシナリオを繰り返し実行）
- デモ/チュートリアル（操作を記録して再生）

#### 状態スナップショット
```rust
#[derive(Serialize, Deserialize)]
struct WorldSnapshot {
    entities: Vec<EntitySnapshot>,
    resources: Vec<ResourceSnapshot>,
}

// 保存
fn save_snapshot(world: &World) -> WorldSnapshot {
    // 全エンティティとリソースをシリアライズ
}

// 復元
fn load_snapshot(world: &mut World, snapshot: &WorldSnapshot) {
    world.clear_entities();
    for entity in &snapshot.entities {
        world.spawn(/* ... */);
    }
}
```

**用途**:
- 特定状態からのテスト開始
- クイックセーブ/ロード機能
- バグ状態の保存

### 4.2 プロパティベーステスト

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn inventory_never_negative(
        items in prop::collection::vec(any::<Item>(), 0..100),
        remove_indices in prop::collection::vec(0usize..100, 0..50)
    ) {
        let mut inventory = Inventory::new();

        // ランダムなアイテム追加
        for item in &items {
            inventory.add(item.clone());
        }

        // ランダムなインデックスで削除
        for idx in remove_indices {
            let _ = inventory.remove_at(idx);
        }

        // 不変条件: 数量は常に0以上
        for slot in inventory.slots() {
            prop_assert!(slot.count >= 0);
        }
    }
}
```

**用途**:
- 予期しない入力パターンの発見
- 境界条件の自動探索
- 不変条件の検証

### 4.3 ゴールデンテスト

```rust
#[test]
fn test_recipe_output_golden() {
    let result = process_recipe("iron_ingot", 100);

    // 期待値をファイルから読み込み
    let expected = include_str!("golden/iron_ingot_100.json");

    // 比較
    assert_eq!(
        serde_json::to_string_pretty(&result).unwrap(),
        expected
    );
}
```

**用途**:
- 出力の回帰テスト
- 設定ファイルのフォーマット検証
- UIスクリーンショット比較

### 4.4 ファズテスト

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // ランダムなバイト列をコマンドとして解釈
    if let Ok(commands) = parse_commands(data) {
        let mut world = setup_test_world();

        for cmd in commands {
            // クラッシュしないことを確認
            let _ = execute_command(&mut world, cmd);
        }
    }
});
```

**用途**:
- セキュリティテスト
- パーサーの堅牢性検証
- 予期しない入力への耐性

---

## 5. CI/CD統合

### 5.1 ヘッドレステスト

```yaml
# .github/workflows/test.yml
name: Game Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Install dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y xvfb libasound2-dev libudev-dev

    - name: Run tests with virtual display
      run: |
        xvfb-run -a cargo test --features headless
      env:
        DISPLAY: ':99'

    - name: Run Tauri tests
      run: |
        cd src-tauri
        xvfb-run -a cargo test
```

### 5.2 パフォーマンス回帰テスト

```rust
#[bench]
fn bench_1000_machines(b: &mut Bencher) {
    let mut app = setup_app_with_machines(1000);

    b.iter(|| {
        app.update();
    });
}

// CI で前回の結果と比較
// 10%以上の劣化で警告
```

### 5.3 スクリーンショット比較

```yaml
- name: Visual regression test
  run: |
    cargo run --features screenshot-test -- --screenshot output.png
    compare -metric AE expected.png output.png diff.png
```

---

## 6. 本プロジェクトへの適用案

### 6.1 テスト階層

```
テスト階層
├── ユニットテスト（Bevy ECS）
│   ├── コンポーネントのロジック
│   ├── システムの単体動作
│   └── リソースの状態管理
│
├── 統合テスト（ゲーム内）
│   ├── 機械の生産サイクル
│   ├── 電力ネットワーク
│   └── レシピ処理
│
├── E2Eテスト（Tauri）
│   ├── エディタUI操作
│   ├── データ保存/読み込み
│   └── プロファイル切り替え
│
└── 回帰テスト（全体）
    ├── リプレイ再生
    ├── スクリーンショット比較
    └── パフォーマンスベンチ
```

### 6.2 推奨ツールセット

| レイヤー | ツール | 理由 |
|---------|--------|------|
| **Bevy入力** | leafwing-input-manager | 入力抽象化、テスト容易 |
| **ECSテスト** | 組み込みWorld操作 | 外部依存なし |
| **Tauri UI** | WebdriverIO | 標準的、ドキュメント豊富 |
| **プロパティ** | proptest | Rust標準、組み合わせ爆発に強い |
| **ファズ** | cargo-fuzz | セキュリティ検証 |
| **CI** | GitHub Actions + xvfb | 無料、Linux対応 |

### 6.3 優先実装順

#### Phase 1: 基盤整備
1. **leafwing-input-manager導入**
   - 現在のRaw入力をAction抽象化
   - テスト用のモック入力機構

2. **ECSユニットテスト強化**
   - 機械生産システムのテスト
   - 電力システムのテスト
   - インベントリ操作のテスト

#### Phase 2: 統合テスト
3. **リプレイシステム**
   - 入力記録/再生
   - 乱数シード固定
   - バグ再現用

4. **状態スナップショット**
   - ワールド保存/復元
   - 特定状態からのテスト

#### Phase 3: E2E・CI
5. **Tauri E2Eテスト**
   - WebdriverIO設定
   - エディタ操作テスト

6. **CI/CD整備**
   - ヘッドレステスト環境
   - パフォーマンス回帰検出

### 6.4 テスト駆動開発への移行

```
新機能追加フロー:
1. 失敗するテストを書く
2. 最小限の実装で通す
3. リファクタリング
4. 回帰テストに追加
```

---

## 7. テストパターン速見表

| パターン | 用途 | 実装コスト | 効果 |
|---------|------|-----------|------|
| **ECSユニット** | 個別システム検証 | 低 | 高 |
| **リプレイ** | バグ再現、回帰テスト | 中 | 高 |
| **プロパティベース** | 境界条件発見 | 中 | 高 |
| **スナップショット** | 状態復元 | 中 | 中 |
| **E2E（Tauri）** | UI操作検証 | 高 | 中 |
| **スクリーンショット** | 視覚回帰 | 低 | 中 |
| **ファズ** | セキュリティ | 中 | 低〜高 |

---

## 8. 注意点

### 8.1 ゲームテストの落とし穴

| 落とし穴 | 説明 | 対策 |
|---------|------|------|
| **非決定性** | 乱数、物理演算で結果が変わる | シード固定、許容範囲設定 |
| **フレーム依存** | テスト環境で速度が異なる | 論理フレームで検証 |
| **状態爆発** | 組み合わせが膨大 | プロパティベースで網羅 |
| **UIフレーキー** | 描画タイミングで失敗 | 明示的な待機、リトライ |

### 8.2 避けるべきアンチパターン

| アンチパターン | 問題 | 代替案 |
|---------------|------|--------|
| **sleep固定待機** | 遅くて不安定 | 条件待機（poll） |
| **画像マッチング依存** | 解像度で壊れる | セマンティックな検証 |
| **巨大な統合テスト** | 遅くて脆い | 小さなユニットテスト |
| **手動テストのみ** | 回帰を見逃す | 自動化必須 |

---

## 参考文献

- [Bevy Testing Best Practices](https://bevy-cheatbook.github.io/programming/testing.html)
- [leafwing-input-manager](https://github.com/leafwing-studios/leafwing-input-manager)
- [Tauri WebDriver Guide](https://v2.tauri.app/develop/tests/webdriver/)
- [proptest Book](https://proptest-rs.github.io/proptest/intro.html)
- [Game Testing Automation (GDC)](https://www.gdcvault.com/browse/gdc-23/tag/testing)

---

*このレポートは、ゲームおよびGUIアプリケーションのテスト手法調査に基づいています。*
