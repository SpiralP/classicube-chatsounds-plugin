mod chat;

pub use self::chat::*;
use super::{OutgoingEvent, OUTGOING_RECEIVER, OUTGOING_SENDER};
use crate::events::{block_future, SIMULATING};
use classicube_sys::{
  Chat_Add, Chat_AddOf, Event_RaiseInput, Event_RaiseInt, InputEvents, OwnedString,
};
use futures::{
  channel::oneshot::{channel as oneshot_channel, Sender as OneshotSender},
  lock::Mutex,
};
use lazy_static::lazy_static;
use std::{
  any::Any,
  cell::RefCell,
  os::raw::c_int,
  sync::mpsc::{channel, Receiver, Sender},
};

pub fn new_outgoing_event(event: OutgoingEvent) {
  let mut outgoing_sender = OUTGOING_SENDER.lock();
  if let Some(sender) = outgoing_sender.as_mut() {
    sender.send(event).unwrap();
  }
}

type AAAA = (
  Box<dyn FnOnce() -> Box<dyn Any + Send + 'static> + Send + 'static>,
  OneshotSender<Box<dyn Any + Send + 'static>>,
);

lazy_static! {
  static ref MAIN_THREAD_TASKS_SENDER: Mutex<Option<Sender<AAAA>>> = Mutex::new(None);
}

thread_local! {
  static MAIN_THREAD_TASKS_RECEIVER: RefCell<Option<Receiver<AAAA>>> = RefCell::new(None);
}

pub fn load() {
  let (sender, receiver) = channel();

  MAIN_THREAD_TASKS_RECEIVER.with(move |ref_cell| {
    let mut ag = ref_cell.borrow_mut();
    *ag = Some(receiver);
  });

  block_future(async move {
    let mut ag = MAIN_THREAD_TASKS_SENDER.lock().await;
    *ag = Some(sender);
  });
}

pub fn handle_outgoing_events() {
  SIMULATING.with(|simulating| {
    simulating.set(true);
  });

  OUTGOING_RECEIVER.with(|ref_cell| {
    let mut maybe_receiver = ref_cell.borrow_mut();

    if let Some(receiver) = maybe_receiver.as_mut() {
      for event in receiver.try_iter() {
        handle_outgoing_event(event);
      }
    }
  });

  MAIN_THREAD_TASKS_RECEIVER.with(|ref_cell| {
    let mut ag = ref_cell.borrow_mut();
    let ag = ag.as_mut().unwrap();

    for (f, sender) in ag.try_iter() {
      let boxed_value = f();
      sender.send(boxed_value).unwrap();
    }
  });

  SIMULATING.with(|simulating| {
    simulating.set(false);
  });
}

pub async fn wait_for_main_thread<T: Any + Send + 'static, F: FnOnce() -> T + Send + 'static>(
  f: F,
) -> Box<T> {
  let receiver = {
    let main_thread_tasks_sender = MAIN_THREAD_TASKS_SENDER.lock().await;
    let main_thread_tasks_sender = main_thread_tasks_sender.as_ref().unwrap();

    let (sender, receiver) = oneshot_channel();

    main_thread_tasks_sender
      .send((
        Box::new(|| {
          let v = f();

          Box::new(v)
        }),
        sender,
      ))
      .unwrap();

    receiver
  };

  receiver.await.unwrap().downcast().unwrap()
}

fn handle_outgoing_event(event: OutgoingEvent) {
  match event {
    OutgoingEvent::ChatAdd(text) => {
      let owned_string = OwnedString::new(text);

      unsafe {
        Chat_Add(owned_string.as_cc_string());
      }
    }

    OutgoingEvent::ChatAddOf(msg, msg_type) => {
      let owned_string = OwnedString::new(msg);

      unsafe {
        Chat_AddOf(owned_string.as_cc_string(), msg_type as _);
      }
    }

    OutgoingEvent::InputPress(chr) => unsafe {
      Event_RaiseInt(&mut InputEvents.Press, c_int::from(chr as u8));
    },

    OutgoingEvent::InputDown(key, repeat) => unsafe {
      Event_RaiseInput(&mut InputEvents.Down, key as _, repeat);
    },

    OutgoingEvent::InputUp(key) => unsafe {
      Event_RaiseInt(&mut InputEvents.Up, key as _);
    },
  }
}
