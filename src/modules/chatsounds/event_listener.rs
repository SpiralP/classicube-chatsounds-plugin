use chatsounds::Chatsounds;
use classicube_helpers::{
    entities::{Entities, ENTITY_SELF_ID},
    tab_list::{remove_color, TabList},
};
use classicube_sys::{MsgType, MsgType_MSG_TYPE_NORMAL, Server, Vec3, WindowInfo};

use super::{entity_emitter::EntityEmitter, random, send_entity::SendEntity};
use crate::{
    helpers::{
        get_self_position_and_yaw, is_continuation_message, is_global_cs_message,
        is_global_csent_message, is_global_cspos_message,
    },
    modules::{
        chatsounds::random::{get_rng, GLOBAL_NAME},
        command::MUTE_LOSE_FOCUS_SETTING_NAME,
        event_handler::{IncomingEvent, IncomingEventListener},
        FutureShared, FuturesModule, OptionModule, SyncShared, ThreadShared,
    },
};

pub struct ChatsoundsEventListener {
    chat_last: Option<String>,
    chatsounds: FutureShared<Option<Chatsounds>>,
    entities: SyncShared<Entities>,
    entity_emitters: ThreadShared<Vec<EntityEmitter>>,
    last_volume: FutureShared<Option<f32>>,
    tab_list: SyncShared<TabList>,
}

impl ChatsoundsEventListener {
    pub fn new(
        chatsounds: FutureShared<Option<Chatsounds>>,
        entities: SyncShared<Entities>,
        tab_list: SyncShared<TabList>,
    ) -> Self {
        Self {
            chat_last: None,
            chatsounds,
            entities,
            entity_emitters: ThreadShared::default(),
            last_volume: FutureShared::default(),
            tab_list,
        }
    }

    fn find_player_from_message(&mut self, mut full_msg: String) -> Option<(u8, String, String)> {
        if unsafe { Server.IsSinglePlayer } != 0 {
            // in singleplayer there is no tab list, even self id infos are null

            return Some((ENTITY_SELF_ID, String::new(), full_msg));
        }

        if let Some(continuation) = is_continuation_message(&full_msg) {
            if let Some(chat_last) = &self.chat_last {
                // we're a continue message
                full_msg = continuation.to_string();

                // most likely there's a space
                // the server trims the first line :(
                full_msg = format!("{chat_last} {full_msg}");
                self.chat_last = Some(full_msg.clone());
            }
        } else {
            // normal message start
            self.chat_last = Some(full_msg.clone());
        }

        // &]SpiralP: &faaa
        // let full_msg = full_msg.into();

        // nickname_resolver_handle_message(full_msg.to_string());

        // find colon from the left
        let opt = full_msg
            .find(": ")
            .and_then(|pos| if pos > 4 { Some(pos) } else { None });
        if let Some(pos) = opt {
            // &]SpiralP
            let left = &full_msg[..pos]; // left without colon
                                         // &faaa
            let right = &full_msg[(pos + 2)..]; // right without colon

            // TODO title is [ ] before nick, team is < > before nick, also there are rank
            // symbols? &f┬ &f♂&6 Goodly: &fhi

            let full_nick = left.to_string();
            let said_text = right.to_string();

            // lookup entity id from nick_name by using TabList
            self.tab_list
                .borrow_mut()
                .find_entry_by_nick_name(&full_nick)
                .and_then(|entry| {
                    entry
                        .upgrade()
                        .map(|entry| (entry.get_id(), entry.get_real_name(), said_text))
                })
        } else {
            None
        }
    }

