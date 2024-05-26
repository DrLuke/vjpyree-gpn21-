use std::cmp::PartialEq;
use bevy::asset::Handle;
use bevy::ecs::bundle::DynamicBundle;
use bevy::hierarchy::Children;
use bevy::math::{Quat, Vec3};
use bevy::prelude::{EventReader, GlobalTransform, Local, Mesh, Query, Real, Res, Time, Transform, With};
use bevy::utils::tracing::event;
use bevy_rapier3d::dynamics::RigidBody;
use rand::{Rng, thread_rng};
use crate::beat::BeatEvent;
use crate::physics_hexagon::effectors::{EyesMode, PhysHexSettings};
use crate::physics_hexagon::HexagonPhysicsElement;


pub fn eyes_mode(
    mut physics_element_query: Query<(&Children, &GlobalTransform), (With<HexagonPhysicsElement>, With<RigidBody>)>,
    mut model_query: Query<&mut Transform, With<Handle<Mesh>>>,
    settings: Res<PhysHexSettings>,
    mut beat_reader: EventReader<BeatEvent>,
    mut beat_rot: Local<Quat>,
    time: Res<Time<Real>>,
) {
    // This beat's rotation
    for _ in beat_reader.read() {
        let mut rng = thread_rng();
        *beat_rot = Quat::from_axis_angle(
            Vec3::new(rng.gen::<f32>() * 2. - 1., rng.gen::<f32>() * 2. - 1., rng.gen::<f32>() * 2. - 1.).normalize(),
            rng.gen::<f32>() * 0.02 + 0.02
        );
    }

    for (children, p_t) in physics_element_query.iter() {
        for child in children {
            let mut t = model_query.get_mut(*child).unwrap();

            match settings.eyes_mode {
                EyesMode::None => { t.rotation = Quat::default(); }
                EyesMode::Crazy => { t.rotation = t.rotation * *beat_rot; }
                EyesMode::Stare => {
                    let current = t.rotation;
                    let target = p_t.to_scale_rotation_translation().1.inverse();
                    let diff_quat = target * current.inverse();
                    let diff_angle = target.angle_between(current);

                    let u_diff = (diff_angle) * (time.delta_seconds()/(0.5+time.delta_seconds()));

                    t.rotation = t.rotation.lerp(diff_quat * current, u_diff/diff_angle);
                }
            }
        }
    }
}