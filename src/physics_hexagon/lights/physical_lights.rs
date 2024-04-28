use std::f32::consts::PI;
use bevy::math::Vec2;
use bevy::pbr::{PointLightBundle, SpotLightBundle};
use bevy::prelude::{BuildChildren, Children, Color, Commands, Component, Entity, error, PointLight, Quat, Query, SpatialBundle, SpotLight, Transform, With};
use bevy::render::view::RenderLayers;
use bevy::utils::default;
use crate::hexagon::HexagonDefinition;
use crate::physics_hexagon::lights::led_tube::{LEDS_COUNT, LedTube, LedTubeLed, TUBE_LENGTH, TubeIndex};
use crate::physics_hexagon::PhysicsHexagon;
use crate::propagating_render_layers::PropagatingRenderLayers;

/// Root component for all the lights of a hexagon
#[derive(Component)]
pub struct HexagonLights;

/// Represents the physical lights of each LED tube inside the hexagon
#[derive(Component)]
pub struct PhysicalLedTube {
    index: TubeIndex,
}

#[derive(Component)]
pub struct PhysicalLedTubeLed {
    /// Index among the tube, from left to right in screen space
    index: isize,
    /// The LedTubeLed this light is driven by
    led_tube_led: Entity,
}

pub fn spawn_physical_led_tube(
    tube_index: TubeIndex,
    for_hexagon: HexagonDefinition,
    commands: &mut Commands,
    physical_hexagon_query: &Query<(Entity, &Children, &PhysicsHexagon)>,
    hexagon_lights_query: &Query<Entity, With<PhysicsHexagon>>,
    led_tube_query: &Query<(&Children, &LedTube), With<LedTube>>,
    led_tube_led_query: &Query<(Entity, &LedTubeLed)>,
) {
    let Some((physics_hexagon_entity, physics_hexagon_children, physics_hexagon)) = physical_hexagon_query
        .iter()
        .find(|(_, _, ph)| {
            ph.hexagon_definition == for_hexagon
        }) else {
        error!("Hexagon for definition {:?} doesn't exist!", for_hexagon);
        return;
    };

    let Some((led_tube_children, led_tube)) = led_tube_query
        .iter()
        .find(|(_, led_tube)| {
            led_tube.get_tube_index() == tube_index
        }) else {
        error!("LedTube for index {:?} doesn't exist!", tube_index);
        return;
    };

    /// Position relative to hexagon center
    let position = tube_index.get_position() - for_hexagon.center();
    println!("Position for {:?}<->{:?}: {:?}", for_hexagon, tube_index, position);
    // TODO: Some tube_index/hexagon_definition combinations (shared edges) require flipping
    let rotation = tube_index.get_rotation() + match (for_hexagon, tube_index) {
        (HexagonDefinition::A1, TubeIndex::Three) |
        (HexagonDefinition::A1, TubeIndex::Five) |
        (HexagonDefinition::A1, TubeIndex::Six) => PI,
        (HexagonDefinition::A3, TubeIndex::Four) |
        (HexagonDefinition::A3, TubeIndex::Seven) => PI,
        (HexagonDefinition::B1, TubeIndex::Sixteen) |
        (HexagonDefinition::B1, TubeIndex::Eighteen) |
        (HexagonDefinition::B1, TubeIndex::Nineteen) => PI,
        (HexagonDefinition::B3, TubeIndex::Seventeen) |
        (HexagonDefinition::B3, TubeIndex::Twenty) => PI,
        _ => 0.
    };
    // Scoot the lights towards the center of the hexagon by this amount
    let scoot_amount = 30.;

    // Find or create root entity for lights
    let root_entity = physics_hexagon_children
        .iter()
        .map(|child| { (*child).clone() })
        .find(|child| {
            hexagon_lights_query.contains(*child)
        })
        .unwrap_or_else(|| {
            let root_entity = commands.spawn((
                HexagonLights,
                SpatialBundle::default(),
            )).id();
            commands.entity(physics_hexagon_entity).push_children(&[root_entity]);
            root_entity
        });

    let physical_led_tube_entity = commands.spawn(
        SpatialBundle {
            transform: Transform::from_xyz(position.x, position.y, 300.).with_rotation(Quat::from_rotation_z(rotation)),
            ..default()
        }
    ).id();
    commands.entity(root_entity).push_children(&[physical_led_tube_entity]);

    let mut physical_led_tube_leds = vec![];

    let step = TUBE_LENGTH / LEDS_COUNT as f32;
    for child in led_tube_children {
        let (led_tube_led_entity, ltl) = led_tube_led_query.get(*child).unwrap();
        let offset = ((0.5 + step * ltl.get_index() as f32) - TUBE_LENGTH/2.)*0.9;
        physical_led_tube_leds.push(
            commands.spawn((
                PhysicalLedTubeLed {
                    index: ltl.get_index(),
                    led_tube_led: led_tube_led_entity,
                },
                SpotLightBundle {
                    spot_light: SpotLight{
                        intensity: 500_000_000.0 / LEDS_COUNT as f32,
                        range: 3000.0,
                        radius: 15.,
                        color: Color::ORANGE_RED,
                        shadows_enabled: false,
                        outer_angle: PI/4.,
                        inner_angle: PI/6.,
                        ..default()
                    },
                    transform: Transform::from_xyz(offset, -scoot_amount, 0.).with_rotation(Quat::from_rotation_x(-PI/4.)),
                    ..default()
                }
            )).id()
        );
    }
    commands.entity(physical_led_tube_entity).push_children(physical_led_tube_leds.as_slice());
}