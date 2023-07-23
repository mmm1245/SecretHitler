mod game;

use std::{
    io::Result,
    sync::{Arc, Mutex, RwLock},
};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use game::{LobbyList, LobbyListShared, User};

use crate::game::UserShared;

async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    app_data: web::Data<LobbyListShared>,
) -> actix_web::Result<HttpResponse> {
    println!("connected");
    ws::start(
        UserShared {
            user: Arc::new(User {
                lobby: Mutex::new(None),
                name: RwLock::new(None),
                lobby_list: app_data.clone(),
                socket: RwLock::new(None),
            }),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> Result<()> {
    let lobby_list = LobbyList::new();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(lobby_list.clone()))
            .route("/ws", web::get().to(websocket_route))
            .service(actix_files::Files::new("", "./web").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
