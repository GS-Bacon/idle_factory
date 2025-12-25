//! ネットワークモジュール（将来実装予定）
//!
//! このモジュールはマルチプレイヤー機能のスタブです。
//! 現在は未実装で、Phase 1-5には含まれません。
//!
//! ## 実装予定
//! - サーバー権威型アーキテクチャ
//! - クライアント予測と補間
//! - lightyearライブラリの統合
//!
//! ## 参照
//! - `.specify/specs/index-compact.md` の `mp(future)` セクション

pub mod messages;
pub mod client;
pub mod server;

use bevy::prelude::*;

/// ネットワークプラグイン（将来実装予定）
///
/// 現在は何も行いません。マルチプレイヤー実装時に有効化されます。
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, _app: &mut App) {
        // 将来実装予定: マルチプレイヤーネットワーキング
        // 現在はスタブのみ
    }
}
