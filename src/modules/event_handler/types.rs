use classicube_sys::{Key, MsgType};

// TODO should these be 1 enum? Event_Emit?
/// comes from main thread
#[derive(Debug, Clone)]
pub enum IncomingEvent {
  Tick,
  ChatReceived(String, MsgType),
  InputDown(Key, bool),
  InputUp(Key),
  InputPress(char),
}

/// goes to main thread
#[derive(Debug, Clone)]
pub enum OutgoingEvent {
  ChatAdd(String),
  ChatAddOf(String, MsgType),
  InputDown(Key, bool),
  InputUp(Key),
  InputPress(char),
}
