use super::entity_emitter::EntityEmitter;
use crate::{
  helpers::remove_color,
  modules::{
    chatsounds::random::rand_index,
    event_handler::{IncomingEvent, IncomingEventListener},
    FutureShared, FuturesModule, SyncShared, TabListModule, ThreadShared,
  },
  printer::print,
};
use chatsounds::Chatsounds;
use classicube_helpers::{Entities, ENTITY_SELF_ID};
use classicube_sys::{MsgType, MsgType_MSG_TYPE_NORMAL};

pub struct ChatsoundsEventListener {
  chatsounds: FutureShared<Chatsounds>,
  entity_emitters: ThreadShared<Vec<EntityEmitter>>,
  chat_last: Option<String>,
  tab_list_module: SyncShared<TabListModule>,
  entities: SyncShared<Entities>,
}

impl ChatsoundsEventListener {
  pub fn new(
    tab_list_module: SyncShared<TabListModule>,
    entities: SyncShared<Entities>,
    chatsounds: FutureShared<Chatsounds>,
  ) -> Self {
    Self {
      chatsounds,
      entity_emitters: ThreadShared::new(Vec::new()),
      chat_last: None,
      tab_list_module,
      entities,
    }
  }

  // run this sync so that chat_last comes in order
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
        .lock()
        .find_entity_id_by_name(full_nick);

      if let Some(entity_id) = found_entity_id {
        // print(format!("FOUND {} {}", entity_id, full_nick));

        let entities = self.entities.lock();

        let (emitter_pos, self_stuff) = {
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

        let chatsounds = self.chatsounds.clone();
        let entity_emitters = self.entity_emitters.clone();

        // it doesn't matter if these are out of order so we just spawn
        FuturesModule::spawn_future(async move {
          play_chatsound(
            entity_id,
            colorless_text,
            emitter_pos,
            self_stuff,
            chatsounds,
            entity_emitters,
          )
          .await;
        });

        // } else { print(format!("not found {}", full_nick)); }
      }
    }
  }
}

pub async fn play_chatsound(
  entity_id: u8,
  sentence: String,
  emitter_pos: Option<[f32; 3]>,
  self_stuff: Option<([f32; 3], f32)>,
  mut chatsounds: FutureShared<Chatsounds>,
  mut entity_emitters: ThreadShared<Vec<EntityEmitter>>,
) {
  if sentence.to_lowercase() == "sh" {
    chatsounds.lock().await.stop_all();
    entity_emitters.lock().clear();
    return;
  }

  let mut chatsounds = chatsounds.lock().await;
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

          entity_emitters
            .lock()
            .push(EntityEmitter::new(entity_id, &sink));
        }
      }
    }
  }
}

impl IncomingEventListener for ChatsoundsEventListener {
  fn handle_incoming_event(&mut self, event: &IncomingEvent) {
    match event.clone() {
      IncomingEvent::ChatReceived(message, msg_type) => {
        self.handle_chat_received(message, msg_type)
      }

      IncomingEvent::Tick => {
        // update positions on emitters

        let mut entity_emitters = self.entity_emitters.lock();

        let mut to_remove = Vec::with_capacity(entity_emitters.len());
        for (i, emitter) in entity_emitters.iter_mut().enumerate() {
          if !emitter.update(&mut self.entities) {
            to_remove.push(i);
          }
        }

        // TODO can't you just use a for remove_id in ().rev()
        if !to_remove.is_empty() {
          for i in (0..entity_emitters.len()).rev() {
            if to_remove.contains(&i) {
              entity_emitters.remove(i);
            }
          }
        }
      }

      _ => {}
    }
  }
}
