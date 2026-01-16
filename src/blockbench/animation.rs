//! Animation parsing for Blockbench models

use bevy::prelude::*;

use super::raw::{RawAnimation, RawKeyframe};

/// Animation from Blockbench
#[derive(Debug, Clone)]
pub struct Animation {
    /// Animation name
    pub name: String,
    /// Loop mode
    pub loop_mode: LoopMode,
    /// Total animation length in seconds
    pub length: f32,
    /// Keyframes per bone (bone UUID -> keyframes)
    pub bone_keyframes: std::collections::HashMap<String, Vec<Keyframe>>,
}

/// Animation loop mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    /// Play once
    #[default]
    Once,
    /// Loop continuously
    Loop,
    /// Hold on last frame
    Hold,
}

/// Single animation keyframe
#[derive(Debug, Clone)]
pub struct Keyframe {
    /// Which property is being animated
    pub channel: AnimationChannel,
    /// Time in seconds
    pub time: f32,
    /// Value (position/scale as Vec3, rotation as Euler angles in degrees)
    pub value: Vec3,
    /// Interpolation method
    pub interpolation: Interpolation,
}

/// Animation channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationChannel {
    /// Position offset
    Position,
    /// Rotation in Euler angles (degrees)
    Rotation,
    /// Scale factor
    Scale,
}

/// Keyframe interpolation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Interpolation {
    /// Linear interpolation
    #[default]
    Linear,
    /// Catmull-Rom spline
    CatmullRom,
    /// Bezier curve
    Bezier,
    /// Step (no interpolation)
    Step,
}

/// Bone (skeletal hierarchy node) from Blockbench outliner
#[derive(Debug, Clone)]
pub struct Bone {
    /// Bone name
    pub name: String,
    /// Parent bone name (None for root bones)
    pub parent: Option<String>,
    /// Pivot point / origin
    pub origin: Vec3,
    /// Child elements (nested bones or element UUIDs)
    pub children: Vec<BoneChild>,
}

/// Child of a bone - either another bone or an element reference
#[derive(Debug, Clone)]
pub enum BoneChild {
    /// Nested bone
    Bone(Bone),
    /// Element UUID reference
    Element(String),
}

/// Parse animations from raw animation data
pub(crate) fn parse_animations(raw_anims: &[RawAnimation]) -> Vec<Animation> {
    raw_anims.iter().map(parse_animation).collect()
}

/// Parse a single animation
pub(crate) fn parse_animation(raw: &RawAnimation) -> Animation {
    let loop_mode = match raw.loop_mode.as_str() {
        "loop" => LoopMode::Loop,
        "hold" => LoopMode::Hold,
        _ => LoopMode::Once,
    };

    let mut bone_keyframes = std::collections::HashMap::new();

    for (bone_uuid, animator) in &raw.animators {
        let mut keyframes = Vec::new();

        // Parse position keyframes
        for kf in &animator.position {
            if let Some(keyframe) = parse_keyframe(kf, AnimationChannel::Position) {
                keyframes.push(keyframe);
            }
        }

        // Parse rotation keyframes
        for kf in &animator.rotation {
            if let Some(keyframe) = parse_keyframe(kf, AnimationChannel::Rotation) {
                keyframes.push(keyframe);
            }
        }

        // Parse scale keyframes
        for kf in &animator.scale {
            if let Some(keyframe) = parse_keyframe(kf, AnimationChannel::Scale) {
                keyframes.push(keyframe);
            }
        }

        // Sort keyframes by time
        keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if !keyframes.is_empty() {
            bone_keyframes.insert(bone_uuid.clone(), keyframes);
        }
    }

    Animation {
        name: raw.name.clone(),
        loop_mode,
        length: raw.length,
        bone_keyframes,
    }
}

/// Parse a single keyframe
pub(crate) fn parse_keyframe(raw: &RawKeyframe, channel: AnimationChannel) -> Option<Keyframe> {
    // Get value from first data point
    let value = if let Some(dp) = raw.data_points.first() {
        Vec3::new(dp.x, dp.y, dp.z)
    } else {
        return None;
    };

    let interpolation = match raw.interpolation.as_str() {
        "catmullrom" => Interpolation::CatmullRom,
        "bezier" => Interpolation::Bezier,
        "step" => Interpolation::Step,
        _ => Interpolation::Linear,
    };

    Some(Keyframe {
        channel,
        time: raw.time,
        value,
        interpolation,
    })
}

