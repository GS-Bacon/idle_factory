//! Bevyシステム関数

use bevy::prelude::*;

use super::types::*;

/// 詰まり検出システム
pub fn detect_stuck_points(
    time: Res<Time>,
    mut recorder: ResMut<PlaySessionRecorder>,
    config: Res<EvaluationConfig>,
    mut stuck_events: EventWriter<StuckDetectedEvent>,
) {
    if recorder.current_session.is_none() {
        return;
    }

    recorder.stuck_timer += time.delta_secs();

    // 閾値を超えたら詰まりとして検出
    if recorder.stuck_timer >= config.stuck_threshold_secs {
        let stuck_point = StuckPoint {
            location: recorder.current_location.clone(),
            duration_secs: recorder.stuck_timer,
            attempted_actions: recorder.stuck_actions.clone(),
            resolution: None,
            timestamp: time.elapsed_secs(),
        };

        if let Some(session) = &mut recorder.current_session {
            session.stats.stuck_points.push(stuck_point.clone());
            session.stats.confusion_moments += 1;
        }

        stuck_events.send(StuckDetectedEvent {
            location: stuck_point.location,
            duration_secs: stuck_point.duration_secs,
            attempted_actions: stuck_point.attempted_actions,
        });

        // 検出後はタイマーをリセット（連続検出を防ぐ）
        recorder.stuck_timer = 0.0;
        recorder.stuck_actions.clear();
    }
}

/// 目標条件チェックシステム
pub fn check_goal_conditions(
    mut recorder: ResMut<PlaySessionRecorder>,
    time: Res<Time>,
    mut goal_events: EventWriter<GoalCompletedEvent>,
    // 必要に応じて他のクエリを追加
) {
    if recorder.current_session.is_none() {
        return;
    }

    let elapsed = time.elapsed_secs();
    let mut completed_goals = vec![];

    for goal in &mut recorder.active_goals {
        if goal.completed_at.is_some() {
            continue;
        }

        // 目標条件をチェック（簡略化版 - 実際にはより詳細なチェックが必要）
        let achieved = match &goal.success_condition {
            GoalCondition::OpenedUI { ui_name: _ } => {
                // UI状態をチェック
                false // 実際の実装では状態を確認
            }
            GoalCondition::PressedKey { key: _ } => {
                false // 実際の実装ではキー入力を確認
            }
            _ => false,
        };

        if achieved {
            let started = goal.started_at.unwrap_or(0.0);
            let time_taken = elapsed - started;
            goal.completed_at = Some(elapsed);

            completed_goals.push((goal.id.clone(), time_taken));
        }
    }

    for (goal_id, time_taken) in completed_goals {
        goal_events.send(GoalCompletedEvent {
            goal_id,
            time_taken_secs: time_taken,
        });
    }
}

/// ゲームイベント記録システム
pub fn record_game_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut recorder: ResMut<PlaySessionRecorder>,
) {
    if recorder.current_session.is_none() {
        return;
    }

    let elapsed = time.elapsed_secs();

    // キー入力を記録
    for key in keyboard.get_just_pressed() {
        let key_name = format!("{:?}", key);
        let context = recorder.current_location.clone();
        recorder.record_event(GameEvent {
            time: elapsed,
            event_type: GameEventType::KeyPress(key_name.clone()),
            context,
            success: true,
            related_goal: None,
        });
        recorder.stuck_actions.push(format!("press_{}", key_name));
    }

    // マウスクリックを記録
    for button in mouse.get_just_pressed() {
        let button_name = format!("{:?}", button);
        let context = recorder.current_location.clone();
        recorder.record_event(GameEvent {
            time: elapsed,
            event_type: GameEventType::MouseClick {
                button: button_name.clone(),
                x: 0.0, // 実際にはカーソル位置を取得
                y: 0.0,
            },
            context,
            success: true,
            related_goal: None,
        });
        recorder.stuck_actions.push(format!("click_{}", button_name));
    }
}

/// 効率メトリクス更新システム
pub fn update_efficiency_metrics(
    time: Res<Time>,
    mut recorder: ResMut<PlaySessionRecorder>,
) {
    if let Some(session) = &mut recorder.current_session {
        session.duration_secs = time.elapsed_secs();
    }
}
