use std::sync::{Arc, Weak};

use chatsounds::ChannelVolumeSink;
use classicube_helpers::entities::Entities;
use classicube_sys::Vec3;
use kira::{Frame, Panning};
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
        let my_pos = vec3_to_vector3(&position);
        let my_forward = vec3_to_vector3(&Vec3::get_dir_vector(self_rot_yaw, 0.0));

        let ent_pos = vec3_to_vector3(&emitter_pos);

        let relative_distance = relative_distance((my_pos - ent_pos).magnitude());
        let relative_volume = 1.0 - relative_distance;

        let up = Vector3::y();
        let left = Vector3::cross(&my_forward, &up);
        let left = left.normalize();

        let pan = (ent_pos - my_pos).normalize().dot(&left);

        let frame = Frame::from_mono(relative_volume).panned(Panning(pan));

        vec![
            //
            frame.left.clamp(0.0, 1.0),
            frame.right.clamp(0.0, 1.0),
        ]
    }
}

// https://docs.rs/kira/0.10.8/src/kira/track/sub/spatial_builder.rs.html#355
fn relative_distance(distance: f32) -> f32 {
    const MIN_DISTANCE: f32 = 3.0;
    const MAX_DISTANCE: f32 = 30.0;

    let distance = distance.clamp(MIN_DISTANCE, MAX_DISTANCE);
    (distance - MIN_DISTANCE) / (MAX_DISTANCE - MIN_DISTANCE)
}
