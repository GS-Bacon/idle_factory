use bevy::prelude::*;
use crate::gameplay::grid::{SimulationGrid, Machine};
use crate::gameplay::interaction::PlayerInteractEvent;
use crate::core::registry::RecipeRegistry;

/// UI state for machine interaction
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum MachineUiState {
    #[default]
    Closed,
    Open,
}

/// Resource tracking which machine is currently being edited
#[derive(Resource, Default)]
pub struct OpenMachineUi {
    pub target_pos: Option<IVec3>,
}

/// Marker component for the machine UI root
#[derive(Component)]
pub struct MachineUiRoot;

/// Marker component for recipe buttons
#[derive(Component)]
pub struct RecipeButton {
    pub recipe_id: String,
}

/// Marker component for close button
#[derive(Component)]
pub struct CloseButton;

/// Marker component for inventory display
#[derive(Component)]
pub struct InventoryDisplay {
    pub slot_type: InventorySlotType,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InventorySlotType {
    Input,
    Output,
}

/// Event to open machine UI
#[derive(Event)]
pub struct OpenMachineUiEvent {
    pub pos: IVec3,
}

/// Event to close machine UI
#[derive(Event)]
pub struct CloseMachineUiEvent;

/// Event to set recipe
#[derive(Event)]
pub struct SetRecipeEvent {
    pub pos: IVec3,
    pub recipe_id: String,
}

pub struct MachineUiPlugin;

impl Plugin for MachineUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<MachineUiState>()
            .init_resource::<OpenMachineUi>()
            .add_event::<OpenMachineUiEvent>()
            .add_event::<CloseMachineUiEvent>()
            .add_event::<SetRecipeEvent>()
            .add_systems(Update, (
                handle_machine_interaction,
                handle_open_ui_event,
                handle_close_ui_event,
                handle_recipe_button_click,
                handle_close_button_click,
                update_inventory_display,
                handle_escape_key,
            ))
            .add_systems(OnEnter(MachineUiState::Open), spawn_machine_ui)
            .add_systems(OnExit(MachineUiState::Open), despawn_machine_ui)
            .add_systems(Update, apply_recipe_event);
    }
}

/// Handle right-click interaction to open machine UI
/// NOTE: 現在は無効化（Assemblerは自動レシピ検索を使用）
fn handle_machine_interaction(
    _events: EventReader<PlayerInteractEvent>,
    _grid: Res<SimulationGrid>,
    _open_events: EventWriter<OpenMachineUiEvent>,
    _state: Res<State<MachineUiState>>,
) {
    // Assemblerは自動レシピ検索を使用するため、UIは無効化
    // 将来的に他の機械でUIが必要になる場合は、ここを変更
    /*
    for event in events.read() {
        if event.mouse_button != MouseButton::Right { continue; }

        if let Some(machine) = grid.machines.get(&event.grid_pos) {
            if matches!(machine.machine_type, Machine::Assembler(_)) {
                if *state.get() == MachineUiState::Closed {
                    open_events.send(OpenMachineUiEvent { pos: event.grid_pos });
                }
            }
        }
    }
    */
}

/// Handle open UI event
fn handle_open_ui_event(
    mut events: EventReader<OpenMachineUiEvent>,
    mut open_machine: ResMut<OpenMachineUi>,
    mut next_state: ResMut<NextState<MachineUiState>>,
) {
    for event in events.read() {
        open_machine.target_pos = Some(event.pos);
        next_state.set(MachineUiState::Open);
    }
}

/// Handle close UI event
fn handle_close_ui_event(
    mut events: EventReader<CloseMachineUiEvent>,
    mut open_machine: ResMut<OpenMachineUi>,
    mut next_state: ResMut<NextState<MachineUiState>>,
) {
    for _ in events.read() {
        open_machine.target_pos = None;
        next_state.set(MachineUiState::Closed);
    }
}

/// Handle escape key to close UI
fn handle_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<MachineUiState>>,
    mut close_events: EventWriter<CloseMachineUiEvent>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && *state.get() == MachineUiState::Open {
        close_events.send(CloseMachineUiEvent);
    }
}

