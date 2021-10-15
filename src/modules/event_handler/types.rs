use classicube_sys::{InputButtons, MsgType};

// TODO should these be 1 enum? Event_Emit?
/// comes from main thread
#[derive(Debug, Clone)]
pub enum IncomingEvent {
    Tick,
    ChatReceived(String, MsgType),
    InputDown(InputButtons, bool),
    InputUp(InputButtons),
    InputPress(char),
}

/// goes to main thread
#[derive(Debug, Clone)]
pub enum OutgoingEvent {
    ChatAdd(String),
    ChatAddOf(String, MsgType),
    InputDown(InputButtons, bool),
    InputUp(InputButtons),
    InputPress(char),
}
