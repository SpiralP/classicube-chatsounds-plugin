use super::entity_emitter::EntityEmitter;
use crate::{
  helpers::remove_color,
  modules::{
    chatsounds::random::rand_index,
    entities::ENTITY_SELF_ID,
    event_handler::{IncomingEvent, IncomingEventListener},
    EntitiesModule, FuturesModule, TabListModule,
  },
  printer::print,
};
use chatsounds::Chatsounds;
use classicube_sys::{MsgType, MsgType_MSG_TYPE_NORMAL};
use parking_lot::Mutex;
use std::{cell::RefCell, rc::Rc, sync::Arc};

pub struct ChatsoundsEventListener {
  chatsounds: Arc<Mutex<Chatsounds>>,
  entity_emitters: Vec<EntityEmitter>,
  chat_last: Option<String>,
  tab_list_module: Rc<RefCell<TabListModule>>,
  entities_module: Rc<RefCell<EntitiesModule>>,
}

impl ChatsoundsEventListener {
  pub fn new(
    tab_list_module: Rc<RefCell<TabListModule>>,
    entities_module: Rc<RefCell<EntitiesModule>>,
    chatsounds: Arc<Mutex<Chatsounds>>,
  ) -> Self {
    Self {
      chatsounds,
      entity_emitters: Vec::new(),
      chat_last: None,
      tab_list_module,
      entities_module,
    }
  }

  fn handle_chat_received(&mut self, mut full_msg: String, msg_type: MsgType) {
    if msg_type != MsgType_MSG_TYPE_NORMAL {
      return;
    }

    if !full_msg.starts_with("> &f") {
      self.chat_last = Some(full_msg.clone());
    } else if let Some(chat_last) = &self.chat_last {
      // we're a continue message
      full_msg = full_msg.split_off(4); // skip "> &f"

      // most likely there's a space
      // the server trims the first line :(
      // TODO try both messages? with and without the space?
      full_msg = format!("{} {}", chat_last, full_msg);
      self.chat_last = Some(full_msg.clone());
    }

    // &]SpiralP: &faaa
    // let full_msg = full_msg.into();

    // nickname_resolver_handle_message(full_msg.to_string());

    // find colon from the left
    if let Some(pos) = full_msg.find(": ") {
      // &]SpiralP
      let left = &full_msg[..pos]; // left without colon
                                   // &faaa
      let right = &full_msg[(pos + 2)..]; // right without colon

      if right.find(':').is_some() {
        // no colons in any chatsound, and we could have parsed nick wrong
        return;
      }

      // TODO title is [ ] before nick, team is < > before nick, also there are rank symbols?
      // &f┬ &f♂&6 Goodly: &fhi

      let full_nick = left.to_string();
      let colorless_text: String = remove_color(right.to_string()).trim().to_string();

      // lookup entity id from nick_name by using TabList
      let found_entity_id = self
        .tab_list_module
        .borrow()
        .find_entity_id_by_name(full_nick);

      if let Some(entity_id) = found_entity_id {
        // print(format!("FOUND {} {}", entity_id, full_nick));

        FuturesModule::block_future(async {
          self.play_chatsound(entity_id, colorless_text).await;
        });

        // } else { print(format!("not found {}", full_nick)); }
      }
    }
  }

  pub async fn play_chatsound(&mut self, entity_id: usize, sentence: String) {
    // TODO need the main thread tick to update positions

    // TODO can't use ENTITIES here
    let (emitter_pos, self_stuff) = {
      let entities = self.entities_module.borrow();

      (
        if let Some(entity) = entities.get(entity_id) {
          Some(entity.get_pos())
        } else {
          None
        },
        if let Some(entity) = entities.get(ENTITY_SELF_ID) {
          Some((entity.get_pos(), entity.get_rot()[1]))
        } else {
          print(format!(
            "couldn't get entity.get_pos/rot() {}",
            ENTITY_SELF_ID
          ));
          None
        },
      )
    };

    if sentence.to_lowercase() == "sh" {
      self.chatsounds.lock().stop_all();
      return;
    }

    let mut chatsounds = self.chatsounds.lock();
    if let Some(sounds) = chatsounds.get(sentence) {
      if let Some(sound) = rand_index(sounds, entity_id).cloned() {
        if entity_id == ENTITY_SELF_ID {
          // if self entity, play 2d sound
          chatsounds.play(&sound).await;
        } else if let Some(emitter_pos) = emitter_pos {
          if let Some((self_pos, self_rot)) = self_stuff {
            let (emitter_pos, left_ear_pos, right_ear_pos) =
              EntityEmitter::coords_to_sink_positions(emitter_pos, self_pos, self_rot);

            let sink = chatsounds
              .play_spatial(&sound, emitter_pos, left_ear_pos, right_ear_pos)
              .await;

            self.entity_emitters.push(EntityEmitter::new(
              entity_id,
              &sink,
              self.entities_module.clone(),
            ));
          }
        }
      }
    }
  }
}

impl IncomingEventListener for ChatsoundsEventListener {
  fn handle_incoming_event(&mut self, event: &IncomingEvent) {
    match event.clone() {
      IncomingEvent::ChatReceived(message, msg_type) => {
        self.handle_chat_received(message, msg_type);
      }

      IncomingEvent::Tick => {
        let mut to_remove = Vec::with_capacity(self.entity_emitters.len());
        for (i, emitter) in self.entity_emitters.iter_mut().enumerate() {
          if !emitter.update() {
            to_remove.push(i);
          }
        }

        // TODO can't you just use a for remove_id in ().rev()
        if !to_remove.is_empty() {
          for i in (0..self.entity_emitters.len()).rev() {
            if to_remove.contains(&i) {
              self.entity_emitters.remove(i);
            }
          }
        }
      }

      _ => {}
    }
  }
}
