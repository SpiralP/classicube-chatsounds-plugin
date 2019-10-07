use classicube::{
  Chat_AddOf, MsgType, MsgType_MSG_TYPE_BOTTOMRIGHT_1, MsgType_MSG_TYPE_BOTTOMRIGHT_2,
  MsgType_MSG_TYPE_BOTTOMRIGHT_3,
};
use std::{
  collections::VecDeque,
  sync::mpsc::{channel, Receiver, Sender},
  time::{Duration, Instant},
};

pub struct Printer {
  sender: Sender<String>,
  receiver: Receiver<String>,
  last_messages: VecDeque<(String, Instant)>,
  remove_delay: Duration,
}
impl Printer {
  pub fn new() -> Self {
    let (sender, receiver) = channel();
    Self {
      sender,
      receiver,
      last_messages: VecDeque::with_capacity(4),
      remove_delay: Duration::from_secs(10),
    }
  }

  pub fn print<T: Into<String>>(&self, s: T) {
    self.sender.send(s.into()).unwrap();
  }

  fn raw_print(s: String, msg_type: MsgType) {
    let length = s.len() as u16;
    let capacity = s.len() as u16;

    let c_str = std::ffi::CString::new(s).unwrap();

    let buffer = c_str.as_ptr() as *mut i8;

    let cc_str = classicube::String {
      buffer,
      length,
      capacity,
    };

    unsafe {
      Chat_AddOf(&cc_str, msg_type);
    }
  }

  pub fn flush(&mut self) {
    let now = Instant::now();

    for s in self.receiver.try_iter() {
      self.last_messages.push_front((s, now));

      if let Some((s, _)) = self.last_messages.get(0) {
        Printer::raw_print(s.clone(), MsgType_MSG_TYPE_BOTTOMRIGHT_1);
      }

      if let Some((s, _)) = self.last_messages.get(1) {
        Printer::raw_print(s.clone(), MsgType_MSG_TYPE_BOTTOMRIGHT_2);
      }

      if let Some((s, _)) = self.last_messages.get(2) {
        Printer::raw_print(s.clone(), MsgType_MSG_TYPE_BOTTOMRIGHT_3);
      }

      if self.last_messages.len() == 4 {
        self.last_messages.pop_back();
      }
    }

    if let Some((_, time)) = self.last_messages.get(0) {
      if (now - *time) > self.remove_delay {
        Printer::raw_print(String::new(), MsgType_MSG_TYPE_BOTTOMRIGHT_1);
      }
    }

    if let Some((_, time)) = self.last_messages.get(1) {
      if (now - *time) > self.remove_delay {
        Printer::raw_print(String::new(), MsgType_MSG_TYPE_BOTTOMRIGHT_2);
      }
    }

    if let Some((_, time)) = self.last_messages.get(2) {
      if (now - *time) > self.remove_delay {
        Printer::raw_print(String::new(), MsgType_MSG_TYPE_BOTTOMRIGHT_3);
      }
    }
  }
}
