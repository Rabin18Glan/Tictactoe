use crate::game::Game;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Stylize},
    widgets::Paragraph,
};
use std::time::Duration;

/// Rasterize the winning stroke so horizontal, vertical and diagonal wins share
/// the same animation speed and always connect the centers of all three cells.
pub fn stroke(start: (u16, u16), end: (u16, u16), elapsed: Duration) -> Vec<(u16, u16)> {
    let dx = i32::from(end.0) - i32::from(start.0);
    let dy = i32::from(end.1) - i32::from(start.1);
    let steps = dx.abs().max(dy.abs()).max(1);
    let progress = (elapsed.as_secs_f32() / 0.7).clamp(0.0, 1.0);
    let count = (steps as f32 * progress) as i32;
    (0..=count)
        .map(|i| {
            (
                (i32::from(start.0) + dx * i / steps) as u16,
                (i32::from(start.1) + dy * i / steps) as u16,
            )
        })
        .collect()
}

pub fn draw_line(frame: &mut Frame, game: &Game, cells: &[Rect], elapsed: Duration) {
    let Some(line) = game.winning_line() else {
        return;
    };
    let center = |index: usize| {
        let c = cells[index];
        (c.x + c.width / 2, c.y + c.height / 2)
    };
    let start = center(line[0]);
    let end = center(line[2]);
    let glyph = if start.1 == end.1 {
        "━"
    } else if start.0 == end.0 {
        "┃"
    } else if start.0 < end.0 {
        "╲"
    } else {
        "╱"
    };
    let color = if elapsed.as_millis() < 1100 && (elapsed.as_millis() / 150).is_multiple_of(2) {
        Color::White
    } else {
        Color::Yellow
    };
    for (x, y) in stroke(start, end, elapsed) {
        frame.render_widget(
            Paragraph::new(glyph).fg(color).bold(),
            Rect::new(x, y, 1, 1),
        );
    }
}

pub fn banner(text: &str) -> String {
    let mut rows = [String::new(), String::new(), String::new()];
    for letter in text.chars() {
        let glyph = match letter {
            'Y' => ["█ █", " █ ", " █ "],
            'O' => ["█▀█", "█ █", "█▄█"],
            'U' => ["█ █", "█ █", "▀▄▀"],
            'W' => ["█ █ █", "█ █ █", "▀▄▀▄▀"],
            'N' => ["█▀█", "█ █", "█ █"],
            'L' => ["█  ", "█  ", "█▄▄"],
            'S' => ["█▀▀", "▀▀█", "▄▄█"],
            'E' => ["█▀▀", "█▀ ", "█▄▄"],
            'X' => ["▀▄▀", " █ ", "▄▀▄"],
            'I' => ["▀█▀", " █ ", "▄█▄"],
            _ => ["   ", "   ", "   "],
        };
        for (row, line) in rows.iter_mut().zip(glyph) {
            if !row.is_empty() {
                row.push(' ');
            }
            row.push_str(line);
        }
    }
    rows.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn winning_stroke_grows_to_both_endpoints_in_all_directions() {
        for end in [(30, 2), (2, 14), (30, 14), (0, 14)] {
            let start = (2, 2);
            let initial = stroke(start, end, Duration::ZERO);
            let middle = stroke(start, end, Duration::from_millis(350));
            let complete = stroke(start, end, Duration::from_secs(1));
            assert_eq!(initial, [start]);
            assert!(middle.len() > initial.len() && middle.len() < complete.len());
            assert_eq!(complete.last(), Some(&end));
        }
    }
}
