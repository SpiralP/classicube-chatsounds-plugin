use crate::{
  command::VOLUME_SETTING_NAME,
  entities::{ENTITIES, ENTITY_SELF_ID},
  entity_emitter::{EntityEmitter, ENTITY_EMITTERS},
  helpers::remove_color,
  option,
  printer::{print, status},
  tablist::TABLIST,
  thread,
};
use chatsounds::Chatsounds;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use rand::seq::SliceRandom;
use std::{fs, path::Path};

pub const VOLUME_NORMAL: f32 = 0.1;

lazy_static! {
  pub static ref CHATSOUNDS: Mutex<Option<Chatsounds>> = Mutex::new(None);
}

pub fn load() {
  print(format!(
    "Loading Chatsounds v{}...",
    env!("CARGO_PKG_VERSION")
  ));

  if fs::metadata("plugins")
    .map(|meta| meta.is_dir())
    .unwrap_or(false)
  {
    let path = Path::new("plugins/chatsounds");
    fs::create_dir_all(path).unwrap();

    let mut chatsounds = Chatsounds::new(path);

    let volume = option::get(VOLUME_SETTING_NAME)
      .and_then(|s| s.parse().ok())
      .unwrap_or(1.0);

    // TODO 0 volume doesn't event play anything
    chatsounds.set_volume(VOLUME_NORMAL * volume);

    *CHATSOUNDS.lock() = Some(chatsounds);
  } else {
    panic!("UH OH");
  }

  thread::spawn("chatsounds source loader", move || {
    status("chatsounds fetching sources...");
    load_sources();
    status("done fetching sources");
  });
}

pub fn unload() {
  *CHATSOUNDS.lock() = None;
}

enum Source {
  Api(&'static str, &'static str),
  Msgpack(&'static str, &'static str),
}

const SOURCES: &[Source] = &[
  Source::Api(
    "Metastruct/garrysmod-chatsounds",
    "sound/chatsounds/autoadd",
  ),
  Source::Api("PAC3-Server/chatsounds", "sounds/chatsounds"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "csgo"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "css"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "ep1"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "ep2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "hl2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "l4d"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "l4d2"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "portal"),
  Source::Msgpack("PAC3-Server/chatsounds-valve-games", "tf2"),
];

fn load_sources() {
  let sources_len = SOURCES.len();
  for (i, source) in SOURCES.iter().enumerate() {
    if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
      let (repo, repo_path) = match source {
        Source::Api(repo, repo_path) => (repo, repo_path),
        Source::Msgpack(repo, repo_path) => (repo, repo_path),
      };

      status(format!(
        "[{}/{}] fetching {} {}",
        i + 1,
        sources_len,
        repo,
        repo_path
      ));

      match source {
        Source::Api(repo, repo_path) => chatsounds.load_github_api(repo, repo_path),
        Source::Msgpack(repo, repo_path) => chatsounds.load_github_msgpack(repo, repo_path),
      }
    }
  }
}

pub fn play_chatsound(entity_id: usize, sentence: String) {
  // TODO need the main thread tick to update positions

  let (emitter_pos, self_stuff) = ENTITIES.with(|entities| {
    let entities = entities.borrow();

    (
      if let Some(entity) = entities.get(&entity_id) {
        Some(entity.get_pos())
      } else {
        None
      },
      if let Some(entity) = entities.get(&ENTITY_SELF_ID) {
        Some((entity.get_pos(), entity.get_rot()[1]))
      } else {
        print(format!(
          "couldn't get entity.get_pos/rot() {}",
          ENTITY_SELF_ID
        ));
        None
      },
    )
  });

  // TODO use 1 thread and a channel
  thread::spawn("chatsounds handle message", move || {
    if let Some(chatsounds) = CHATSOUNDS.lock().as_mut() {
      if sentence.to_lowercase() == "sh" {
        chatsounds.stop_all();
        return;
      }

      if let Some(sounds) = chatsounds.get(sentence) {
        let mut rng = rand::thread_rng();

        if let Some(sound) = sounds.choose(&mut rng).cloned() {
          if entity_id == ENTITY_SELF_ID {
            // if self entity, play 2d sound
            chatsounds.play(&sound);
          } else if let Some(emitter_pos) = emitter_pos {
            if let Some((self_pos, self_rot)) = self_stuff {
              let (emitter_pos, left_ear_pos, right_ear_pos) =
                EntityEmitter::coords_to_sink_positions(emitter_pos, self_pos, self_rot);

              // if sound.can_be_3d() {
              let sink = chatsounds.play_spatial(&sound, emitter_pos, left_ear_pos, right_ear_pos);

              ENTITY_EMITTERS
                .lock()
                .push(EntityEmitter::new(entity_id, sink));
              // } else {
              //   unimplemented!()
              // }
            }
          }
        }
      }
    }
  });
}

pub fn handle_chat_message<S: Into<String>>(full_msg: S) {
  // &]SpiralP: &faaa
  let full_msg = full_msg.into();

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
    let found_entity_id = TABLIST.with(|tablist| {
      let tablist = tablist.borrow();

      // try exact match
      tablist
        .iter()
        .find_map(|(id, entry)| {
          if entry.nick_name == full_nick {
            Some(*id)
          } else {
            None
          }
        })
        .or_else(|| {
          // match from the right, choose the one with most chars matched
          let mut id_positions: Vec<(usize, usize)> = tablist
            .iter()
            .filter_map(|(id, entry)| {
              // full_nick &g[&x&7___&g] &m___&0 Cjnator38
              // real_nick &g&m___&0 Cjnator38

              // &7<Map>&r&r[&f/dl&r] Empy: &fthis milk chocolate fuck has

              // remove color at beginning
              let full_nick = if full_nick.len() >= 2 && full_nick.starts_with('&') {
                let (_color, full_nick) = full_nick.split_at(2);
                full_nick
              } else {
                full_nick.as_str()
              };
              let real_nick = if entry.nick_name.len() >= 2 && entry.nick_name.starts_with('&') {
                let (_color, real_nick) = entry.nick_name.split_at(2);
                real_nick
              } else {
                entry.nick_name.as_str()
              };

              full_nick.rfind(&real_nick).map(|pos| (*id, pos))
            })
            .collect();

          // choose smallest position, or "most chars matched"
          id_positions.sort_unstable_by(|(id1, pos1), (id2, pos2)| {
            pos1
              .partial_cmp(pos2)
              .unwrap()
              .then_with(|| id1.partial_cmp(&id2).unwrap())
          });

          id_positions.first().map(|(id, _pos)| *id)
        })
    });

    if let Some(entity_id) = found_entity_id {
      // print(format!("FOUND {} {}", entity_id, full_nick));

      play_chatsound(entity_id, colorless_text);
      // } else {
      // print(format!("not found {}", full_nick));
    }
  }
}
