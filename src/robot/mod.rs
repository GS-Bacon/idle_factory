//! Robot automation system for autonomous factory operations

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::components::Direction;
use crate::core::ItemId;
use crate::utils::GridPos;

/// ロボットの種類
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum RobotType {
    /// 建設ロボット（ブロック配置）
    Builder,
    /// 採掘ロボット（ブロック破壊）
    Miner,
    /// 輸送ロボット（アイテム運搬）
    Transporter,
    /// 修理ロボット（機械メンテナンス）
    Repairer,
}

impl RobotType {
    /// ロボット名を取得
    pub fn name(&self) -> &'static str {
        match self {
            RobotType::Builder => "Builder Robot",
            RobotType::Miner => "Mining Robot",
            RobotType::Transporter => "Transport Robot",
            RobotType::Repairer => "Repair Robot",
        }
    }

    /// 基本速度を取得
    pub fn base_speed(&self) -> f32 {
        match self {
            RobotType::Builder => 2.0,
            RobotType::Miner => 1.5,
            RobotType::Transporter => 3.0,
            RobotType::Repairer => 2.5,
        }
    }

    /// エネルギー消費率を取得
    pub fn energy_consumption(&self) -> f32 {
        match self {
            RobotType::Builder => 1.0,
            RobotType::Miner => 1.5,
            RobotType::Transporter => 0.8,
            RobotType::Repairer => 1.2,
        }
    }
}

/// ロボットの命令
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RobotCommand {
    /// 移動
    MoveTo(GridPos),
    /// ブロック配置
    PlaceBlock(GridPos, ItemId),
    /// ブロック破壊
    BreakBlock(GridPos),
    /// アイテム収集
    PickupItem(GridPos),
    /// アイテム投下
    DropItem(GridPos),
    /// 待機（秒）
    Wait(f32),
    /// 機械修理
    RepairMachine(Entity),
    /// 指定位置で向きを変える
    Face(Direction),
}

/// ロボットの状態
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum RobotState {
    /// 待機中
    #[default]
    Idle,
    /// 移動中
    Moving,
    /// 作業中
    Working,
    /// 充電中
    Charging,
    /// エラー状態
    Error,
}

/// ロボットコンポーネント
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Robot {
    /// ロボットの種類
    pub robot_type: RobotType,
    /// 現在の状態
    pub state: RobotState,
    /// 現在のエネルギー（0-100）
    pub energy: f32,
    /// 最大エネルギー
    pub max_energy: f32,
    /// 移動速度倍率
    pub speed_multiplier: f32,
    /// 作業効率倍率
    pub efficiency_multiplier: f32,
    /// ホームポジション（充電ステーション）
    pub home_position: Option<GridPos>,
}

impl Robot {
    /// 新しいロボットを作成
    pub fn new(robot_type: RobotType) -> Self {
        Self {
            robot_type,
            state: RobotState::Idle,
            energy: 100.0,
            max_energy: 100.0,
            speed_multiplier: 1.0,
            efficiency_multiplier: 1.0,
            home_position: None,
        }
    }

    /// 実効移動速度を取得
    pub fn effective_speed(&self) -> f32 {
        self.robot_type.base_speed() * self.speed_multiplier
    }

    /// エネルギーを消費
    pub fn consume_energy(&mut self, amount: f32) -> bool {
        let consumption = amount * self.robot_type.energy_consumption();
        if self.energy >= consumption {
            self.energy -= consumption;
            true
        } else {
            false
        }
    }

    /// エネルギーを充電
    pub fn charge(&mut self, amount: f32) {
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    /// バッテリー残量（0.0-1.0）
    pub fn battery_level(&self) -> f32 {
        self.energy / self.max_energy
    }

    /// 低バッテリーか
    pub fn is_low_battery(&self) -> bool {
        self.battery_level() < 0.2
    }

    /// ホームポジションを設定
    pub fn with_home(mut self, pos: GridPos) -> Self {
        self.home_position = Some(pos);
        self
    }
}

/// ロボットのコマンドキュー
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct RobotCommandQueue {
    /// コマンドキュー
    pub commands: VecDeque<RobotCommand>,
    /// 現在実行中のコマンド
    pub current: Option<RobotCommand>,
    /// 現在コマンドの進捗（0.0-1.0）
    pub progress: f32,
    /// ループモード
    pub loop_mode: bool,
    /// 実行済みコマンド（ループ用）
    executed: Vec<RobotCommand>,
}

impl RobotCommandQueue {
    /// コマンドを追加
    pub fn push(&mut self, command: RobotCommand) {
        self.commands.push_back(command);
    }

