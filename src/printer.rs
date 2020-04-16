use crate::modules::event_handler::{chat_add, chat_add_of, IncomingEvent, IncomingEventListener};
use classicube_sys::MsgType_MSG_TYPE_CLIENTSTATUS_2;
use lazy_static::lazy_static;
use log::info;
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
    chat_add(s);
  }

  pub fn status<T: Into<String>>(&mut self, s: T) {
    let now = Instant::now();
    chat_add_of(s, MsgType_MSG_TYPE_CLIENTSTATUS_2);
    self.status_decay = Some(now + STATUS_DURATION);
  }

  pub fn status_forever<T: Into<String>>(&mut self, s: T) {
    chat_add_of(s, MsgType_MSG_TYPE_CLIENTSTATUS_2);
    self.status_decay = None;
  }
}

pub struct PrinterEventListener {}

impl IncomingEventListener for PrinterEventListener {
  fn handle_incoming_event(&mut self, event: &IncomingEvent) {
    if let IncomingEvent::Tick = event {
      let mut printer = PRINTER.lock();

      let now = Instant::now();

      if let Some(status_decay) = printer.status_decay {
        if now >= status_decay {
          chat_add_of("", MsgType_MSG_TYPE_CLIENTSTATUS_2);
          printer.status_decay = None;
        }
      }
    }
  }
}

pub fn print<T: Into<String>>(s: T) {
  let s = s.into();
  info!("{}", s);
  Printer::print(s)
}

pub fn status<T: Into<String>>(s: T) {
  let s = s.into();
  PRINTER.lock().status(s);
}

pub fn status_forever<T: Into<String>>(s: T) {
  let s = s.into();
  PRINTER.lock().status_forever(s);
}
