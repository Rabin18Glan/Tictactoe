use crate::game::{Game, Mark};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
};

const MUTED: Color = Color::Rgb(110, 125, 150);
const X: Color = Color::Rgb(95, 220, 210);
const O: Color = Color::Rgb(245, 160, 185);

pub fn cells(area: Rect) -> Vec<Rect> {
    let width = area.width.min(39);
    let height = area.height.saturating_sub(10).min(15);
    let board = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + 6,
        width,
        height,
    );
    let cell_width = board.width.saturating_sub(2) / 3;
    let cell_height = board.height.saturating_sub(2) / 3;
    (0..9)
        .map(|i| {
            Rect::new(
                board.x + (i % 3) * (cell_width + 1),
                board.y + (i / 3) * (cell_height + 1),
                cell_width,
                cell_height,
            )
        })
        .collect()
}

fn outcome(game: &Game, role: &str) -> String {
    let player = if role.starts_with("Host") {
        Some(Mark::X)
    } else if role.starts_with("Guest") {
        Some(Mark::O)
    } else {
        None
    };
    if let Some(winner) = game.winner() {
        match player {
            Some(p) if p == winner => "You won".into(),
            Some(_) => "You lost".into(),
            None => format!("{} won", winner.label()),
        }
    } else if game.finished() {
        "Draw".into()
    } else if player.is_none_or(|p| p == game.turn()) {
        "Your turn".into()
    } else {
        "Friend’s turn".into()
    }
}