    /// 複数コマンドを追加
    pub fn push_all(&mut self, commands: impl IntoIterator<Item = RobotCommand>) {
        for cmd in commands {
            self.commands.push_back(cmd);
        }
    }

    /// 次のコマンドを開始
    pub fn start_next(&mut self) -> Option<&RobotCommand> {
        if let Some(current) = self.current.take() {
            if self.loop_mode {
                self.executed.push(current);
            }
        }

        self.progress = 0.0;

        if let Some(cmd) = self.commands.pop_front() {
            self.current = Some(cmd);
            self.current.as_ref()
        } else if self.loop_mode && !self.executed.is_empty() {
            // ループモード: 実行済みコマンドを復元
            self.commands = self.executed.drain(..).collect();
            self.start_next()
        } else {
            None
        }
    }

    /// 現在のコマンドを完了
    pub fn complete_current(&mut self) {
        if let Some(current) = self.current.take() {
            if self.loop_mode {
                self.executed.push(current);
            }
        }
        self.progress = 0.0;
    }

    /// キューをクリア
    pub fn clear(&mut self) {
        self.commands.clear();
        self.current = None;
        self.progress = 0.0;
        self.executed.clear();
    }

    /// 残りコマンド数
    pub fn remaining(&self) -> usize {
        self.commands.len() + if self.current.is_some() { 1 } else { 0 }
    }

    /// アイドル状態か
    pub fn is_idle(&self) -> bool {
        self.current.is_none() && self.commands.is_empty()
    }
}

/// ロボットインベントリ（輸送ロボット用）
#[derive(Component, Debug, Clone, Default, Serialize, Deserialize)]
pub struct RobotInventory {
    /// 保持アイテム
    pub items: Vec<(ItemId, u32)>,
    /// 最大容量
    pub capacity: u32,
}

impl RobotInventory {
    /// 新しいインベントリを作成
    pub fn new(capacity: u32) -> Self {
        Self {
            items: Vec::new(),
            capacity,
        }
    }

    /// 現在の使用量
    pub fn used(&self) -> u32 {
        self.items.iter().map(|(_, count)| count).sum()
    }

    /// 空き容量
    pub fn free(&self) -> u32 {
        self.capacity.saturating_sub(self.used())
    }

    /// アイテムを追加
    pub fn add(&mut self, item_id: ItemId, count: u32) -> u32 {
        let can_add = count.min(self.free());
        if can_add > 0 {
            if let Some((_, existing)) = self.items.iter_mut().find(|(id, _)| *id == item_id) {
                *existing += can_add;
            } else {
                self.items.push((item_id, can_add));
            }
        }
        can_add
    }

    /// アイテムを取り出す
    pub fn remove(&mut self, item_id: ItemId, count: u32) -> u32 {
        if let Some(idx) = self.items.iter().position(|(id, _)| *id == item_id) {
            let (_, existing) = &mut self.items[idx];
            let can_remove = count.min(*existing);
            *existing -= can_remove;
            if *existing == 0 {
                self.items.remove(idx);
            }
            can_remove
        } else {
            0
        }
    }

    /// 空か
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// ロボット生成イベント
#[derive(Message)]
pub struct SpawnRobotEvent {
    /// ロボットの種類
    pub robot_type: RobotType,
    /// 生成位置
    pub position: GridPos,
    /// ホームポジション
    pub home: Option<GridPos>,
}

/// ロボットコマンド完了イベント
#[derive(Message)]
pub struct RobotCommandCompletedEvent {
    /// ロボットエンティティ
    pub robot: Entity,
    /// 完了したコマンド
    pub command: RobotCommand,
    /// 成功したか
    pub success: bool,
}

/// ロボットプラグイン
pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnRobotEvent>()
            .add_message::<RobotCommandCompletedEvent>()
            .add_systems(Update, update_robots);
    }
}

