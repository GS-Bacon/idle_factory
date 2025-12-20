use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use crate::gameplay::commands::{CommandRegistry, ExecuteCommandEvent, CommandResultEvent};

/// コマンドUI表示状態
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum CommandUiState {
    #[default]
    Closed,
    Open,
}

/// コマンド入力リソース
#[derive(Resource, Default)]
pub struct CommandInput {
    pub text: String,
    pub cursor_position: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
}

/// コマンド実行結果履歴
#[derive(Resource, Default)]
pub struct CommandHistory {
    pub messages: Vec<(String, bool, f32)>, // (message, success, timestamp)
}

/// マーカーコンポーネント
#[derive(Component)]
pub struct CommandUiRoot;

#[derive(Component)]
pub struct CommandInputText;

#[derive(Component)]
pub struct CommandHistoryText;

#[derive(Component)]
pub struct CommandSuggestions;

pub struct CommandUiPlugin;

impl Plugin for CommandUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<CommandUiState>()
            .init_resource::<CommandInput>()
            .init_resource::<CommandHistory>()
            .add_systems(Update, (
                toggle_command_ui,
                handle_command_input.run_if(in_state(CommandUiState::Open)),
                update_command_results,
            ))
            .add_systems(OnEnter(CommandUiState::Open), (spawn_command_ui, release_cursor))
            .add_systems(OnExit(CommandUiState::Open), despawn_command_ui);
    }
}

fn toggle_command_ui(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<CommandUiState>>,
    mut next_state: ResMut<NextState<CommandUiState>>,
) {
    // スラッシュキーまたはTキーでコマンドUIを開く
    if (keyboard.just_pressed(KeyCode::Slash) || keyboard.just_pressed(KeyCode::KeyT))
        && *state.get() == CommandUiState::Closed
    {
        next_state.set(CommandUiState::Open);
    }

    // Escapeキーで閉じる
    if keyboard.just_pressed(KeyCode::Escape) && *state.get() == CommandUiState::Open {
        next_state.set(CommandUiState::Closed);
    }
}

fn release_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

