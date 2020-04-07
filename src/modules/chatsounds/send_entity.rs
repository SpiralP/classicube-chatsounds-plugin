use classicube_helpers::entities::Entity;
use classicube_sys::Vec3;

/// so that we capture and send entity data through futures/threads
pub struct SendEntity {
  pub id: u8,
  pub pos: Vec3,
  pub rot: [f32; 3],
}

impl From<&Entity> for SendEntity {
  fn from(e: &Entity) -> Self {
    let id = e.get_id();
    let pos = e.get_position();
    let rot = e.get_rot();

    // display_name isn't the same as tab_list.real_name
    // let real_name = e.get_display_name();

    Self { id, pos, rot }
  }
}
