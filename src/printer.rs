use classicube::{
  Chat_AddOf, MsgType, MsgType_MSG_TYPE_BOTTOMRIGHT_1, MsgType_MSG_TYPE_BOTTOMRIGHT_2,
  MsgType_MSG_TYPE_BOTTOMRIGHT_3, MsgType_MSG_TYPE_NORMAL,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::{
  collections::VecDeque,
  convert::TryInto,
  time::{Duration, Instant},
};

lazy_static! {
  pub static ref PRINTER: Mutex<Printer> = Mutex::new(Printer::new());
}

pub struct Printer {
  sender: Sender<String>,
  receiver: Receiver<String>,
  last_messages: VecDeque<(String, Instant)>,
  remove_delay: Duration,
}
impl Printer {
  pub fn new() -> Self {
    let (sender, receiver) = unbounded();
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

  pub fn chat_add_of<S: Into<Vec<u8>>>(s: S, msg_type: MsgType) {
    let s = s.into();

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
      Chat_AddOf(&cc_str, msg_type.try_into().unwrap());
    }
  }

  pub fn chat_add<S: Into<Vec<u8>>>(s: S) {
    Printer::chat_add_of(s, MsgType_MSG_TYPE_NORMAL)
  }

  pub fn flush(&mut self) {
    let now = Instant::now();

    for s in self.receiver.try_iter() {
      self.last_messages.push_front((s, now));

      if let Some((s, _)) = self.last_messages.get(0) {
        Printer::chat_add_of(s.clone(), MsgType_MSG_TYPE_BOTTOMRIGHT_1);
      }

      if let Some((s, _)) = self.last_messages.get(1) {
        Printer::chat_add_of(s.clone(), MsgType_MSG_TYPE_BOTTOMRIGHT_2);
      }

      if let Some((s, _)) = self.last_messages.get(2) {
        Printer::chat_add_of(s.clone(), MsgType_MSG_TYPE_BOTTOMRIGHT_3);
      }

      if self.last_messages.len() == 4 {
        self.last_messages.pop_back();
      }
    }

    if let Some((_, time)) = self.last_messages.get(0) {
      if (now - *time) > self.remove_delay {
        Printer::chat_add_of(String::new(), MsgType_MSG_TYPE_BOTTOMRIGHT_1);
      }
    }

    if let Some((_, time)) = self.last_messages.get(1) {
      if (now - *time) > self.remove_delay {
        Printer::chat_add_of(String::new(), MsgType_MSG_TYPE_BOTTOMRIGHT_2);
      }
    }

    if let Some((_, time)) = self.last_messages.get(2) {
      if (now - *time) > self.remove_delay {
        Printer::chat_add_of(String::new(), MsgType_MSG_TYPE_BOTTOMRIGHT_3);
      }
    }
  }
}

pub fn print<T: Into<String>>(s: T) {
  PRINTER.lock().print(s)
}