/// Parse outliner array into bone hierarchy
pub(crate) fn parse_outliner(
    outliner: &[serde_json::Value],
    parent_name: Option<&str>,
) -> Vec<Bone> {
    let mut bones = Vec::new();

    for item in outliner {
        match item {
            // String = element UUID reference (not a bone, skip at root level)
            serde_json::Value::String(_) => {
                // Element references at root level are orphaned elements
                // They don't belong to any bone, so we skip them here
            }
            // Object = bone definition
            serde_json::Value::Object(obj) => {
                if let Some(bone) = parse_bone_object(obj, parent_name) {
                    bones.push(bone);
                }
            }
            _ => {}
        }
    }

    bones
}

/// Parse a single bone object from JSON
pub(crate) fn parse_bone_object(
    obj: &serde_json::Map<String, serde_json::Value>,
    parent_name: Option<&str>,
) -> Option<Bone> {
    // Get bone name (required)
    let name = obj.get("name")?.as_str()?.to_string();

    // Get origin (pivot point), default to [0, 0, 0]
    let origin = if let Some(serde_json::Value::Array(arr)) = obj.get("origin") {
        Vec3::new(
            arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        )
    } else {
        Vec3::ZERO
    };

    // Parse children
    let mut children = Vec::new();
    if let Some(serde_json::Value::Array(child_arr)) = obj.get("children") {
        for child in child_arr {
            match child {
                // String = element UUID
                serde_json::Value::String(uuid) => {
                    children.push(BoneChild::Element(uuid.clone()));
                }
                // Object = nested bone
                serde_json::Value::Object(child_obj) => {
                    if let Some(child_bone) = parse_bone_object(child_obj, Some(&name)) {
                        children.push(BoneChild::Bone(child_bone));
                    }
                }
                _ => {}
            }
        }
    }

    Some(Bone {
        name,
        parent: parent_name.map(|s| s.to_string()),
        origin,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockbench::raw::{RawAnimator, RawDataPoint};

    #[test]
    fn test_parse_outliner_simple() {
        // Simple outliner with one bone containing one element
        let outliner_json = r#"[
            {
                "name": "root",
                "origin": [0, 0, 0],
                "children": ["element-uuid-1"]
            }
        ]"#;

        let outliner: Vec<serde_json::Value> = serde_json::from_str(outliner_json).unwrap();
        let bones = parse_outliner(&outliner, None);

        assert_eq!(bones.len(), 1);
        assert_eq!(bones[0].name, "root");
        assert!(bones[0].parent.is_none());
        assert_eq!(bones[0].origin, Vec3::ZERO);
        assert_eq!(bones[0].children.len(), 1);

        match &bones[0].children[0] {
            BoneChild::Element(uuid) => assert_eq!(uuid, "element-uuid-1"),
            BoneChild::Bone(_) => panic!("Expected Element, got Bone"),
        }
    }

    #[test]
    fn test_parse_outliner_nested() {
        // Nested outliner with parent and child bones
        let outliner_json = r#"[
            {
                "name": "root",
                "origin": [0, 0, 0],
                "children": [
                    "element-uuid-1",
                    {
                        "name": "arm",
                        "origin": [4, 8, 0],
                        "children": ["element-uuid-2"]
                    }
                ]
            }
        ]"#;

        let outliner: Vec<serde_json::Value> = serde_json::from_str(outliner_json).unwrap();
        let bones = parse_outliner(&outliner, None);

        assert_eq!(bones.len(), 1);
        let root = &bones[0];
        assert_eq!(root.name, "root");
        assert!(root.parent.is_none());
        assert_eq!(root.children.len(), 2);

        // First child is an element
        match &root.children[0] {
            BoneChild::Element(uuid) => assert_eq!(uuid, "element-uuid-1"),
            BoneChild::Bone(_) => panic!("Expected Element"),
        }

        // Second child is a nested bone
        match &root.children[1] {
            BoneChild::Bone(arm) => {
                assert_eq!(arm.name, "arm");
                assert_eq!(arm.parent, Some("root".to_string()));
                assert_eq!(arm.origin, Vec3::new(4.0, 8.0, 0.0));
                assert_eq!(arm.children.len(), 1);

                match &arm.children[0] {
                    BoneChild::Element(uuid) => assert_eq!(uuid, "element-uuid-2"),
                    BoneChild::Bone(_) => panic!("Expected Element in arm"),
                }
            }
            BoneChild::Element(_) => panic!("Expected Bone"),
        }
    }

    #[test]
    fn test_parse_outliner_empty() {
        let outliner: Vec<serde_json::Value> = vec![];
        let bones = parse_outliner(&outliner, None);
        assert!(bones.is_empty());
    }

    #[test]
    fn test_parse_outliner_root_element_only() {
        // Outliner with only element UUIDs at root level (no bones)
        let outliner_json = r#"["element-uuid-1", "element-uuid-2"]"#;
        let outliner: Vec<serde_json::Value> = serde_json::from_str(outliner_json).unwrap();
        let bones = parse_outliner(&outliner, None);

        // Root-level elements are not bones, so bones should be empty
        assert!(bones.is_empty());
    }

    #[test]
    fn test_parse_animation_simple() {
        // Simple animation with position and rotation keyframes
        let raw = RawAnimation {
            name: "walk".to_string(),
            loop_mode: "loop".to_string(),
            length: 1.0,
            animators: {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "bone-uuid-1".to_string(),
                    RawAnimator {
                        position: vec![
                            crate::blockbench::raw::RawKeyframe {
                                time: 0.0,
                                data_points: vec![RawDataPoint {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "linear".to_string(),
                            },
                            crate::blockbench::raw::RawKeyframe {
                                time: 0.5,
                                data_points: vec![RawDataPoint {
                                    x: 0.0,
                                    y: 1.0,
                                    z: 0.0,
                                }],
                                interpolation: "linear".to_string(),
                            },
                        ],
                        rotation: vec![crate::blockbench::raw::RawKeyframe {
                            time: 0.0,
                            data_points: vec![RawDataPoint {
                                x: 0.0,
                                y: 45.0,
                                z: 0.0,
                            }],
                            interpolation: "catmullrom".to_string(),
                        }],
                        scale: vec![],
                    },
                );
                map
            },
        };

        let anim = parse_animation(&raw);

        assert_eq!(anim.name, "walk");
        assert_eq!(anim.loop_mode, LoopMode::Loop);
        assert_eq!(anim.length, 1.0);
        assert_eq!(anim.bone_keyframes.len(), 1);

        let keyframes = anim.bone_keyframes.get("bone-uuid-1").unwrap();
        assert_eq!(keyframes.len(), 3); // 2 position + 1 rotation

        // Keyframes should be sorted by time
        assert_eq!(keyframes[0].time, 0.0);
        assert_eq!(keyframes[1].time, 0.0);
        assert_eq!(keyframes[2].time, 0.5);
    }

    #[test]
    fn test_parse_animation_all_interpolation_types() {
        // Test all interpolation types
        let raw = RawAnimation {
            name: "test".to_string(),
            loop_mode: "once".to_string(),
            length: 2.0,
            animators: {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    "bone-1".to_string(),
                    RawAnimator {
                        position: vec![
                            crate::blockbench::raw::RawKeyframe {
                                time: 0.0,
                                data_points: vec![RawDataPoint {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "linear".to_string(),
                            },
                            crate::blockbench::raw::RawKeyframe {
                                time: 0.5,
                                data_points: vec![RawDataPoint {
                                    x: 1.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "catmullrom".to_string(),
                            },
                            crate::blockbench::raw::RawKeyframe {
                                time: 1.0,
                                data_points: vec![RawDataPoint {
                                    x: 2.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "bezier".to_string(),
                            },
                            crate::blockbench::raw::RawKeyframe {
                                time: 1.5,
                                data_points: vec![RawDataPoint {
                                    x: 3.0,
                                    y: 0.0,
                                    z: 0.0,
                                }],
                                interpolation: "step".to_string(),
                            },
                        ],
                        rotation: vec![],
                        scale: vec![crate::blockbench::raw::RawKeyframe {
                            time: 0.0,
                            data_points: vec![RawDataPoint {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            }],
                            interpolation: "linear".to_string(),
                        }],
                    },
                );
                map
            },
        };

        let anim = parse_animation(&raw);

        assert_eq!(anim.name, "test");
        assert_eq!(anim.loop_mode, LoopMode::Once);
        assert_eq!(anim.length, 2.0);

        let keyframes = anim.bone_keyframes.get("bone-1").unwrap();
        assert_eq!(keyframes.len(), 5); // 4 position + 1 scale

        // Check interpolation types (keyframes are sorted by time)
        assert_eq!(keyframes[0].interpolation, Interpolation::Linear); // position at 0.0
        assert_eq!(keyframes[1].interpolation, Interpolation::Linear); // scale at 0.0
        assert_eq!(keyframes[2].interpolation, Interpolation::CatmullRom); // position at 0.5
        assert_eq!(keyframes[3].interpolation, Interpolation::Bezier); // position at 1.0
        assert_eq!(keyframes[4].interpolation, Interpolation::Step); // position at 1.5

        // Check channels
        assert_eq!(keyframes[0].channel, AnimationChannel::Position);
        assert_eq!(keyframes[1].channel, AnimationChannel::Scale);
        assert_eq!(keyframes[2].channel, AnimationChannel::Position);

        // Check values
        assert_eq!(keyframes[0].value, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(keyframes[1].value, Vec3::new(1.0, 1.0, 1.0)); // scale
        assert_eq!(keyframes[2].value, Vec3::new(1.0, 0.0, 0.0));
    }
}