    // run this sync so that chat_last comes in order
    fn handle_chat_received(&mut self, full_msg: String, msg_type: MsgType) {
        if msg_type != MsgType_MSG_TYPE_NORMAL {
            return;
        }

        let focused = unsafe { WindowInfo.Focused } != 0;
        if !focused {
            return;
        }

        let Some((self_pos, self_rot_yaw)) = get_self_position_and_yaw() else {
            return;
        };

        let (id, real_name, said_text, static_pos) = if let Some(said_text) =
            is_global_cs_message(&full_msg)
        {
            (
                ENTITY_SELF_ID,
                GLOBAL_NAME.to_string(),
                said_text.to_string(),
                None,
            )
        } else if let Some((said_text, static_pos)) = is_global_cspos_message(&full_msg) {
            (
                ENTITY_SELF_ID,
                GLOBAL_NAME.to_string(),
                said_text.to_string(),
                Some(static_pos),
            )
        } else if let Some((said_text, entity_id)) = is_global_csent_message(&full_msg) {
            (
                entity_id,
                GLOBAL_NAME.to_string(),
                said_text.to_string(),
                None,
            )
        } else if let Some((id, real_name, said_text)) = self.find_player_from_message(full_msg) {
            (id, real_name, said_text, None)
        } else {
            return;
        };

        random::update_chat_count(&real_name);

        if let Some(entity) = self.entities.borrow_mut().get(id).and_then(|e| e.upgrade()) {
            // if entity is in our map
            let colorless_text: String = remove_color(said_text).trim().to_string();

            let send_entity = SendEntity::from(&entity);

            let chatsounds = self.chatsounds.clone();
            let entity_emitters = self.entity_emitters.clone();

            // it doesn't matter if these are out of order so we just spawn
            FuturesModule::spawn_future(async move {
                play_chatsound(
                    colorless_text,
                    real_name,
                    send_entity,
                    self_pos,
                    self_rot_yaw,
                    chatsounds,
                    entity_emitters,
                    static_pos,
                )
                .await;
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn play_chatsound(
    sentence: String,
    real_name: String,
    entity: SendEntity,
    self_pos: Vec3,
    self_rot_yaw: f32,
    chatsounds: FutureShared<Option<Chatsounds>>,
    entity_emitters: ThreadShared<Vec<EntityEmitter>>,
    static_pos: Option<Vec3>,
) {
    let mut chatsounds = chatsounds.lock().await;
    let chatsounds = chatsounds.as_mut().unwrap();

    if chatsounds.volume() == 0.0 {
        // don't even play the sound if we have 0 volume
        return;
    }

    if sentence.to_lowercase() == "sh" {
        chatsounds.stop_all();
        entity_emitters.lock().unwrap().clear();
        return;
    }

    if static_pos.is_none() && entity.id == ENTITY_SELF_ID {
        // if self entity, play 2d sound
        let _ignore_error = chatsounds.play(&sentence, get_rng(&real_name)).await;
    } else {
        let channel_volumes = EntityEmitter::coords_to_sink_channel_volumes(
            static_pos.unwrap_or(entity.pos),
            self_pos,
            self_rot_yaw,
        );

        if let Ok((sink, _played_chatsounds)) = chatsounds
            .play_channel_volume(&sentence, get_rng(&real_name), channel_volumes)
            .await
        {
            // don't print other's errors
            entity_emitters
                .lock()
                .unwrap()
                .push(EntityEmitter::new(entity.id, &sink, static_pos));
        }
    }
}

impl IncomingEventListener for ChatsoundsEventListener {
    fn handle_incoming_event(&mut self, event: &IncomingEvent) {
        match event.clone() {
            IncomingEvent::ChatReceived(message, msg_type) => {
                self.handle_chat_received(message, msg_type);
            }

            IncomingEvent::FocusChanged(focused) => {
                let mute_lose_focus = OptionModule::get(MUTE_LOSE_FOCUS_SETTING_NAME)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(true);

                if mute_lose_focus {
                    let chatsounds = self.chatsounds.clone();
                    let last_volume = self.last_volume.clone();

                    FuturesModule::spawn_future(async move {
                        let mut chatsounds = chatsounds.lock().await;
                        let chatsounds = chatsounds.as_mut().unwrap();

                        let mut last_volume = last_volume.lock().await;

                        if !focused {
                            *last_volume = Some(chatsounds.volume());
                            chatsounds.set_volume(0.0);
                        } else if let Some(volume) = *last_volume {
                            chatsounds.set_volume(volume);
                        }
                    });
                }
            }

            IncomingEvent::Tick => {
                // update positions on emitters

                let mut entity_emitters = self.entity_emitters.lock().unwrap();

                let mut to_remove = Vec::with_capacity(entity_emitters.len());
                for (i, emitter) in entity_emitters.iter_mut().enumerate() {
                    if emitter.update(&mut self.entities).is_none() {
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
