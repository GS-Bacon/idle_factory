//! Mod API server for external mod communication
//!
//! JSON-RPC style API over WebSocket (future implementation)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API リクエスト
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiRequest {
    /// リクエストID
    pub id: u64,
    /// メソッド名
    pub method: String,
    /// パラメータ
    pub params: HashMap<String, serde_json::Value>,
}

impl ApiRequest {
    /// 新しいリクエストを作成
    pub fn new(id: u64, method: &str) -> Self {
        Self {
            id,
            method: method.to_string(),
            params: HashMap::new(),
        }
    }

    /// パラメータを追加
    pub fn with_param(mut self, key: &str, value: serde_json::Value) -> Self {
        self.params.insert(key.to_string(), value);
        self
    }
}

/// API レスポンス
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    /// リクエストID
    pub id: u64,
    /// 成功フラグ
    pub success: bool,
    /// 結果データ
    pub result: Option<serde_json::Value>,
    /// エラーメッセージ
    pub error: Option<String>,
}

impl ApiResponse {
    /// 成功レスポンスを作成
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        Self {
            id,
            success: true,
            result: Some(result),
            error: None,
        }
    }

    /// エラーレスポンスを作成
    pub fn error(id: u64, message: &str) -> Self {
        Self {
            id,
            success: false,
            result: None,
            error: Some(message.to_string()),
        }
    }
}

/// API メソッドハンドラ
pub type ApiHandler = fn(&ApiRequest) -> ApiResponse;

/// API サーバー設定
#[derive(Clone, Debug)]
pub struct ApiServerConfig {
    /// ポート番号
    pub port: u16,
    /// ホスト
    pub host: String,
    /// 認証が必要か
    pub require_auth: bool,
    /// 最大接続数
    pub max_connections: usize,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            port: 9877,
            host: "127.0.0.1".to_string(),
            require_auth: false,
            max_connections: 10,
        }
    }
}

/// API メソッド定義
#[derive(Clone, Debug)]
pub struct ApiMethod {
    /// メソッド名
    pub name: String,
    /// 説明
    pub description: String,
    /// 必須パラメータ
    pub required_params: Vec<String>,
    /// オプションパラメータ
    pub optional_params: Vec<String>,
}

impl ApiMethod {
    /// 新しいメソッド定義を作成
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            required_params: Vec::new(),
            optional_params: Vec::new(),
        }
    }

    /// 必須パラメータを追加
    pub fn with_required(mut self, param: &str) -> Self {
        self.required_params.push(param.to_string());
        self
    }

    /// オプションパラメータを追加
    pub fn with_optional(mut self, param: &str) -> Self {
        self.optional_params.push(param.to_string());
        self
    }
}

/// API レジストリ
#[derive(Default)]
pub struct ApiRegistry {
    /// メソッド定義
    methods: HashMap<String, ApiMethod>,
}

impl ApiRegistry {
    /// 新しいレジストリを作成
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_default_methods();
        registry
    }

    /// デフォルトメソッドを登録
    fn register_default_methods(&mut self) {
        // ゲーム情報
        self.register(ApiMethod::new("game.version", "Get game version"));
        self.register(ApiMethod::new("game.state", "Get current game state"));

        // Mod管理
        self.register(ApiMethod::new("mod.list", "List all mods"));
        self.register(ApiMethod::new("mod.info", "Get mod information").with_required("mod_id"));
        self.register(ApiMethod::new("mod.enable", "Enable a mod").with_required("mod_id"));
        self.register(ApiMethod::new("mod.disable", "Disable a mod").with_required("mod_id"));

        // アイテム
        self.register(ApiMethod::new("item.list", "List all items"));
        self.register(
            ApiMethod::new("item.add", "Add custom item")
                .with_required("id")
                .with_required("name")
                .with_optional("stack_size"),
        );

        // 機械
        self.register(ApiMethod::new("machine.list", "List all machines"));
        self.register(
            ApiMethod::new("machine.add", "Add custom machine")
                .with_required("id")
                .with_required("name"),
        );

        // レシピ
        self.register(ApiMethod::new("recipe.list", "List all recipes"));
        self.register(
            ApiMethod::new("recipe.add", "Add custom recipe")
                .with_required("inputs")
                .with_required("outputs"),
        );

        // イベント
        self.register(
            ApiMethod::new("event.subscribe", "Subscribe to game events")
                .with_required("event_type"),
        );
        self.register(
            ApiMethod::new("event.unsubscribe", "Unsubscribe from game events")
                .with_required("event_type"),
        );
    }

    /// メソッドを登録
    pub fn register(&mut self, method: ApiMethod) {
        self.methods.insert(method.name.clone(), method);
    }

    /// メソッドを取得
    pub fn get(&self, name: &str) -> Option<&ApiMethod> {
        self.methods.get(name)
    }

    /// 全メソッドを取得
    pub fn all(&self) -> impl Iterator<Item = &ApiMethod> {
        self.methods.values()
    }

    /// メソッド数を取得
    pub fn count(&self) -> usize {
        self.methods.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_request_new() {
        let req = ApiRequest::new(1, "game.version");

        assert_eq!(req.id, 1);
        assert_eq!(req.method, "game.version");
        assert!(req.params.is_empty());
    }

    #[test]
    fn test_api_request_with_param() {
        let req = ApiRequest::new(1, "item.add")
            .with_param("id", serde_json::json!("custom_item"))
            .with_param("name", serde_json::json!("Custom Item"));

        assert_eq!(req.params.len(), 2);
        assert_eq!(req.params.get("id").unwrap(), "custom_item");
    }

    #[test]
    fn test_api_response_success() {
        let resp = ApiResponse::success(1, serde_json::json!({"version": "1.0.0"}));

        assert!(resp.success);
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let resp = ApiResponse::error(1, "Method not found");

        assert!(!resp.success);
        assert!(resp.result.is_none());
        assert_eq!(resp.error, Some("Method not found".to_string()));
    }

    #[test]
    fn test_api_server_config_default() {
        let config = ApiServerConfig::default();

        assert_eq!(config.port, 9877);
        assert_eq!(config.host, "127.0.0.1");
        assert!(!config.require_auth);
    }

    #[test]
    fn test_api_method_builder() {
        let method = ApiMethod::new("test.method", "Test method")
            .with_required("param1")
            .with_required("param2")
            .with_optional("opt1");

        assert_eq!(method.name, "test.method");
        assert_eq!(method.required_params.len(), 2);
        assert_eq!(method.optional_params.len(), 1);
    }

    #[test]
    fn test_api_registry() {
        let registry = ApiRegistry::new();

        // デフォルトメソッドが登録されている
        assert!(registry.count() > 0);
        assert!(registry.get("game.version").is_some());
        assert!(registry.get("mod.list").is_some());
    }

    #[test]
    fn test_api_registry_custom_method() {
        let mut registry = ApiRegistry::new();
        let initial_count = registry.count();

        registry.register(ApiMethod::new("custom.method", "Custom method"));

        assert_eq!(registry.count(), initial_count + 1);
        assert!(registry.get("custom.method").is_some());
    }
}
