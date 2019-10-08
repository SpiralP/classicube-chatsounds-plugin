use chatsounds::Chatsounds;
use lazy_static::lazy_static;
use parking_lot::Mutex;

lazy_static! {
  pub static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
}
