//! Guarded event writer with depth protection

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::{EventDepth, EventError, EventSystemConfig};

/// 安全なイベント送信（深さチェック付き）
///
/// 通常のEventWriterの代わりにこれを使うことで、
/// イベント連鎖の無限ループを防止できる。
#[derive(SystemParam)]
pub struct GuardedEventWriter<'w, E: Event> {
    writer: EventWriter<'w, E>,
    depth: ResMut<'w, EventDepth>,
    config: Res<'w, EventSystemConfig>,
}

impl<E: Event> GuardedEventWriter<'_, E> {
    /// 深さチェック付きイベント送信
    pub fn send(&mut self, event: E) -> Result<(), EventError> {
        if self.depth.0 >= self.config.max_depth {
            error!(
                "Event depth exceeded (max: {}): {:?}",
                self.config.max_depth,
                std::any::type_name::<E>()
            );
            return Err(EventError::MaxDepthExceeded);
        }
        self.depth.0 += 1;
        if self.config.log_enabled {
            debug!(
                "Event sent (depth {}): {:?}",
                self.depth.0,
                std::any::type_name::<E>()
            );
        }
        self.writer.send(event);
        Ok(())
    }

    /// 深さチェック付き複数イベント送信
    pub fn send_batch(&mut self, events: impl IntoIterator<Item = E>) -> Result<(), EventError> {
        for event in events {
            self.send(event)?;
        }
        Ok(())
    }

    /// 深さチェックなしで送信（内部用、注意して使用）
    pub fn send_unchecked(&mut self, event: E) {
        self.writer.send(event);
    }

    /// 現在の深さを取得
    pub fn current_depth(&self) -> u8 {
        self.depth.0
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
