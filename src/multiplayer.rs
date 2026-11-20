use crate::{
    game::{Game, Mark},
    protocol::{Request, Snapshot},
    transport::Connection,
};
use std::{
    io,
    net::TcpListener,
    time::{Duration, Instant},
};

pub struct Host {
    listener: TcpListener,
    invitation: String,
    identity: Option<String>,
    peer: Option<Connection>,
    pending: Vec<(Connection, Instant)>,
    votes: [bool; 2],
}
impl Host {
    pub fn bind(address: &str, invitation: String) -> io::Result<Self> {
        let listener = TcpListener::bind(address)?;
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            invitation,
            identity: None,
            peer: None,
            pending: Vec::new(),
            votes: [false; 2],
        })
    }
    pub fn address(&self) -> io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }
    pub fn connected(&self) -> bool {
        self.peer.is_some()
    }
    pub fn rematch(&mut self, player: usize, game: &mut Game) {
        if game.finished() {
            self.votes[player] = true;
        }
        if self.votes == [true, true] {
            *game = Game::default();
            self.votes = [false; 2];
        }
    }
    pub fn tick(&mut self, game: &mut Game, status: &mut String) -> io::Result<()> {
        // Limit admission work per frame and require a handshake within three seconds.
        for _ in 0..8 {
            match self.listener.accept() {
                Ok((stream, _)) if self.pending.len() < 8 => self
                    .pending
                    .push((Connection::new(stream)?, Instant::now())),
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        let mut retained = Vec::new();
        for (mut conn, since) in self.pending.drain(..) {
            if since.elapsed() > Duration::from_secs(3) {
                continue;
            }
            match conn.receive::<Request>() {
                Ok(messages) if messages.is_empty() => retained.push((conn, since)),
                Ok(messages) => {
                    if let Some(Request::Join {
                        invitation,
                        identity,
                    }) = messages.first()
                    {
                        let valid = uuid::Uuid::parse_str(identity).is_ok();
                        if valid
                            && invitation == &self.invitation
                            && self.peer.is_none()
                            && self.identity.as_ref().is_none_or(|saved| saved == identity)
                        {
                            self.identity = Some(identity.clone());
                            self.peer = Some(conn);
                            *status = "Friend connected as O.".into();
                        } else {
                            let _ = conn.send(&Snapshot {
                                game: game.clone(),
                                connected: false,
                                message: "Room locked, already connected, or invalid invitation."
                                    .into(),
                            });
                        }
                    }
                }
                Err(_) => {}
            }
        }
        self.pending = retained;
        if let Some(peer) = &mut self.peer {
            match peer.receive::<Request>() {
                Ok(messages) => {
                    for request in messages {
                        match request {
                            Request::Move { cell, revision } if revision == game.moves => {
                                if let Err(e) = game.play(Mark::O, cell) {
                                    *status = e.into();
                                }
                            }
                            Request::Rematch => self.votes[1] = game.finished(),
                            _ => {}
                        }
                    }
                }
                Err(_) => {
                    self.peer = None;
                    *status = "Friend disconnected. Their seat is reserved.".into();
                }
            }
        }
        if self.votes == [true, true] {
            *game = Game::default();
            self.votes = [false; 2];
            *status = "New round!".into();
        }
        let message = if self.votes.contains(&true) {
            "Rematch requested. Both players must press r."
        } else {
            "Connected. Press r after the round to request a rematch."
        };
        if let Some(peer) = &mut self.peer
            && peer
                .send(&Snapshot {
                    game: game.clone(),
                    connected: true,
                    message: message.into(),
                })
                .is_err()
        {
            self.peer = None;
            *status = "Friend disconnected. Their seat is reserved.".into();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    fn join(host: &Host, identity: &str) -> Connection {
        let mut peer =
            Connection::new(TcpStream::connect(host.listener.local_addr().unwrap()).unwrap())
                .unwrap();
        peer.send(&Request::Join {
            invitation: "invite".into(),
            identity: identity.into(),
        })
        .unwrap();
        peer
    }
    #[test]
    fn reserves_guest_seat_and_restores_board_on_rejoin() {
        let mut host = Host::bind("127.0.0.1:0", "invite".into()).unwrap();
        let mut game = Game::default();
        let mut status = String::new();
        let identity = uuid::Uuid::new_v4().to_string();
        let mut guest = join(&host, &identity);
        host.tick(&mut game, &mut status).unwrap();
        assert!(host.connected());
        game.play(Mark::X, 0).unwrap();
        guest
            .send(&Request::Move {
                cell: 1,
                revision: 1,
            })
            .unwrap();
        host.tick(&mut game, &mut status).unwrap();
        assert_eq!(game.cells[1], Some(Mark::O));
        // A duplicate request cannot apply twice.
        guest
            .send(&Request::Move {
                cell: 2,
                revision: 1,
            })
            .unwrap();
        host.tick(&mut game, &mut status).unwrap();
        assert_eq!(game.moves, 2);
        drop(guest);
        host.tick(&mut game, &mut status).unwrap();
        assert!(!host.connected());
        let _intruder = join(&host, &uuid::Uuid::new_v4().to_string());
        host.tick(&mut game, &mut status).unwrap();
        assert!(!host.connected());
        let mut returned = join(&host, &identity);
        host.tick(&mut game, &mut status).unwrap();
        assert!(host.connected());
        let snapshots = returned.receive::<Snapshot>().unwrap();
        assert_eq!(snapshots.last().unwrap().game.moves, 2);
        for cell in [3, 2, 6] {
            game.play(game.turn(), cell).unwrap();
        }
        assert!(game.finished());
        host.rematch(0, &mut game);
        assert!(game.finished());
        returned.send(&Request::Rematch).unwrap();
        host.tick(&mut game, &mut status).unwrap();
        assert_eq!(game.moves, 0);
    }
}