/// ロボット更新システム
fn update_robots(
    time: Res<Time>,
    mut robots: Query<(Entity, &mut Robot, &mut RobotCommandQueue)>,
    mut completed_events: MessageWriter<RobotCommandCompletedEvent>,
) {
    for (entity, mut robot, mut queue) in robots.iter_mut() {
        // エネルギー消費チェック
        if robot.is_low_battery() && robot.state != RobotState::Charging {
            robot.state = RobotState::Idle;
            continue;
        }

        // 現在のコマンドを処理
        if let Some(cmd) = &queue.current {
            let dt = time.delta_secs();

            match cmd {
                RobotCommand::Wait(duration) => {
                    queue.progress += dt / duration;
                    if queue.progress >= 1.0 {
                        let cmd = queue.current.clone().unwrap();
                        queue.complete_current();
                        completed_events.write(RobotCommandCompletedEvent {
                            robot: entity,
                            command: cmd,
                            success: true,
                        });
                    }
                }
                _ => {
                    // 他のコマンドは実際の物理シミュレーションで処理
                    // ここでは進捗を進めるだけ
                    let work_speed = robot.effective_speed() * robot.efficiency_multiplier;
                    queue.progress += dt * work_speed * 0.5;

                    if queue.progress >= 1.0 {
                        robot.consume_energy(1.0);
                        let cmd = queue.current.clone().unwrap();
                        queue.complete_current();
                        completed_events.write(RobotCommandCompletedEvent {
                            robot: entity,
                            command: cmd,
                            success: true,
                        });
                    }
                }
            }
        } else {
            // 次のコマンドを開始
            if queue.start_next().is_some() {
                robot.state = RobotState::Working;
            } else {
                robot.state = RobotState::Idle;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::items;

    #[test]
    fn test_robot_type_properties() {
        let builder = RobotType::Builder;
        assert_eq!(builder.name(), "Builder Robot");
        assert!(builder.base_speed() > 0.0);
        assert!(builder.energy_consumption() > 0.0);
    }

    #[test]
    fn test_robot_new() {
        let robot = Robot::new(RobotType::Miner);

        assert_eq!(robot.robot_type, RobotType::Miner);
        assert_eq!(robot.state, RobotState::Idle);
        assert_eq!(robot.energy, 100.0);
        assert!(robot.home_position.is_none());
    }

    #[test]
    fn test_robot_energy() {
        let mut robot = Robot::new(RobotType::Builder);

        assert!(!robot.is_low_battery());
        assert!(robot.consume_energy(50.0));
        assert_eq!(robot.battery_level(), 0.5);

        robot.charge(30.0);
        assert_eq!(robot.energy, 80.0);

        // 過充電しない
        robot.charge(50.0);
        assert_eq!(robot.energy, 100.0);
    }

    #[test]
    fn test_robot_low_battery() {
        let mut robot = Robot::new(RobotType::Builder);
        robot.energy = 15.0;

        assert!(robot.is_low_battery());
    }

    #[test]
    fn test_robot_command_queue() {
        let mut queue = RobotCommandQueue::default();

        queue.push(RobotCommand::Wait(1.0));
        queue.push(RobotCommand::Wait(2.0));

        assert_eq!(queue.remaining(), 2);
        assert!(!queue.is_idle());

        let cmd = queue.start_next();
        assert!(cmd.is_some());
        assert_eq!(queue.remaining(), 2); // 現在 + キュー1個

        queue.complete_current();
        assert_eq!(queue.remaining(), 1);
    }

    #[test]
    fn test_robot_command_queue_loop() {
        let mut queue = RobotCommandQueue::default();
        queue.loop_mode = true;

        queue.push(RobotCommand::Wait(1.0));
        queue.push(RobotCommand::Wait(2.0));

        // 最初のサイクル
        queue.start_next();
        queue.complete_current();
        queue.start_next();
        queue.complete_current();

        // ループして戻る
        let cmd = queue.start_next();
        assert!(cmd.is_some());
    }

    #[test]
    fn test_robot_inventory() {
        let mut inv = RobotInventory::new(100);

        assert_eq!(inv.add(items::stone(), 50), 50);
        assert_eq!(inv.used(), 50);
        assert_eq!(inv.free(), 50);

        assert_eq!(inv.add(items::coal(), 60), 50); // 50だけ入る
        assert_eq!(inv.free(), 0);

        assert_eq!(inv.remove(items::stone(), 30), 30);
        assert_eq!(inv.free(), 30);
    }

    #[test]
    fn test_robot_inventory_remove_all() {
        let mut inv = RobotInventory::new(100);

        inv.add(items::stone(), 50);
        assert_eq!(inv.remove(items::stone(), 50), 50);
        assert!(inv.is_empty());
    }

    #[test]
    fn test_robot_with_home() {
        let robot = Robot::new(RobotType::Transporter).with_home(GridPos::new(10, 5, 10));

        assert!(robot.home_position.is_some());
        assert_eq!(robot.home_position.unwrap(), GridPos::new(10, 5, 10));
    }

    #[test]
    fn test_robot_state_values() {
        let states = [
            RobotState::Idle,
            RobotState::Moving,
            RobotState::Working,
            RobotState::Charging,
            RobotState::Error,
        ];

        for state in states {
            let mut robot = Robot::new(RobotType::Builder);
            robot.state = state;
            assert_eq!(robot.state, state);
        }
    }
}
