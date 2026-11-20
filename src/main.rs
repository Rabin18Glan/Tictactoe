mod clipboard;
mod game;
mod invitation;
mod multiplayer;
mod protocol;
mod session;
mod transport;
mod ui;

use clap::{Parser, Subcommand};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    MouseButton, MouseEventKind,
};
use game::{Game, Mark};
use multiplayer::Host;
use protocol::{Request, Snapshot};
use std::{
    io,
    net::{TcpStream, ToSocketAddrs},
    path::PathBuf,
    time::Duration,
};
use transport::Connection;

#[derive(Parser)]
#[command(
    version,
    about = "Terminal tic-tac-toe. Run without arguments for local two-player play."
)]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,
}
#[derive(Subcommand)]
enum Mode {
    /// Host a game. Share your reachable IP:port and the printed invitation.
    Host {
        #[arg(long, default_value = "0.0.0.0:7878")]
        bind: String,
    },
    /// Join or resume a game. Keep the session file private and reuse it to reconnect.
    Join {
        address: String,
        #[arg(long)]
        invite: String,
        #[arg(long, default_value = ".tictactoe-session")]
        session: PathBuf,
    },
}
fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut host = None;
    let mut guest = None;
    let mut details = String::new();
    let mut join_command = None;
    let mut clipboard = clipboard::Clipboard::default();
    let mut copy_notice = String::new();
    let role;
    match cli.mode {
        Some(Mode::Host { bind }) => {
            let invitation = uuid::Uuid::new_v4().to_string();
            host = Some(Host::bind(&bind, invitation.clone())?);
            role = "Host / X";
            match invitation::command(host.as_ref().unwrap().address()?, &invitation) {
                Ok(command) => {
                    details = format!("Local network · share with your friend\n\n{command}");
                    join_command = Some(command);
                }
                Err(_) => {
                    details = format!(
                        "Could not detect IP. Use your reachable IP and port.\nInvitation: {invitation}"
                    )
                }
            }
        }
        Some(Mode::Join {
            address,
            invite,
            session,
        }) => {
            let identity = session::identity(&session)?;
            let addresses: Vec<_> = address.to_socket_addrs()?.collect();
            let stream = addresses
                .iter()
                .find_map(|a| TcpStream::connect_timeout(a, Duration::from_secs(3)).ok())
                .ok_or_else(|| {
                    io::Error::other(
                        "Cannot reach host. Check address, firewall, and port forwarding.",
                    )
                })?;
            let mut connection = Connection::new(stream)?;
            connection.send(&Request::Join {
                invitation: invite,
                identity,
            })?;
            guest = Some(connection);
            role = "Guest / O";
            details = format!(
                "Host: {address}\nReconnect with the same command and session file: {}",
                session.display()
            );
        }
        None => role = "Local / two players",
    }
    let mut game = Game::default();
    let mut selected = 0;
    let mut status = "Ready. Waiting for players if hosting.".to_string();
    let mut joined = false;
    let mut terminal = ratatui::init();
    let mut info = host.is_some();
    let result = (|| -> io::Result<()> {
        crossterm::execute!(io::stdout(), EnableMouseCapture)?;
        loop {
            if let Some(h) = &mut host {
                h.tick(&mut game, &mut status)?;
            }
            if let Some(g) = &mut guest {
                match g.receive::<Snapshot>() {
                    Ok(snapshots) => {
                        for snapshot in snapshots {
                            game = snapshot.game;
                            joined = snapshot.connected;
                            status = snapshot.message;
                        }
                    }
                    Err(_) => {
                        guest = None;
                        joined = false;
                        status = "Disconnected. Quit and rerun the join command to resume.".into();
                    }
                }
            }
            terminal.draw(|frame| {
                ui::draw(
                    frame,
                    &game,
                    selected,
                    role,
                    &status,
                    &details,
                    info,
                    join_command.is_some(),
                    &copy_notice,
                )
            })?;
            if !event::poll(Duration::from_millis(100))? {
                continue;
            }
            let code = match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    key.code
                }
                Event::Mouse(mouse)
                    if info
                        && join_command.is_some()
                        && mouse.kind == MouseEventKind::Down(MouseButton::Left) =>
                {
                    let size = terminal.size()?;
                    if ui::copy_button(ratatui::layout::Rect::new(0, 0, size.width, size.height))
                        .contains((mouse.column, mouse.row).into())
                    {
                        KeyCode::Char('c')
                    } else {
                        continue;
                    }
                }
                Event::Mouse(mouse) if !info => {
                    if let Some(index) = ui::cells(ratatui::layout::Rect::new(
                        0,
                        0,
                        terminal.size()?.width,
                        terminal.size()?.height,
                    ))
                    .iter()
                    .position(|rect| rect.contains((mouse.column, mouse.row).into()))
                    {
                        selected = index;
                        if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                            KeyCode::Enter
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };
            if info && code == KeyCode::Char('c') {
                if let Some(command) = &join_command {
                    copy_notice = clipboard
                        .copy(command)
                        .unwrap_or("Clipboard unavailable")
                        .into();
                }
                continue;
            }
            if code == KeyCode::Char('i') || (info && code == KeyCode::Esc) {
                info = !info;
                continue;
            }
            if matches!(code, KeyCode::Char('q') | KeyCode::Esc) {
                break;
            }
            if info {
                continue;
            }
            match code {
                KeyCode::Left => selected = (selected + 8) % 9,
                KeyCode::Right => selected = (selected + 1) % 9,
                KeyCode::Up => selected = (selected + 6) % 9,
                KeyCode::Down => selected = (selected + 3) % 9,
                KeyCode::Char('r') => {
                    if let Some(h) = &mut host {
                        h.rematch(0, &mut game);
                        status = "Rematch requested; waiting for friend.".into();
                    } else if let Some(g) = &mut guest {
                        g.send(&Request::Rematch)?;
                    } else if role.starts_with("Local") {
                        game = Game::default();
                    }
                }
                KeyCode::Enter | KeyCode::Char('1'..='9') => {
                    if let KeyCode::Char(c) = code {
                        selected = (c as u8 - b'1') as usize;
                    }
                    if let Some(h) = &host {
                        if !h.connected() {
                            status = "Wait for your friend to connect.".into();
                            continue;
                        }
                        status = game
                            .play(Mark::X, selected)
                            .err()
                            .unwrap_or("Move accepted.")
                            .into();
                    } else if let Some(g) = &mut guest {
                        if joined {
                            g.send(&Request::Move {
                                cell: selected,
                                revision: game.moves,
                            })?;
                        }
                    } else if role.starts_with("Local") {
                        status = game
                            .play(game.turn(), selected)
                            .err()
                            .unwrap_or("Move accepted.")
                            .into();
                    }
                }
                _ => {}
            }
        }
        Ok(())
    })();
    let _ = crossterm::execute!(io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}
