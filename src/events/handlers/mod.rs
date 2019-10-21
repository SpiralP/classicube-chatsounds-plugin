mod incoming;
mod outgoing;

pub use self::{incoming::*, outgoing::*};
use classicube_sys::{Key_, MsgType};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::cell::RefCell;
use tokio::runtime::Runtime;

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

thread_local! {
  pub static TOKIO_RUNTIME: RefCell<Option<Runtime>> = RefCell::new(None);
  pub static OUTGOING_RECEIVER: RefCell<Option<Receiver<OutgoingEvent>>> = RefCell::new(None);
}

lazy_static! {
  pub static ref OUTGOING_SENDER: Mutex<Option<Sender<OutgoingEvent>>> = Mutex::new(None);
}

pub fn load() {
  let mut outgoing_sender = OUTGOING_SENDER.lock();

  OUTGOING_RECEIVER.with(|ref_cell| {
    let (sender, receiver) = unbounded();

    ref_cell.replace(Some(receiver));
    outgoing_sender.replace(sender);
  });

  TOKIO_RUNTIME.with(|ref_cell| {
    let rt = Runtime::new().unwrap();

    ref_cell.replace(Some(rt));
  });
}

pub fn unload() {
  TOKIO_RUNTIME.with(|ref_cell| {
    let mut maybe_rt = ref_cell.replace(None);
    if let Some(rt) = maybe_rt.take() {
      rt.shutdown_now();
    }
  });

  *OUTGOING_SENDER.lock() = None;

  OUTGOING_RECEIVER.with(|ref_cell| {
    ref_cell.replace(None);
  });
}
