use crate::modules::event_handler::{chat_add, chat_add_of};
use classicube_sys::{MsgType, MsgType_MSG_TYPE_CLIENTSTATUS_2};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::time::{Duration, Instant};

const STATUS_DURATION: Duration = Duration::from_secs(8);

lazy_static! {
  pub static ref PRINTER: Mutex<Printer> = Mutex::new(Printer::new());
}

pub struct Printer {
  status_decay: Option<Instant>,
}

impl Printer {
  pub fn new() -> Self {
    Self { status_decay: None }
  }

  pub fn print<T: Into<String>>(s: T) {
    Self::chat_add(s);
  }

  pub fn status<T: Into<String>>(&mut self, s: T) {
    let now = Instant::now();
    Self::chat_add_of(s, MsgType_MSG_TYPE_CLIENTSTATUS_2);
    self.status_decay = Some(now + STATUS_DURATION);
  }

  pub fn status_forever<T: Into<String>>(&mut self, s: T) {
    Self::chat_add_of(s, MsgType_MSG_TYPE_CLIENTSTATUS_2);
    self.status_decay = None;
  }

  pub fn chat_add<S: Into<String>>(s: S) {
    chat_add(s)
  }

  pub fn chat_add_of<S: Into<String>>(s: S, msg_type: MsgType) {
    chat_add_of(s, msg_type)
  }

  pub fn tick(&mut self) {
    // TODO
    // let now = Instant::now();

    // if let Some(status_decay) = self.status_decay {
    //   if now >= status_decay {
    //     Self::chat_add_of("", MsgType_MSG_TYPE_CLIENTSTATUS_2);
    //     self.status_decay = None;
    //   }
    // }

    todo!()
  }
}

pub fn print<T: Into<String>>(s: T) {
  Printer::print(s)
}

pub fn status<T: Into<String>>(s: T) {
  PRINTER.lock().status(s);
}

pub fn status_forever<T: Into<String>>(s: T) {
  PRINTER.lock().status_forever(s);
}