fn spawn_command_ui(mut commands: Commands) {
    commands.spawn((
        CommandUiRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ))
    .with_children(|parent| {
        // コマンド履歴表示エリア
        parent.spawn((
            CommandHistoryText,
            Text::new(""),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ));

        // 候補表示エリア
        parent.spawn((
            CommandSuggestions,
            Text::new(""),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));

        // コマンド入力ボックス
        parent.spawn((
            Node {
                width: Val::Percent(60.0),
                height: Val::Px(30.0),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                CommandInputText,
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn despawn_command_ui(
    mut commands: Commands,
    query: Query<Entity, With<CommandUiRoot>>,
    mut input: ResMut<CommandInput>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    input.text.clear();
    input.cursor_position = 0;
}

fn handle_command_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<CommandInput>,
    registry: Res<CommandRegistry>,
    mut next_state: ResMut<NextState<CommandUiState>>,
    mut input_text_query: Query<&mut Text, With<CommandInputText>>,
    mut suggestions_query: Query<&mut Text, (With<CommandSuggestions>, Without<CommandInputText>)>,
    mut command_events: EventWriter<ExecuteCommandEvent>,
) {
    // Enterキーでコマンド実行
    if keyboard.just_pressed(KeyCode::Enter) {
        if !input.text.is_empty() {
            let cmd_text = if input.text.starts_with('/') {
                input.text[1..].to_string()
            } else {
                input.text.clone()
            };

            // コマンド実行イベント送信
            command_events.send(ExecuteCommandEvent { input: cmd_text });

            // 履歴に追加
            let text_clone = input.text.clone();
            input.history.push(text_clone);
            input.text.clear();
            input.cursor_position = 0;
            input.history_index = None;
        }

        next_state.set(CommandUiState::Closed);
        return;
    }

    // 文字入力
    for key in keyboard.get_just_pressed() {
        match key {
            KeyCode::Backspace => {
                let cursor_pos = input.cursor_position;
                if cursor_pos > 0 {
                    input.text.remove(cursor_pos - 1);
                    input.cursor_position -= 1;
                }
            }
            KeyCode::Delete => {
                let cursor_pos = input.cursor_position;
                if cursor_pos < input.text.len() {
                    input.text.remove(cursor_pos);
                }
            }
            KeyCode::ArrowLeft => {
                if input.cursor_position > 0 {
                    input.cursor_position -= 1;
                }
            }
            KeyCode::ArrowRight => {
                if input.cursor_position < input.text.len() {
                    input.cursor_position += 1;
                }
            }
            KeyCode::ArrowUp => {
                // 履歴を遡る
                if !input.history.is_empty() {
                    let new_index = match input.history_index {
                        None => Some(input.history.len() - 1),
                        Some(i) if i > 0 => Some(i - 1),
                        Some(i) => Some(i),
                    };
                    if let Some(idx) = new_index {
                        input.text = input.history[idx].clone();
                        input.cursor_position = input.text.len();
                        input.history_index = new_index;
                    }
                }
            }
            KeyCode::ArrowDown => {
                // 履歴を進む
                if let Some(idx) = input.history_index {
                    if idx < input.history.len() - 1 {
                        input.text = input.history[idx + 1].clone();
                        input.cursor_position = input.text.len();
                        input.history_index = Some(idx + 1);
                    } else {
                        input.text.clear();
                        input.cursor_position = 0;
                        input.history_index = None;
                    }
                }
            }
            KeyCode::Space => {
                let cursor_pos = input.cursor_position;
                input.text.insert(cursor_pos, ' ');
                input.cursor_position += 1;
            }
            KeyCode::Tab => {
                // オートコンプリート
                let cmd_text = if input.text.starts_with('/') {
                    &input.text[1..]
                } else {
                    &input.text
                };

                let completions = registry.get_completions(cmd_text);
                if completions.len() == 1 {
                    let parts: Vec<&str> = cmd_text.split_whitespace().collect();

                    if parts.is_empty() || (parts.len() == 1 && !cmd_text.ends_with(' ')) {
                        // コマンド名の補完
                        input.text = completions[0].clone();
                        input.cursor_position = input.text.len();
                    } else {
                        // コマンド引数の補完（最後の引数を置き換える）
                        let mut new_parts = parts[..parts.len() - 1].to_vec();
                        new_parts.push(&completions[0]);
                        input.text = new_parts.join(" ");
                        input.cursor_position = input.text.len();
                    }
                }
            }
            _ => {
                // 通常の文字入力
                if let Some(c) = key_to_char(*key) {
                    let cursor_pos = input.cursor_position;
                    input.text.insert(cursor_pos, c);
                    input.cursor_position += 1;
                }
            }
        }
    }

    // UIテキスト更新
    if let Ok(mut text) = input_text_query.get_single_mut() {
        text.0 = format!("/{}", input.text);
    }

    // オートコンプリート候補表示
    if let Ok(mut text) = suggestions_query.get_single_mut() {
        let cmd_text = if input.text.starts_with('/') {
            &input.text[1..]
        } else {
            &input.text
        };

        let completions = registry.get_completions(cmd_text);
        if !completions.is_empty() {
            text.0 = format!("候補: {}", completions.join(", "));
        } else {
            text.0 = String::new();
        }
    }
}

/// コマンド結果を履歴に追加
fn update_command_results(
    mut result_events: EventReader<CommandResultEvent>,
    mut history: ResMut<CommandHistory>,
    mut history_text_query: Query<&mut Text, With<CommandHistoryText>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for event in result_events.read() {
        history.messages.push((
            event.result.message.clone(),
            event.result.success,
            current_time,
        ));
    }

    // 2秒以上経過したメッセージを削除
    history.messages.retain(|(_, _, timestamp)| {
        current_time - timestamp < 2.0
    });

    // 履歴表示（最新5件、成功/失敗で色分け）
    if let Ok(mut text) = history_text_query.get_single_mut() {
        let recent: Vec<String> = history.messages.iter()
            .rev()
            .take(5)
            .map(|(msg, success, _)| {
                let prefix = if *success { "✓" } else { "✗" };
                format!("{} {}", prefix, msg)
            })
            .collect();
        text.0 = recent.join("\n");
    }
}

/// キーコードを文字に変換
fn key_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        _ => None,
    }
}
