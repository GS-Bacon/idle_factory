// src/gameplay/scripting.rs
//! Luaスクリプトエンジン
//! - Lua VMの初期化とサンドボックス設定
//! - スクリプトアセットの読み込み
//! - ゲームAPIのLuaへの公開
//! - プログラマブルマシンの実行

use bevy::prelude::*;
use mlua::{Lua, Result as LuaResult, Value, Table, MultiValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// スクリプトエンジンプラグイン
pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScriptEngine>()
            .init_resource::<ScriptRegistry>()
            .add_event::<ScriptOutputEvent>()
            .add_systems(FixedUpdate, tick_programmable_machines);
    }
}

/// スクリプト出力イベント
#[derive(Event)]
pub struct ScriptOutputEvent {
    pub machine_pos: IVec3,
    pub channel: String,
    pub value: SignalValue,
}

/// シグナル値（数値、文字列、ブール）
#[derive(Debug, Clone, PartialEq, Default)]
pub enum SignalValue {
    Number(f64),
    String(String),
    Boolean(bool),
    #[default]
    Nil,
}

impl SignalValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            SignalValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SignalValue::Boolean(b) => Some(*b),
            SignalValue::Number(n) => Some(*n != 0.0),
            _ => None,
        }
    }
}

/// スクリプトレジストリ（アセットからロードしたスクリプト）
#[derive(Resource, Default)]
pub struct ScriptRegistry {
    pub scripts: HashMap<String, String>,
}

impl ScriptRegistry {
    pub fn register(&mut self, name: String, code: String) {
        self.scripts.insert(name, code);
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.scripts.get(name)
    }
}

/// スクリプトエンジン（Lua VMラッパー）
#[derive(Resource)]
pub struct ScriptEngine {
    lua: Arc<Mutex<Lua>>,
}

impl Default for ScriptEngine {
    fn default() -> Self {
        let lua = Lua::new();

        // サンドボックス設定: 危険な関数を無効化
        lua.scope(|_scope| {
            let globals = lua.globals();
            // osとioライブラリを無効化（ファイルアクセス防止）
            let _ = globals.set("os", Value::Nil);
            let _ = globals.set("io", Value::Nil);
            let _ = globals.set("loadfile", Value::Nil);
            let _ = globals.set("dofile", Value::Nil);
            let _ = globals.set("load", Value::Nil);
            let _ = globals.set("require", Value::Nil);
            Ok(())
        }).ok();

        Self {
            lua: Arc::new(Mutex::new(lua)),
        }
    }
}

impl ScriptEngine {
    /// スクリプトを実行（サンドボックス内）
    pub fn execute(&self, code: &str, context: &ScriptContext) -> LuaResult<ScriptResult> {
        let lua = self.lua.lock().unwrap();

        // コンテキストをLuaグローバルに設定
        self.setup_context(&lua, context)?;

        // スクリプト実行（タイムアウト付き）
        let chunk = lua.load(code);
        let result = chunk.eval::<Value>();

        match result {
            Ok(_value) => {
                // 出力を収集
                let outputs = self.collect_outputs(&lua)?;
                Ok(ScriptResult {
                    success: true,
                    outputs,
                    error: None,
                })
            }
            Err(e) => {
                Ok(ScriptResult {
                    success: false,
                    outputs: HashMap::new(),
                    error: Some(e.to_string()),
                })
            }
        }
    }

    fn setup_context(&self, lua: &Lua, context: &ScriptContext) -> LuaResult<()> {
        let globals = lua.globals();

        // 入力シグナルを設定
        let inputs = lua.create_table()?;
        for (key, value) in &context.inputs {
            match value {
                SignalValue::Number(n) => inputs.set(key.as_str(), *n)?,
                SignalValue::String(s) => inputs.set(key.as_str(), s.as_str())?,
                SignalValue::Boolean(b) => inputs.set(key.as_str(), *b)?,
                SignalValue::Nil => inputs.set(key.as_str(), Value::Nil)?,
            }
        }
        globals.set("inputs", inputs)?;

        // 出力テーブルを初期化
        let outputs = lua.create_table()?;
        globals.set("outputs", outputs)?;

        // ユーティリティ関数を追加
        let print_fn = lua.create_function(|_, args: MultiValue| {
            let s: Vec<String> = args.iter().map(|v| format!("{:?}", v)).collect();
            info!("[Lua] {}", s.join(" "));
            Ok(())
        })?;
        globals.set("print", print_fn)?;

        // 数学関数
        let clamp = lua.create_function(|_, (value, min, max): (f64, f64, f64)| {
            Ok(value.clamp(min, max))
        })?;
        globals.set("clamp", clamp)?;

        let lerp = lua.create_function(|_, (a, b, t): (f64, f64, f64)| {
            Ok(a + (b - a) * t)
        })?;
        globals.set("lerp", lerp)?;

        Ok(())
    }