pub fn copy_button(area: Rect, index: u16) -> Rect {
    let width = area.width.min(76).saturating_sub(4);
    Rect::new(
        area.x + (area.width - width) / 2 + 2,
        area.y + area.height.saturating_sub(13) / 2 + 4 + index * 3,
        width.saturating_sub(4),
        1,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn draw(
    frame: &mut Frame,
    game: &Game,
    selected: usize,
    role: &str,
    status: &str,
    details: &str,
    info: bool,
    can_copy: bool,
    copy_notice: &str,
    animation: std::time::Duration,
) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(15, 19, 29)).fg(Color::White)),
        area,
    );
    if area.width < 32 || area.height < 18 {
        frame.render_widget(
            Paragraph::new("Resize terminal to 32 × 18").centered(),
            area,
        );
        return;
    }
    let title = Rect::new(area.x, area.y + 1, area.width, 1);
    frame.render_widget(
        Paragraph::new("T I C  /  T A C  /  T O E")
            .centered()
            .bold()
            .fg(X),
        title,
    );
    let outcome = outcome(game, role);
    if game.winner().is_some() {
        let label = if outcome == "You won" {
            "YOU WON"
        } else if outcome == "You lost" {
            "YOU LOSE"
        } else if game.winner() == Some(Mark::X) {
            "X WINS"
        } else {
            "O WINS"
        };
        let color = if outcome == "You lost" { O } else { X };
        frame.render_widget(
            Paragraph::new(crate::celebration::banner(label))
                .centered()
                .fg(color)
                .bold(),
            Rect::new(area.x, area.y + 2, area.width, 3),
        );
    } else {
        frame.render_widget(
            Paragraph::new(outcome).centered().fg(MUTED),
            Rect::new(area.x, area.y + 2, area.width, 1),
        );
    }
    let targets = cells(area);
    let first = targets[0];
    let last = targets[8];
    for target in targets.iter().take(2) {
        let x = target.right();
        frame.render_widget(
            Paragraph::new(
                (first.y..last.bottom())
                    .map(|_| "│")
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
            .fg(MUTED),
            Rect::new(x, first.y, 1, last.bottom() - first.y),
        );
    }
    for row in 0..2 {
        let y = targets[row * 3].bottom();
        let line: String = (first.x..last.right())
            .map(|x| {
                if x == targets[0].right() || x == targets[1].right() {
                    '┼'
                } else {
                    '─'
                }
            })
            .collect();
        frame.render_widget(
            Paragraph::new(line).fg(MUTED),
            Rect::new(first.x, y, last.right() - first.x, 1),
        );
    }
    for (index, rect) in cells(area).iter().enumerate() {
        let mark = game.cells[index];
        let winner = game
            .winning_line()
            .is_some_and(|line| line.contains(&index));
        let color = match mark {
            Some(Mark::X) => X,
            Some(Mark::O) => O,
            None => MUTED,
        };
        let active = index == selected;
        let inner = *rect;
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(15, 19, 29)).fg(if winner {
                Color::Yellow
            } else {
                color
            })),
            *rect,
        );
        let glyph = match mark {
            Some(Mark::X) => "╲ ╱\n ╳ \n╱ ╲",
            Some(Mark::O) => "╭─╮\n│ │\n╰─╯",
            None => "",
        };
        if inner.height >= 3 && mark.is_some() {
            frame.render_widget(
                Paragraph::new(glyph).centered().fg(color).bold(),
                Rect::new(inner.x, inner.y + inner.height / 2 - 1, inner.width, 3),
            );
        } else {
            let text = mark
                .map(|m| m.label())
                .unwrap_or(if active { "·" } else { "" });
            frame.render_widget(
                Paragraph::new(text).centered().fg(color).bold(),
                Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1),
            );
        }
    }
    crate::celebration::draw_line(frame, game, &targets, animation);
    let quiet = status == "Move accepted."
        || status.starts_with("Ready.")
        || status.starts_with("Connected.");
    let notice = if quiet {
        if game.finished() {
            "[ r ] Play again"
        } else {
            "Click a square to play"
        }
    } else {
        status
    };
    let footer = area.y + area.height - 3;
    frame.render_widget(
        Paragraph::new(notice).centered().fg(MUTED),
        Rect::new(area.x, footer, area.width, 1),
    );
    frame.render_widget(
        Paragraph::new("↑↓←→ move   ⏎ play   r rematch   i info   q quit")
            .centered()
            .fg(MUTED),
        Rect::new(area.x, footer + 2, area.width, 1),
    );
    if info {
        let width = area.width.min(76).saturating_sub(4);
        let popup = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - 13) / 2,
            width,
            13,
        );
        frame.render_widget(Clear, popup);
        let content = if can_copy {
            "Invite a friend\n\n\n\n\n\n\nAnother computer: same network\n\ni / Esc closes"
                .to_string()
        } else if details.is_empty() {
            "Local two-player game\n\nTake turns on the same keyboard.\n\ni / Esc closes".into()
        } else {
            format!("{details}\n\ni / Esc closes")
        };
        frame.render_widget(
            Paragraph::new(content)
                .centered()
                .wrap(Wrap { trim: false })
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(" Session ")
                        .border_style(Style::default().fg(X)),
                ),
            popup,
        );
        if can_copy {
            for (index, label) in ["[ 1  Same computer ]", "[ 2  Another computer ]"]
                .iter()
                .enumerate()
            {
                frame.render_widget(
                    Paragraph::new(*label)
                        .centered()
                        .bold()
                        .fg(Color::Black)
                        .bg(X),
                    copy_button(area, index as u16),
                );
            }
            frame.render_widget(
                Paragraph::new(copy_notice).centered().fg(X),
                Rect::new(popup.x + 1, popup.y + 10, popup.width.saturating_sub(2), 1),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    #[test]
    fn turn_and_result_follow_each_players_perspective() {
        let mut game = Game::default();
        assert_eq!(outcome(&game, "Host / X"), "Your turn");
        assert_eq!(outcome(&game, "Guest / O"), "Friend’s turn");
        for cell in [0, 3, 1, 4, 2] {
            game.play(game.turn(), cell).unwrap();
        }
        assert_eq!(outcome(&game, "Host / X"), "You won");
        assert_eq!(outcome(&game, "Guest / O"), "You lost");
    }
    #[test]
    fn renders_small_and_normal_terminals_with_matching_hit_targets() {
        for (width, height) in [(20, 10), (32, 18), (80, 24), (120, 40)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            for info in [false, true] {
                terminal
                    .draw(|frame| {
                        draw(
                            frame,
                            &Game::default(),
                            4,
                            "Local",
                            "Move accepted.",
                            "",
                            info,
                            true,
                            "",
                            std::time::Duration::ZERO,
                        )
                    })
                    .unwrap();
            }
            if width >= 32 && height >= 18 {
                let targets = cells(Rect::new(0, 0, width, height));
                assert_eq!(targets.len(), 9);
                for target in targets {
                    assert!(target.width >= 3 && target.height >= 2);
                    assert!(target.bottom() <= height - 3);
                }
            }
        }
    }
}
