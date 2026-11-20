use crate::game::{Game, Mark};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
};

const MUTED: Color = Color::Rgb(110, 125, 150);
const X: Color = Color::Rgb(95, 220, 210);
const O: Color = Color::Rgb(245, 160, 185);

pub fn cells(area: Rect) -> Vec<Rect> {
    let width = area.width.min(39);
    let height = area.height.saturating_sub(8).min(15);
    let board = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + 4,
        width,
        height,
    );
    let rows = Layout::vertical([Constraint::Ratio(1, 3); 3]).split(board);
    rows.iter()
        .flat_map(|row| {
            Layout::horizontal([Constraint::Ratio(1, 3); 3])
                .split(*row)
                .to_vec()
        })
        .collect()
}

pub fn copy_button(area: Rect) -> Rect {
    let width = area.width.min(76).saturating_sub(4);
    Rect::new(
        area.x + (area.width - width) / 2 + 2,
        area.y + area.height.saturating_sub(13) / 2 + 10,
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
    let outcome = if let Some(winner) = game.winner() {
        format!("{} wins!   ✦", winner.label())
    } else if game.finished() {
        "Draw — well played".into()
    } else {
        format!("{}'s turn", game.turn().label())
    };
    frame.render_widget(
        Paragraph::new(format!("{role}   ·   {outcome}"))
            .centered()
            .fg(MUTED),
        Rect::new(area.x, area.y + 2, area.width, 1),
    );
    for (index, rect) in cells(area).iter().enumerate() {
        let mark = game.cells[index];
        let winner = mark.is_some() && mark == game.winner();
        let color = match mark {
            Some(Mark::X) => X,
            Some(Mark::O) => O,
            None => MUTED,
        };
        let active = index == selected;
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(if active {
                Color::White
            } else if winner {
                Color::Yellow
            } else {
                Color::Rgb(45, 57, 77)
            }))
            .style(Style::default().bg(if active {
                Color::Rgb(35, 46, 65)
            } else {
                Color::Rgb(20, 26, 38)
            }));
        let inner = block.inner(*rect);
        frame.render_widget(block, *rect);
        let glyph = match mark {
            Some(Mark::X) => "╲ ╱\n ╳ \n╱ ╲",
            Some(Mark::O) => "╭─╮\n│ │\n╰─╯",
            None => "",
        };
        if inner.height >= 3 && mark.is_some() {
            frame.render_widget(
                Paragraph::new(glyph).centered().fg(color).bold(),
                Rect::new(inner.x, inner.y + (inner.height - 3) / 2, inner.width, 3),
            );
        } else {
            let text = mark
                .map(|m| m.label().to_string())
                .unwrap_or_else(|| (index + 1).to_string());
            frame.render_widget(
                Paragraph::new(text).centered().fg(color).bold(),
                Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1),
            );
        }
    }
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
        frame.render_widget(Paragraph::new(if details.is_empty() { "Local two-player game\n\nTake turns on the same keyboard.\n\nPress i or Esc to close.".into() } else { format!("{details}\n\nShift+drag to select · i / Esc closes") }).wrap(Wrap { trim:false }).block(Block::bordered().border_type(BorderType::Rounded).title(" Session ").border_style(Style::default().fg(X))),popup);
        if can_copy {
            let label = if copy_notice.is_empty() {
                "[ c  Copy command ]"
            } else {
                copy_notice
            };
            frame.render_widget(
                Paragraph::new(label)
                    .centered()
                    .bold()
                    .fg(Color::Black)
                    .bg(X),
                copy_button(area),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
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
                        )
                    })
                    .unwrap();
            }
            if width >= 32 && height >= 18 {
                let targets = cells(Rect::new(0, 0, width, height));
                assert_eq!(targets.len(), 9);
                for target in targets {
                    assert!(target.width >= 3 && target.height >= 3);
                    assert!(target.bottom() <= height - 3);
                }
            }
        }
    }
}