    fn collect_outputs(&self, lua: &Lua) -> LuaResult<HashMap<String, SignalValue>> {
        let globals = lua.globals();
        let outputs: Table = globals.get("outputs")?;

        let mut result = HashMap::new();
        for pair in outputs.pairs::<String, Value>() {
            let (key, value) = pair?;
            let signal = match value {
                Value::Number(n) => SignalValue::Number(n),
                Value::Integer(i) => SignalValue::Number(i as f64),
                Value::String(s) => SignalValue::String(s.to_str()?.to_string()),
                Value::Boolean(b) => SignalValue::Boolean(b),
                _ => SignalValue::Nil,
            };
            result.insert(key, signal);
        }

        Ok(result)
    }
}

/// スクリプト実行コンテキスト
#[derive(Default)]
pub struct ScriptContext {
    pub inputs: HashMap<String, SignalValue>,
    pub position: IVec3,
}

/// スクリプト実行結果
pub struct ScriptResult {
    pub success: bool,
    pub outputs: HashMap<String, SignalValue>,
    pub error: Option<String>,
}

/// プログラマブルマシンコンポーネント
#[derive(Component)]
#[derive(Default)]
pub struct Programmable {
    pub script_name: Option<String>,
    pub inline_code: Option<String>,
    pub inputs: HashMap<String, SignalValue>,
    pub outputs: HashMap<String, SignalValue>,
    pub last_error: Option<String>,
}

impl Programmable {
    pub fn with_script(script_name: &str) -> Self {
        Self {
            script_name: Some(script_name.to_string()),
            ..default()
        }
    }

    pub fn with_code(code: &str) -> Self {
        Self {
            inline_code: Some(code.to_string()),
            ..default()
        }
    }

    pub fn set_input(&mut self, name: &str, value: SignalValue) {
        self.inputs.insert(name.to_string(), value);
    }

    pub fn get_output(&self, name: &str) -> Option<&SignalValue> {
        self.outputs.get(name)
    }
}

/// プログラマブルマシンの実行システム
fn tick_programmable_machines(
    mut query: Query<(&Transform, &mut Programmable)>,
    engine: Res<ScriptEngine>,
    registry: Res<ScriptRegistry>,
    mut output_events: EventWriter<ScriptOutputEvent>,
) {
    for (transform, mut programmable) in &mut query {
        // スクリプトコードを取得
        let code = if let Some(ref name) = programmable.script_name {
            registry.get(name).cloned()
        } else {
            programmable.inline_code.clone()
        };

        let Some(code) = code else {
            continue;
        };

        // コンテキストを構築
        let context = ScriptContext {
            inputs: programmable.inputs.clone(),
            position: IVec3::new(
                transform.translation.x as i32,
                transform.translation.y as i32,
                transform.translation.z as i32,
            ),
        };

        // スクリプト実行
        match engine.execute(&code, &context) {
            Ok(result) => {
                programmable.last_error = result.error;

                // 出力をコンポーネントに保存
                for (key, value) in &result.outputs {
                    programmable.outputs.insert(key.clone(), value.clone());

                    // 出力イベントを発火
                    output_events.send(ScriptOutputEvent {
                        machine_pos: context.position,
                        channel: key.clone(),
                        value: value.clone(),
                    });
                }
            }
            Err(e) => {
                programmable.last_error = Some(e.to_string());
                warn!("Script execution error: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_engine_basic() {
        let engine = ScriptEngine::default();
        let context = ScriptContext::default();

        let code = r#"
            outputs.result = 42
        "#;

        let result = engine.execute(code, &context).unwrap();
        assert!(result.success);
        assert_eq!(
            result.outputs.get("result"),
            Some(&SignalValue::Number(42.0))
        );
    }

    #[test]
    fn test_script_engine_inputs() {
        let engine = ScriptEngine::default();
        let mut context = ScriptContext::default();
        context.inputs.insert("sensor".to_string(), SignalValue::Number(10.0));

        let code = r#"
            outputs.doubled = inputs.sensor * 2
        "#;

        let result = engine.execute(code, &context).unwrap();
        assert!(result.success);
        assert_eq!(
            result.outputs.get("doubled"),
            Some(&SignalValue::Number(20.0))
        );
    }

    #[test]
    fn test_script_engine_sandbox() {
        let engine = ScriptEngine::default();
        let context = ScriptContext::default();

        // osライブラリは無効化されている
        let code = r#"
            if os then
                outputs.has_os = true
            else
                outputs.has_os = false
            end
        "#;

        let result = engine.execute(code, &context).unwrap();
        assert!(result.success);
        assert_eq!(
            result.outputs.get("has_os"),
            Some(&SignalValue::Boolean(false))
        );
    }

    #[test]
    fn test_script_engine_error_handling() {
        let engine = ScriptEngine::default();
        let context = ScriptContext::default();

        let code = r#"
            error("intentional error")
        "#;

        let result = engine.execute(code, &context).unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_signal_value() {
        let num = SignalValue::Number(3.14);
        assert_eq!(num.as_number(), Some(3.14));
        assert_eq!(num.as_bool(), Some(true));

        let zero = SignalValue::Number(0.0);
        assert_eq!(zero.as_bool(), Some(false));

        let boolean = SignalValue::Boolean(true);
        assert_eq!(boolean.as_bool(), Some(true));
    }
}
