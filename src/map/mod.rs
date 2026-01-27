//! Map system for world overview

use bevy::prelude::*;
use std::collections::HashSet;

/// マーカータイプ
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarkerType {
    Player,
    Machine,
    DeliveryPlatform,
    Waypoint,
    Custom,
}

/// マップマーカー
#[derive(Debug, Clone)]
pub struct MapMarker {
    pub position: IVec3,
    pub marker_type: MarkerType,
    pub label: Option<String>,
    pub color: Option<Color>,
}

impl MapMarker {
    pub fn new(position: IVec3, marker_type: MarkerType) -> Self {
        Self {
            position,
            marker_type,
            label: None,
            color: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// マップデータリソース
#[derive(Resource, Debug)]
pub struct MapData {
    /// 探索済みチャンク
    pub explored_chunks: HashSet<IVec2>,
    /// マーカー一覧
    pub markers: Vec<MapMarker>,
    /// マップ表示状態
    pub is_visible: bool,
    /// ズームレベル (1.0 = 通常)
    pub zoom: f32,
    /// マップ中心位置
    pub center: IVec2,
}

impl Default for MapData {
    fn default() -> Self {
        Self::new()
    }
}

impl MapData {
    pub fn new() -> Self {
        Self {
            explored_chunks: HashSet::new(),
            markers: Vec::new(),
            is_visible: false,
            zoom: 1.0,
            center: IVec2::ZERO,
        }
    }

    /// チャンクを探索済みにする
    pub fn explore_chunk(&mut self, chunk: IVec2) {
        self.explored_chunks.insert(chunk);
    }

    /// チャンクが探索済みか確認
    pub fn is_explored(&self, chunk: IVec2) -> bool {
        self.explored_chunks.contains(&chunk)
    }

    /// マーカーを追加
    pub fn add_marker(&mut self, marker: MapMarker) {
        self.markers.push(marker);
    }

    /// 位置からマーカーを削除
    pub fn remove_marker_at(&mut self, position: IVec3) {
        self.markers.retain(|m| m.position != position);
    }

    /// 探索済みチャンク数
    pub fn explored_count(&self) -> usize {
        self.explored_chunks.len()
    }

    /// ズームイン
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.25).min(4.0);
    }

    /// ズームアウト
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.25).max(0.25);
    }
}

/// マップ表示トグルイベント
#[derive(Message, Debug)]
pub struct ToggleMap;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapData>().add_message::<ToggleMap>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_data_default() {
        let map = MapData::default();
        assert!(!map.is_visible);
        assert_eq!(map.zoom, 1.0);
        assert_eq!(map.explored_count(), 0);
    }

    #[test]
    fn test_explore_chunk() {
        let mut map = MapData::new();
        map.explore_chunk(IVec2::new(0, 0));
        map.explore_chunk(IVec2::new(1, 0));

        assert!(map.is_explored(IVec2::new(0, 0)));
        assert!(!map.is_explored(IVec2::new(2, 0)));
        assert_eq!(map.explored_count(), 2);
    }

    #[test]
    fn test_markers() {
        let mut map = MapData::new();
        let marker = MapMarker::new(IVec3::new(10, 5, 20), MarkerType::Machine).with_label("Miner");

        map.add_marker(marker);
        assert_eq!(map.markers.len(), 1);

        map.remove_marker_at(IVec3::new(10, 5, 20));
        assert_eq!(map.markers.len(), 0);
    }

    #[test]
    fn test_zoom() {
        let mut map = MapData::new();
        map.zoom_in();
        assert!(map.zoom > 1.0);

        map.zoom = 1.0;
        map.zoom_out();
        assert!(map.zoom < 1.0);
    }
}
