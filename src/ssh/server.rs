use std::{collections::HashMap, sync::Arc};

use super::keys::Key;
use crate::repl::{REPLMode, REPL};
use futures::{executor::block_on, lock::Mutex};
use thrussh::{
    server::{self, Auth, Session},
    ChannelId, CryptoVec,
};
use thrussh_keys::key;

#[derive(Clone)]
pub struct Server {
    pub client_pubkey: Arc<thrussh_keys::key::PublicKey>,
    pub clients: Arc<Mutex<HashMap<(usize, ChannelId), (thrussh::server::Handle, REPL)>>>,
    pub id: usize,
    pub repl_mode: REPLMode,
}

impl server::Server for Server {
    type Handler = Self;
    fn new(&mut self, _: Option<std::net::SocketAddr>) -> Self {
        let s = self.clone();
        self.id += 1;
        s
    }
}

impl server::Handler for Server {
    type Error = anyhow::Error;
    type FutureAuth = futures::future::Ready<Result<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = futures::future::Ready<Result<(Self, Session), anyhow::Error>>;
    type FutureBool = futures::future::Ready<Result<(Self, Session, bool), anyhow::Error>>;

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
        futures::future::ready(Ok((self, auth)))
    }

    fn finished_bool(self, b: bool, s: Session) -> Self::FutureBool {
        futures::future::ready(Ok((self, s, b)))
    }

    fn finished(self, s: Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, s)))
    }

    fn shell_request(self, channel: ChannelId, mut session: Session) -> Self::FutureUnit {
        self.send_line(channel, &mut session, "Welcome to Rocky! Let's be nerds!");
        self.send(channel, &mut session, ">>> ");
        self.finished(session)
    }

    fn channel_open_session(self, channel: ChannelId, session: Session) -> Self::FutureUnit {
        {
            let repl = REPL::new(self.repl_mode).unwrap();
            let mut clients = block_on(self.clients.lock());
            clients.insert((self.id, channel), (session.handle(), repl));
        }
        self.finished(session)
    }

    fn auth_publickey(self, _: &str, _: &key::PublicKey) -> Self::FutureAuth {
        self.finished_auth(server::Auth::Accept)
    }

    fn data(self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit {
        {
            let mut clients = block_on(self.clients.lock());
            for ((id, channel), (ref mut s, _)) in clients.iter_mut() {
                if *id != self.id {
                    block_on(s.data(*channel, CryptoVec::from_slice(data))).unwrap();
                }
            }
        }
        self.handle_input(
            channel,
            &mut session,
            &String::from_utf8(data.to_vec()).unwrap(),
        );
        self.finished(session)
    }
}

impl Server {
    fn handle_input<'a>(&self, channel: ChannelId, session: &mut Session, data: &'a str) {
        let key = Key::from(data);
        println!("{key:?}");
        self.send(channel, session, data);
    }

    fn send_line<'a>(&self, channel: ChannelId, session: &mut Session, line: &'a str) {
        self.send(channel, session, &format!("{line}\n\r"));
        session.flush().unwrap();
    }

    fn send<'a>(&self, channel: ChannelId, session: &mut Session, data: &'a str) {
        session.data(channel, CryptoVec::from_slice(&data.as_bytes()));
    }
}
