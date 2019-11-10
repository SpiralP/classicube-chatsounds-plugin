use classicube_sys::{StringsBuffer_UNSAFE_Get, TabList};
use std::os::raw::c_int;

#[derive(Debug, Clone)]
pub struct TabListEntry {
  pub id: u8,
  pub real_name: String,
  pub nick_name: String,
  pub group: String,
}

impl TabListEntry {
  pub fn from_id(id: u8) -> Self {
    Self {
      id,
      real_name: tablist_get_real_name(id),
      nick_name: tablist_get_nick_name(id),
      group: tablist_get_group(id),
    }
  }
}

fn tablist_get_real_name(id: u8) -> String {
  // or "Player"
  unsafe {
    StringsBuffer_UNSAFE_Get(
      &mut TabList._buffer,
      c_int::from(TabList.NameOffsets[id as usize] - 3),
    )
  }
  .to_string()
}

fn tablist_get_nick_name(id: u8) -> String {
  // or "Text"
  unsafe {
    StringsBuffer_UNSAFE_Get(
      &mut TabList._buffer,
      c_int::from(TabList.NameOffsets[id as usize] - 2),
    )
  }
  .to_string()
}

fn tablist_get_group(id: u8) -> String {
  unsafe {
    StringsBuffer_UNSAFE_Get(
      &mut TabList._buffer,
      c_int::from(TabList.NameOffsets[id as usize] - 1),
    )
  }
  .to_string()
}
