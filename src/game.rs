use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
use actix_web::web;
use actix_web_actors::ws;
use base64::Engine;
use rand::Rng;

#[derive(Clone)]
pub struct UserShared {
    pub user: Arc<User>,
}
pub struct User {
    pub lobby: Mutex<Option<LobbyShared>>,
    pub name: RwLock<Option<String>>,
    pub lobby_list: web::Data<LobbyListShared>,
    pub socket: RwLock<Option<Addr<UserShared>>>,
}
impl UserShared {
    pub fn send_message(&self, message: MessageS2C) {
        self.user
            .socket
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .try_send(message)
            .unwrap();
    }
}
pub type LobbyShared = Arc<Lobby>;
impl Actor for UserShared {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UserShared {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        {
            let mut socket = self.user.socket.write().unwrap();
            if socket.is_none() {
                *socket = Some(ctx.address());
            }
        }
        match msg {
            Ok(ws::Message::Close(_)) => {
                if let Some(lobby) = &mut *self.user.lobby.lock().unwrap() {
                    lobby.player_leave(self);
                }
                println!("closed");
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                let message: MessageC2S = serde_json::from_str(&text).unwrap();
                match message {
                    MessageC2S::CreateRoom { name } => {
                        if name.trim().is_empty() {
                            self.send_message(MessageS2C::SendAlert {
                                text: "name cannot be empty".to_string(),
                            });
                            return;
                        }
                        *self.user.name.write().unwrap() = Some(name);
                        let in_lobby = self.user.lobby.lock().unwrap().is_some();
                        assert!(!in_lobby);
                        let lobby = self.user.lobby_list.create();
                        *self.user.lobby.lock().unwrap() = Some(lobby.clone());
                        assert!(lobby.player_try_join(self));
                    }
                    MessageC2S::JoinRoom { name, room_id } => {
                        if name.trim().is_empty() {
                            self.send_message(MessageS2C::SendAlert {
                                text: "name cannot be empty".to_string(),
                            });
                            return;
                        }
                        *self.user.name.write().unwrap() = Some(name);
                        let in_lobby = self.user.lobby.lock().unwrap().is_some();
                        assert!(!in_lobby);
                        let lobby = self.user.lobby_list.get_lobby(&room_id);
                        if let Some(lobby) = lobby {
                            if !lobby.player_try_join(self) {
                                self.send_message(MessageS2C::SendAlert {
                                    text: "name taken".to_string(),
                                });
                            }
                        } else {
                            self.send_message(MessageS2C::SendAlert {
                                text: "lobby not found".to_string(),
                            });
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}
pub struct Lobby {
    id: String,
    users: Mutex<HashMap<String, UserShared>>,
}
impl Lobby {
    pub fn new() -> LobbyShared {
        let mut random_data = [0u8; 6];
        rand::thread_rng().fill(&mut random_data);
        Arc::new(Lobby {
            id: base64::engine::general_purpose::STANDARD_NO_PAD.encode(random_data),
            users: Mutex::new(HashMap::new()),
        })
    }
    pub fn player_try_join(&self, player: &UserShared) -> bool {
        {
            let mut users = self.users.lock().unwrap();
            let name = { player.user.name.read().unwrap().as_ref().unwrap().clone() };
            if users.contains_key(name.as_str()) {
                return false;
            }
            users.insert(name, player.clone());
        }
        self.resend_ui();
        true
    }
    pub fn player_leave(&self, player: &UserShared) {
        println!("leave: {}", self.users.lock().unwrap().len());
        self.users
            .lock()
            .unwrap()
            .remove(player.user.name.read().unwrap().as_ref().unwrap().as_str());
        println!("leavep: {}", self.users.lock().unwrap().len());
        self.resend_ui();
    }
    pub fn resend_ui(&self) {
        let message = MessageS2C::PreGameUI {
            room_id: self.id.clone(),
            players: self
                .users
                .lock()
                .unwrap()
                .keys()
                .map(|name| name.clone())
                .collect(),
        };
        for user in self.users.lock().unwrap().values() {
            user.send_message(message.clone());
        }
    }
}
pub struct LobbyList {
    lobbies: Mutex<HashMap<String, LobbyShared>>,
}
pub type LobbyListShared = Arc<LobbyList>;
impl LobbyList {
    pub fn new() -> LobbyListShared {
        Arc::new(LobbyList {
            lobbies: Mutex::new(HashMap::new()),
        })
    }
    pub fn create(&self) -> LobbyShared {
        let lobby = Lobby::new();
        let id = lobby.id.clone();
        self.lobbies.lock().unwrap().insert(id, lobby.clone());
        lobby
    }
    pub fn get_lobby(&self, id: &str) -> Option<LobbyShared> {
        let id = id.trim();
        self.lobbies
            .lock()
            .unwrap()
            .get(id)
            .map(|lobby| lobby.clone())
    }
}
#[derive(serde::Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum MessageC2S {
    CreateRoom { name: String },
    JoinRoom { name: String, room_id: String },
}
#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum MessageS2C {
    SendAlert {
        text: String,
    },
    PreGameUI {
        room_id: String,
        players: Vec<String>,
    },
}
impl Message for MessageS2C {
    type Result = std::io::Result<()>;
}
impl Handler<MessageS2C> for UserShared {
    type Result = std::io::Result<()>;
    fn handle(&mut self, msg: MessageS2C, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(serde_json::to_string(&msg)?);
        Ok(())
    }
}
