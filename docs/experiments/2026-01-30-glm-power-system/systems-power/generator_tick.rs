//! Generator tick system for fuel consumption
//!
//! This module handles fuel consumption for fuel-based generators.

use bevy::prelude::*;

use crate::components::power::PowerProducer;
use crate::components::Machine;
use crate::game_spec::ProcessType;

/// Update generator fuel consumption
pub fn update_generator_fuel(
    time: Res<Time>,
    mut power_producers: Query<(Entity, &mut PowerProducer)>,
    mut machines: Query<&mut Machine>,
) {
    let delta = time.delta_secs();

    for (entity, mut producer) in &mut power_producers {
        // Skip if not a fuel-based generator
        let fuel_slot = match &producer.fuel_slot {
            Some(slot) => slot,
            None => continue,
        };

        // Check fuel availability
        let has_fuel = fuel_slot.fuel.is_some();

        // Handle startup delay
        if has_fuel && !producer.is_operational {
            producer.startup_timer -= delta;
            if producer.startup_timer <= 0.0 {
                producer.is_operational = true;
            }
        } else if !has_fuel && producer.is_operational {
            producer.is_operational = false;
            producer.startup_timer = fuel_slot.startup_delay;
        }

        // Consume fuel if operational
        if producer.is_operational && has_fuel {
            if let Some((item_id, count)) = fuel_slot.fuel {
                let fuel_to_consume = fuel_slot.consumption_rate * delta;

                if fuel_to_consume >= count as f32 {
                    // Fuel depleted
                    producer.fuel_slot.as_mut().unwrap().fuel = None;
                    producer.is_operational = false;
                } else {
                    // Reduce fuel count
                    let new_count = count - fuel_to_consume.floor() as u32;
                    producer.fuel_slot.as_mut().unwrap().fuel = Some((item_id, new_count));
                }
            }
        }
    }
}

            }
        }
    }
}
