use bevy::prelude::*;
use std::collections::HashMap;

/// ゲームモード（サバイバル/クリエイティブ）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum GameMode {
    Survival,
    #[default]
    Creative, // テスト段階ではクリエイティブモードをデフォルトに
}

/// コマンド実行結果
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

/// コマンド実行イベント
#[derive(Event)]
pub struct ExecuteCommandEvent {
    pub input: String,
}

/// コマンド結果イベント
#[derive(Event)]
pub struct CommandResultEvent {
    pub result: CommandResult,
}

/// コマンドレジストリ
#[derive(Resource, Default)]
pub struct CommandRegistry {
    commands: HashMap<String, fn(&[String]) -> CommandAction>,
}

/// コマンドアクション
#[derive(Debug, Clone)]
pub enum CommandAction {
    SetGameMode(GameMode),
    ShowHelp(Vec<String>),
    Error(String),
}

impl CommandRegistry {
    pub fn register(&mut self, name: &str, handler: fn(&[String]) -> CommandAction) {
        self.commands.insert(name.to_string(), handler);
    }

    pub fn parse(&self, input: &str) -> CommandAction {
        let parts: Vec<String> = input.trim().split_whitespace().map(|s| s.to_string()).collect();

        if parts.is_empty() {
            return CommandAction::Error("コマンドを入力してください".to_string());
        }

        let cmd_name = &parts[0];
        let args = &parts[1..];

        if let Some(handler) = self.commands.get(cmd_name) {
            handler(args)
        } else {
            CommandAction::Error(format!("不明なコマンド: {}", cmd_name))
        }
    }

    pub fn get_completions(&self, input: &str) -> Vec<String> {
        let parts: Vec<String> = input.trim().split_whitespace().map(|s| s.to_string()).collect();

        if parts.is_empty() || (parts.len() == 1 && !input.ends_with(' ')) {
            // コマンド名の補完
            let prefix = parts.first().map(|s| s.as_str()).unwrap_or("");
            return self.commands.keys()
                .filter(|name| name.starts_with(prefix))
                .cloned()
                .collect();
        }

        // コマンド引数の補完（gamemode専用）
        let cmd_name = &parts[0];
        if cmd_name == "gamemode" {
            let prefix = parts.get(1).map(|s| s.as_str()).unwrap_or("");
            return vec!["survival", "creative"]
                .into_iter()
                .filter(|mode| mode.starts_with(prefix))
                .map(|s| s.to_string())
                .collect();
        }

        vec![]
    }

    pub fn list_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }
}

/// /gamemode コマンドハンドラー
fn gamemode_handler(args: &[String]) -> CommandAction {
    if args.is_empty() {
        return CommandAction::Error("/gamemode <survival|creative>".to_string());
    }

    let mode_str = args[0].to_lowercase();
    match mode_str.as_str() {
        "survival" | "s" | "0" => CommandAction::SetGameMode(GameMode::Survival),
        "creative" | "c" | "1" => CommandAction::SetGameMode(GameMode::Creative),
        _ => CommandAction::Error(format!("不明なゲームモード: {}", mode_str)),
    }
}

/// /help コマンドハンドラー
fn help_handler(_args: &[String]) -> CommandAction {
    CommandAction::ShowHelp(vec!["gamemode".to_string(), "help".to_string()])
}

/// コマンドシステムプラグイン
pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GameMode>()
            .init_resource::<CommandRegistry>()
            .add_event::<ExecuteCommandEvent>()
            .add_event::<CommandResultEvent>()
            .add_systems(Startup, register_default_commands)
            .add_systems(Update, execute_commands);
    }
}

fn register_default_commands(mut registry: ResMut<CommandRegistry>) {
    registry.register("gamemode", gamemode_handler);
    registry.register("help", help_handler);
}

/// コマンド実行システム
fn execute_commands(
    mut events: EventReader<ExecuteCommandEvent>,
    mut result_events: EventWriter<CommandResultEvent>,
    registry: Res<CommandRegistry>,
    mut game_mode: ResMut<GameMode>,
) {
    for event in events.read() {
        let action = registry.parse(&event.input);

        let result = match action {
            CommandAction::SetGameMode(mode) => {
                *game_mode = mode;
                CommandResult {
                    success: true,
                    message: format!("ゲームモードを {:?} に変更しました", mode),
                }
            }
            CommandAction::ShowHelp(commands) => {
                CommandResult {
                    success: true,
                    message: format!("利用可能なコマンド:\n{}", commands.join(", ")),
                }
            }
            CommandAction::Error(msg) => {
                CommandResult {
                    success: false,
                    message: msg,
                }
            }
        };

        result_events.send(CommandResultEvent { result });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamemode_command() {
        let mut app = App::new();
        app.add_plugins(CommandsPlugin);
        app.update();

        // デフォルトはサバイバル
        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Survival);

        // クリエイティブに変更
        app.world_mut().insert_resource(GameMode::Creative);
        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Creative);

        // サバイバルに戻す
        app.world_mut().insert_resource(GameMode::Survival);
        assert_eq!(*app.world().resource::<GameMode>(), GameMode::Survival);
    }

    #[test]
    fn test_command_completions() {
        let mut app = App::new();
        app.add_plugins(CommandsPlugin);
        app.update();

        let registry = app.world().resource::<CommandRegistry>();

        // コマンド名の補完
        let completions = registry.get_completions("gam");
        assert!(completions.contains(&"gamemode".to_string()));

        // 引数の補完
        let completions = registry.get_completions("gamemode ");
        assert!(completions.contains(&"survival".to_string()));
        assert!(completions.contains(&"creative".to_string()));
    }
}
