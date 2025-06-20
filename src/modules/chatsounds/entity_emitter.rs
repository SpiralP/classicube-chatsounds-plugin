use std::sync::{Arc, Weak};

use chatsounds::rodio::SpatialSink;
use classicube_helpers::entities::{Entities, ENTITY_SELF_ID};
use classicube_sys::Vec3;

use crate::modules::SyncShared;

pub struct EntityEmitter {
    entity_id: Option<u8>,
    sink: Weak<SpatialSink>,
}

impl EntityEmitter {
    pub fn new(entity_id: Option<u8>, sink: &Arc<SpatialSink>) -> Self {
        Self {
            entity_id,
            sink: Arc::downgrade(sink),
        }
    }

    /// returns true if still alive
    pub fn update(&mut self, entities: &mut SyncShared<Entities>) -> bool {
        let entity_id = if let Some(entity_id) = self.entity_id {
            entity_id
        } else {
            return true;
        };

        let (emitter_pos, self_stuff) = {
            let entities = entities.borrow_mut();

            (
                if let Some(entity) = entities.get(entity_id) {
                    entity.upgrade().map(|entity| entity.get_position())
                } else {
                    None
                },
                if let Some(entity) = entities.get(ENTITY_SELF_ID) {
                    entity
                        .upgrade()
                        .map(|entity| (entity.get_position(), entity.get_rot()[1]))
                } else {
                    None
                },
            )
        };

        if let Some(emitter_pos) = emitter_pos {
            if let Some((self_pos, self_rot)) = self_stuff {
                let (emitter_pos, left_ear_pos, right_ear_pos) =
                    EntityEmitter::coords_to_sink_positions(emitter_pos, self_pos, self_rot);

                return self.update_sink(emitter_pos, left_ear_pos, right_ear_pos);
            }
        }

        true
    }

    pub fn coords_to_sink_positions(
        emitter_pos: Vec3,
        self_pos: Vec3,
        self_rot_yaw: f32,
    ) -> ([f32; 3], [f32; 3], [f32; 3]) {
        use std::f32::consts::PI;

        let (left_sin, left_cos) = {
            let ratio = self_rot_yaw / 360.0;
            let rot = ratio * (2.0 * PI) - PI;
            rot.sin_cos()
        };

        let (right_sin, right_cos) = {
            let ratio = self_rot_yaw / 360.0;
            let rot = ratio * (2.0 * PI);
            rot.sin_cos()
        };

        const HEAD_SIZE: f32 = 0.2;

        // z is negative going forward

        // print(format!(
        //   "{:?} {:?}",
        //   &[left_cos, left_sin],
        //   &[right_cos, right_sin]
        // ));

        let mut left_ear_pos = self_pos;
        left_ear_pos.x += HEAD_SIZE * left_cos; // x
        left_ear_pos.z += HEAD_SIZE * left_sin; // z

        let mut right_ear_pos = self_pos;
        right_ear_pos.x += HEAD_SIZE * right_cos; // x
        right_ear_pos.z += HEAD_SIZE * right_sin; // z

        (
            [emitter_pos.x, emitter_pos.y, emitter_pos.z],
            [left_ear_pos.x, left_ear_pos.y, left_ear_pos.z],
            [right_ear_pos.x, right_ear_pos.y, right_ear_pos.z],
        )
    }

    pub fn update_sink(
        &self,
        emitter_pos: [f32; 3],
        left_ear_pos: [f32; 3],
        right_ear_pos: [f32; 3],
    ) -> bool {
        if let Some(sink) = self.sink.upgrade() {
            // const DIST_FIX: f32 = 0.3;

            // print(format!("{:?}", emitter_pos));
            // print(format!("{:?} {:?}", left_ear_pos, right_ear_pos));

            // TODO LOL CHANGING LEFT TO RIGHT RIGHT TO LEFT
            sink.set_left_ear_position(mul_3(right_ear_pos, 0.2));

            sink.set_right_ear_position(mul_3(left_ear_pos, 0.2));

            sink.set_emitter_position(mul_3(emitter_pos, 0.2));

            true
        } else {
            false
        }
    }
}

fn mul_3(a: [f32; 3], n: f32) -> [f32; 3] {
    [a[0] * n, a[1] * n, a[2] * n]
}
