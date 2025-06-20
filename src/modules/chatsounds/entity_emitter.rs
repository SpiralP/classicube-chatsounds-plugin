use std::{
    f32::consts::{FRAC_PI_4, SQRT_2},
    sync::{Arc, Weak},
};

use chatsounds::ChannelVolumeSink;
use classicube_helpers::entities::Entities;
use classicube_sys::Vec3;
use ncollide3d::na::Vector3;

use crate::{
    helpers::{get_self_position_and_yaw, vec3_to_vector3},
    modules::SyncShared,
};

pub struct EntityEmitter {
    entity_id: u8,
    sink: Weak<ChannelVolumeSink>,
    static_pos: Option<Vec3>,
}

impl EntityEmitter {
    pub fn new(entity_id: u8, sink: &Arc<ChannelVolumeSink>, static_pos: Option<Vec3>) -> Self {
        Self {
            entity_id,
            sink: Arc::downgrade(sink),
            static_pos,
        }
    }

    /// returns None to remove the emitter
    pub fn update(&mut self, entities: &mut SyncShared<Entities>) -> Option<()> {
        let emitter_pos = self.static_pos.or_else(|| {
            let entity = entities.borrow_mut().get(self.entity_id)?;
            let entity = entity.upgrade()?;
            Some(entity.get_position())
        })?;

        let (self_pos, self_rot_yaw) = get_self_position_and_yaw()?;
        let channel_volumes =
            EntityEmitter::coords_to_sink_channel_volumes(emitter_pos, self_pos, self_rot_yaw);

        self.sink.upgrade()?.set_channel_volumes(channel_volumes);

        Some(())
    }

    pub fn coords_to_sink_channel_volumes(
        emitter_pos: Vec3,
        position: Vec3,
        self_rot_yaw: f32,
    ) -> Vec<f32> {
        const MAX_DISTANCE: f32 = 30.0;

        let my_pos = vec3_to_vector3(&position);
        let my_forward = vec3_to_vector3(&Vec3::get_dir_vector(self_rot_yaw, 0.0));

        let ent_pos = vec3_to_vector3(&emitter_pos);

        let diff = my_pos - ent_pos;
        let percent = diff.magnitude() / MAX_DISTANCE;
        let percent = (1.0 - percent).clamp(0.0, 1.0);

        let up = Vector3::y();

        let left = Vector3::cross(&my_forward, &up);
        let left = left.normalize();

        let pan = (ent_pos - my_pos).normalize().dot(&left);
        let pan = pan * 0.8; // -1 full left, 1 full right

        let angle = (pan + 1.0) * FRAC_PI_4;

        let gain_l = angle.cos();
        let gain_r = angle.sin();

        let output_l = gain_l * SQRT_2;
        let output_r = gain_r * SQRT_2;

        vec![
            percent * output_l, // left channel volume
            percent * output_r, // right channel volume
        ]
    }
}

#[test]
fn test_coords_to_sink_channel_volumes() {
    let emitter_pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let self_pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    for (self_rot_yaw, result) in [
        (0.0f32, [0.9666666, 0.9666666]),  // Facing -z
        (90.0, [1.3502421, 0.21385734]),   // Facing +x
        (180.0, [0.96666646, 0.96666676]), // Facing +z
        (270.0, [0.21385737, 1.3502421]),  // Facing -x
    ] {
        let channel_volumes = EntityEmitter::coords_to_sink_channel_volumes(
            emitter_pos,
            self_pos,
            self_rot_yaw.to_radians(),
        );
        assert_eq!(
            channel_volumes, result,
            "Failed for self_rot_yaw: {}",
            self_rot_yaw
        );
    }

    let emitter_pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let self_pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 10.0,
    };

    for (self_rot_yaw, result) in [
        (0.0f32, [0.66666657, 0.66666657]), // Facing -z
        (90.0, [0.93120146, 0.1474878]),    // Facing +x
        (180.0, [0.6666665, 0.6666667]),    // Facing +z
        (270.0, [0.14748783, 0.93120146]),  // Facing -x
    ] {
        let channel_volumes = EntityEmitter::coords_to_sink_channel_volumes(
            emitter_pos,
            self_pos,
            self_rot_yaw.to_radians(),
        );
        assert_eq!(
            channel_volumes, result,
            "Failed for self_rot_yaw: {}",
            self_rot_yaw
        );
    }

    let emitter_pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let self_pos = Vec3 {
        x: 0.0,
        y: 0.0,
        z: 30.0,
    };

    for (self_rot_yaw, result) in [
        (0.0f32, [0.0, 0.0]), // Facing -z
        (90.0, [0.0, 0.0]),   // Facing +x
        (180.0, [0.0, 0.0]),  // Facing +z
        (270.0, [0.0, 0.0]),  // Facing -x
    ] {
        let channel_volumes = EntityEmitter::coords_to_sink_channel_volumes(
            emitter_pos,
            self_pos,
            self_rot_yaw.to_radians(),
        );
        assert_eq!(
            channel_volumes, result,
            "Failed for self_rot_yaw: {}",
            self_rot_yaw
        );
    }
}
