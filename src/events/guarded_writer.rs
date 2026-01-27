//! Guarded event writer with depth protection

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{EventDepth, EventError, EventSystemConfig};

/// 安全なイベント送信（深さチェック付き）
///
/// 通常のMessageWriterの代わりにこれを使うことで、
/// イベント連鎖の無限ループを防止できる。
///
/// EventDepthはAtomicU8を使用しているため、同一システム内で
/// 複数のGuardedMessageWriterを使用可能（Resで取得）。
#[derive(SystemParam)]
pub struct GuardedMessageWriter<'w, E: Message> {
    writer: MessageWriter<'w, E>,
    depth: Res<'w, EventDepth>,
    config: Res<'w, EventSystemConfig>,
}

impl<E: Message> GuardedMessageWriter<'_, E> {
    /// 深さチェック付きメッセージ送信
    pub fn write(&mut self, event: E) -> Result<(), EventError> {
        let current = self.depth.get();
        if current >= self.config.max_depth {
            error!(
                "Event depth exceeded (max: {}): {:?}",
                self.config.max_depth,
                std::any::type_name::<E>()
            );
            return Err(EventError::MaxDepthExceeded);
        }
        let new_depth = self.depth.increment();
        if self.config.log_enabled {
            debug!(
                "Event sent (depth {}): {:?}",
                new_depth,
                std::any::type_name::<E>()
            );
        }
        self.writer.write(event);
        Ok(())
    }

    /// 深さチェック付き複数メッセージ送信
    pub fn write_batch(&mut self, events: impl IntoIterator<Item = E>) -> Result<(), EventError> {
        for event in events {
            self.write(event)?;
        }
        Ok(())
    }

    /// 深さチェックなしで送信（内部用、注意して使用）
    pub fn write_unchecked(&mut self, event: E) {
        self.writer.write(event);
    }

    /// 現在の深さを取得
    pub fn current_depth(&self) -> u8 {
        self.depth.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_error_debug() {
        let err = EventError::MaxDepthExceeded;
        assert_eq!(format!("{:?}", err), "MaxDepthExceeded");
    }
}
