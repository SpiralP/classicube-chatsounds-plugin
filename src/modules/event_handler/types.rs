use classicube_sys::{Key_, MsgType};

// TODO should these be 1 enum? Event_Emit?
/// comes from main thread
#[derive(Debug, Clone)]
pub enum IncomingEvent {
  Tick,
  ChatReceived(String, MsgType),
  InputDown(Key_, bool),
  InputUp(Key_),
  InputPress(char),
}

/// goes to main thread
#[derive(Debug, Clone)]
pub enum OutgoingEvent {
  ChatAdd(String),
  ChatAddOf(String, MsgType),
  InputDown(Key_, bool),
  InputUp(Key_),
  InputPress(char),
}
