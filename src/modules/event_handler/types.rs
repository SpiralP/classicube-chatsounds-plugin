use classicube_sys::{InputButtons, InputDevice, MsgType};

// TODO should these be 1 enum? Event_Emit?
/// comes from main thread
#[derive(Debug, Clone)]
pub enum IncomingEvent {
    Tick,
    ChatReceived(String, MsgType),
    InputDown(InputButtons, bool, InputDeviceSend),
    InputUp(InputButtons, bool),
    InputPress(char),
}

/// goes to main thread
#[derive(Debug, Clone)]
pub enum OutgoingEvent {
    ChatAdd(String),
    ChatAddOf(String, MsgType),
    InputDown(InputButtons, bool, InputDeviceSend),
    InputUp(InputButtons, bool, InputDeviceSend),
    InputPress(char),
}

#[derive(Debug, Clone, Copy)]
pub struct InputDeviceSend(pub *mut InputDevice);
unsafe impl Send for InputDeviceSend {}
