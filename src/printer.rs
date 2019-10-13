use classicube_sys::{
  Chat_AddOf, MsgType, MsgType_MSG_TYPE_CLIENTSTATUS_2, MsgType_MSG_TYPE_NORMAL, OwnedString,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::{
  os::raw::c_int,
  time::{Duration, Instant},
};

const DEFAULT_STATUS_DURATION: Duration = Duration::from_secs(10);

lazy_static! {
  pub static ref PRINTER: Mutex<Printer> = Mutex::new(Printer::new());
}

enum Message {
  Normal(String),
  Status(String),
  StatusForever(String),
}

pub struct Printer {
  sender: Sender<Message>,
  receiver: Receiver<Message>,
  status_decay: Option<Instant>,
}
impl Printer {
  pub fn new() -> Self {
    let (sender, receiver) = unbounded();

    Self {
      sender,
      receiver,
      status_decay: None,
    }
  }

  pub fn print<T: Into<String>>(&self, s: T) {
    self.sender.send(Message::Normal(s.into())).unwrap();
  }

  pub fn status<T: Into<String>>(&self, s: T) {
    self.sender.send(Message::Status(s.into())).unwrap();
  }

  pub fn status_forever<T: Into<String>>(&self, s: T) {
    self.sender.send(Message::StatusForever(s.into())).unwrap();
  }

  pub fn chat_add_of<S: Into<Vec<u8>>>(s: S, msg_type: MsgType) {
    let owned_string = OwnedString::new(s);

    unsafe {
      Chat_AddOf(owned_string.as_cc_string(), msg_type as c_int);
    }
  }

  pub fn chat_add<S: Into<Vec<u8>>>(s: S) {
    Printer::chat_add_of(s, MsgType_MSG_TYPE_NORMAL)
  }

  pub fn flush(&mut self) {
    let now = Instant::now();

    for message in self.receiver.try_iter() {
      match message {
        Message::Normal(s) => {
          Printer::chat_add(s);
        }

        Message::Status(s) => {
          Self::chat_add_of(s, MsgType_MSG_TYPE_CLIENTSTATUS_2);
          self.status_decay = Some(now + DEFAULT_STATUS_DURATION);
        }

        Message::StatusForever(s) => {
          Self::chat_add_of(s, MsgType_MSG_TYPE_CLIENTSTATUS_2);
          self.status_decay = None;
        }
      }
    }

    if let Some(status_decay) = self.status_decay {
      if now >= status_decay {
        Self::chat_add_of("", MsgType_MSG_TYPE_CLIENTSTATUS_2);
      }
    }
  }
}

pub fn print<T: Into<String>>(s: T) {
  // TODO check if main thread somehow and print directly
  PRINTER.lock().print(s)
}

pub fn status<T: Into<String>>(s: T) {
  PRINTER.lock().status(s);
}

pub fn status_forever<T: Into<String>>(s: T) {
  PRINTER.lock().status_forever(s);
}
