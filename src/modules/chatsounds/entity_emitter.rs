use crate::modules::SyncShared;
use chatsounds::SpatialSink;
use classicube_helpers::entities::{Entities, ENTITY_SELF_ID};
use classicube_sys::Vec3;
use std::sync::{Arc, Weak};

pub struct EntityEmitter {
  entity_id: u8,
  sink: Weak<SpatialSink>,
}

impl EntityEmitter {
  pub fn new(entity_id: u8, sink: &Arc<SpatialSink>) -> Self {
    Self {
      entity_id,
      sink: Arc::downgrade(&sink),
    }
  }

  /// returns true if still alive
  pub fn update(&mut self, entities: &mut SyncShared<Entities>) -> bool {
    let (emitter_pos, self_stuff) = {
      let entities = entities.lock();

      (
        if let Some(entity) = entities.get(self.entity_id) {
          Some(entity.get_position())
        } else {
          None
        },
        if let Some(entity) = entities.get(ENTITY_SELF_ID) {
          Some((entity.get_position(), entity.get_rot()[1]))
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
    left_ear_pos.X += HEAD_SIZE * left_cos; // x
    left_ear_pos.Z += HEAD_SIZE * left_sin; // z

    let mut right_ear_pos = self_pos;
    right_ear_pos.X += HEAD_SIZE * right_cos; // x
    right_ear_pos.Z += HEAD_SIZE * right_sin; // z

    (
      [emitter_pos.X, emitter_pos.Y, emitter_pos.Z],
      [left_ear_pos.X, left_ear_pos.Y, left_ear_pos.Z],
      [right_ear_pos.X, right_ear_pos.Y, right_ear_pos.Z],
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
