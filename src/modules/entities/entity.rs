use classicube_sys::Entities;

/// 255 is self entity
pub const ENTITY_SELF_ID: usize = 255;

#[derive(Debug, Clone)]
pub struct Entity {
  pub id: usize,
}

impl Entity {
  fn get_entity(&self) -> &classicube_sys::Entity {
    unsafe { &*Entities.List[self.id] }
  }

  pub fn get_pos(&self) -> [f32; 3] {
    let entity = self.get_entity();
    [entity.Position.X, entity.Position.Y, entity.Position.Z]
  }

  /// 0-360
  pub fn get_rot(&self) -> [f32; 3] {
    let entity = self.get_entity();
    [entity.RotX, entity.RotY, entity.RotZ]
  }

  // pub fn get_real_name(&self) -> String {
  //   let entity = self.get_entity();
  //   let c_str = unsafe { CStr::from_ptr(&entity.DisplayNameRaw as *const i8) };
  //   c_str.to_string_lossy().to_string()
  // }
}
