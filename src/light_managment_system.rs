use bevy_ecs::prelude::Query;
use nalgebra::{Matrix3, Matrix4, Vector3};

use crate::common_component::{Camera, MainCamera, PointLight, Transform};

pub fn light_assignment_prepass(
    camera: Query<(&Camera, &Transform, &MainCamera)>,
    lights: Query<(&PointLight, &Transform)>,
) {
    let bounds = lights.iter().fold(
        (Vector3::<f32>::zeros(), Vector3::<f32>::zeros()),
        |acc, (_pl, t)| {
            let pos = t.isometry.translation;
            (
                Vector3::new(acc.0.x.max(pos.x), acc.0.y.max(pos.y), acc.0.z.max(pos.z)),
                Vector3::new(acc.1.x.min(pos.x), acc.1.y.min(pos.y), acc.1.z.min(pos.z)),
            )
        },
    );

    let t = bounds.1;
    let s = bounds.0 - bounds.1;

    let transform = Matrix4::new_translation(&t).append_nonuniform_scaling(&s);

    let morton_lights = lights.iter().map(|(_pl, t)| {});
}