/// Spawn the machine UI
fn spawn_machine_ui(
    mut commands: Commands,
    open_machine: Res<OpenMachineUi>,
    grid: Res<SimulationGrid>,
    recipes: Res<RecipeRegistry>,
) {
    let Some(pos) = open_machine.target_pos else { return };
    let Some(machine) = grid.machines.get(&pos) else { return };
    let Machine::Assembler(assembler) = &machine.machine_type else { return };

    // Root container - full screen overlay
    commands.spawn((
        MachineUiRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
    )).with_children(|parent| {
        // Main panel
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(350.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        )).with_children(|panel| {
            // Header row
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
            )).with_children(|header| {
                // Title
                header.spawn((
                    Text::new("Assembler Settings"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                // Close button
                header.spawn((
                    CloseButton,
                    Button,
                    Node {
                        width: Val::Px(30.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new("X"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });

            // Current recipe display
            let current_recipe_text = assembler.active_recipe
                .as_ref()
                .and_then(|id| recipes.map.get(id))
                .map(|r| format!("Current: {}", r.name))
                .unwrap_or_else(|| "Current: None".to_string());

            panel.spawn((
                Text::new(current_recipe_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Recipe selection label
            panel.spawn((
                Text::new("Select Recipe:"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Recipe buttons container
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
            )).with_children(|recipe_container| {
                for (recipe_id, recipe) in recipes.map.iter() {
                    let is_selected = assembler.active_recipe.as_ref() == Some(recipe_id);
                    let bg_color = if is_selected {
                        Color::srgb(0.2, 0.5, 0.3)
                    } else {
                        Color::srgb(0.3, 0.3, 0.35)
                    };

                    recipe_container.spawn((
                        RecipeButton { recipe_id: recipe_id.clone() },
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(bg_color),
                    )).with_children(|btn| {
                        let inputs_str: Vec<String> = recipe.inputs.iter()
                            .map(|i| format!("{}x{}", i.count, i.item))
                            .collect();
                        let outputs_str: Vec<String> = recipe.outputs.iter()
                            .map(|o| format!("{}x{}", o.count, o.item))
                            .collect();
                        let label = format!("{}: {} -> {}",
                            recipe.name,
                            inputs_str.join(", "),
                            outputs_str.join(", ")
                        );

                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });

            // Inventory display
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            )).with_children(|inv_row| {
                // Input inventory
                inv_row.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                )).with_children(|input_col| {
                    input_col.spawn((
                        Text::new("Input:"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));

                    let input_text = if assembler.input_inventory.is_empty() {
                        "Empty".to_string()
                    } else {
                        assembler.input_inventory.iter()
                            .map(|s| format!("{}x{}", s.count, s.item_id))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };

                    input_col.spawn((
                        InventoryDisplay { slot_type: InventorySlotType::Input },
                        Text::new(input_text),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

                // Output inventory
                inv_row.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                )).with_children(|output_col| {
                    output_col.spawn((
                        Text::new("Output:"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));

                    let output_text = if assembler.output_inventory.is_empty() {
                        "Empty".to_string()
                    } else {
                        assembler.output_inventory.iter()
                            .map(|s| format!("{}x{}", s.count, s.item_id))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };

                    output_col.spawn((
                        InventoryDisplay { slot_type: InventorySlotType::Output },
                        Text::new(output_text),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
    });
}

/// Despawn the machine UI
fn despawn_machine_ui(
    mut commands: Commands,
    query: Query<Entity, With<MachineUiRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Handle recipe button clicks
fn handle_recipe_button_click(
    mut interaction_query: Query<
        (&Interaction, &RecipeButton, &mut BackgroundColor),
        Changed<Interaction>
    >,
    open_machine: Res<OpenMachineUi>,
    mut set_recipe_events: EventWriter<SetRecipeEvent>,
) {
    for (interaction, recipe_btn, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if let Some(pos) = open_machine.target_pos {
                    set_recipe_events.send(SetRecipeEvent {
                        pos,
                        recipe_id: recipe_btn.recipe_id.clone(),
                    });
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.45));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.35));
            }
        }
    }
}

/// Handle close button click
fn handle_close_button_click(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CloseButton>)
    >,
    mut close_events: EventWriter<CloseMachineUiEvent>,
) {
    for (interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                close_events.send(CloseMachineUiEvent);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.8, 0.3, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.2, 0.2));
            }
        }
    }
}

/// Apply recipe selection to machine
fn apply_recipe_event(
    mut events: EventReader<SetRecipeEvent>,
    mut grid: ResMut<SimulationGrid>,
    mut close_events: EventWriter<CloseMachineUiEvent>,
) {
    for event in events.read() {
        if let Some(machine) = grid.machines.get_mut(&event.pos) {
            if let Machine::Assembler(assembler) = &mut machine.machine_type {
                assembler.active_recipe = Some(event.recipe_id.clone());
                assembler.crafting_progress = 0.0;
                info!("Set recipe '{}' for assembler at {:?}", event.recipe_id, event.pos);
            }
        }
        close_events.send(CloseMachineUiEvent);
    }
}

/// Update inventory display in real-time
fn update_inventory_display(
    open_machine: Res<OpenMachineUi>,
    grid: Res<SimulationGrid>,
    mut query: Query<(&InventoryDisplay, &mut Text)>,
    state: Res<State<MachineUiState>>,
) {
    if *state.get() != MachineUiState::Open { return; }

    let Some(pos) = open_machine.target_pos else { return };
    let Some(machine) = grid.machines.get(&pos) else { return };
    let Machine::Assembler(assembler) = &machine.machine_type else { return };

    for (display, mut text) in query.iter_mut() {
        let new_text = match display.slot_type {
            InventorySlotType::Input => {
                if assembler.input_inventory.is_empty() {
                    "Empty".to_string()
                } else {
                    assembler.input_inventory.iter()
                        .map(|s| format!("{}x{}", s.count, s.item_id))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            }
            InventorySlotType::Output => {
                if assembler.output_inventory.is_empty() {
                    "Empty".to_string()
                } else {
                    assembler.output_inventory.iter()
                        .map(|s| format!("{}x{}", s.count, s.item_id))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            }
        };
        **text = new_text;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::grid::{MachineInstance, Direction};
    use crate::gameplay::machines::assembler::Assembler;
    use bevy::MinimalPlugins;
    use bevy::state::app::StatesPlugin;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<MachineUiState>();
        app.init_resource::<OpenMachineUi>();
        app.init_resource::<SimulationGrid>();
        app.init_resource::<RecipeRegistry>();
        app.add_event::<OpenMachineUiEvent>();
        app.add_event::<CloseMachineUiEvent>();
        app.add_event::<SetRecipeEvent>();
        app.add_systems(Update, (
            handle_open_ui_event,
            handle_close_ui_event,
            apply_recipe_event,
        ));
        app
    }

    #[test]
    fn test_open_close_ui_state() {
        let mut app = setup_test_app();

        // Initial state should be Closed
        assert_eq!(*app.world().resource::<State<MachineUiState>>().get(), MachineUiState::Closed);

        // Send open event and run multiple updates for state transition
        app.world_mut().send_event(OpenMachineUiEvent { pos: IVec3::ZERO });
        for _ in 0..3 {
            app.update();
        }

        // State should be Open
        assert_eq!(*app.world().resource::<State<MachineUiState>>().get(), MachineUiState::Open);

        // Target pos should be set
        let open_machine = app.world().resource::<OpenMachineUi>();
        assert_eq!(open_machine.target_pos, Some(IVec3::ZERO));

        // Send close event and run multiple updates
        app.world_mut().send_event(CloseMachineUiEvent);
        for _ in 0..3 {
            app.update();
        }

        // State should be Closed
        assert_eq!(*app.world().resource::<State<MachineUiState>>().get(), MachineUiState::Closed);
    }

    #[test]
    fn test_set_recipe_event() {
        let mut app = setup_test_app();

        // Setup grid with assembler
        let pos = IVec3::new(1, 1, 1);
        {
            let mut grid = app.world_mut().resource_mut::<SimulationGrid>();
            grid.machines.insert(pos, MachineInstance {
                id: "assembler".to_string(),
                orientation: Direction::North,
                machine_type: Machine::Assembler(Assembler::default()),
                power_node: None,
            });
        }

        // Send set recipe event
        app.world_mut().send_event(SetRecipeEvent {
            pos,
            recipe_id: "ore_to_ingot".to_string(),
        });
        app.update();

        // Check recipe was set
        let grid = app.world().resource::<SimulationGrid>();
        let machine = grid.machines.get(&pos).unwrap();
        if let Machine::Assembler(asm) = &machine.machine_type {
            assert_eq!(asm.active_recipe, Some("ore_to_ingot".to_string()));
        } else {
            panic!("Expected Assembler");
        }
    }
}
