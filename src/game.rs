use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mark {
    X,
    O,
}
impl Mark {
    pub fn label(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::O => "O",
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Game {
    pub cells: [Option<Mark>; 9],
    pub moves: usize,
}
impl Game {
    pub fn turn(&self) -> Mark {
        if self.moves.is_multiple_of(2) {
            Mark::X
        } else {
            Mark::O
        }
    }
    pub fn winner(&self) -> Option<Mark> {
        for [a, b, c] in [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            [0, 4, 8],
            [2, 4, 6],
        ] {
            if self.cells[a].is_some()
                && self.cells[a] == self.cells[b]
                && self.cells[b] == self.cells[c]
            {
                return self.cells[a];
            }
        }
        None
    }
    pub fn finished(&self) -> bool {
        self.winner().is_some() || self.moves == 9
    }
    pub fn play(&mut self, player: Mark, cell: usize) -> Result<(), &'static str> {
        if self.finished() {
            return Err("Round finished. Press r for a rematch.");
        }
        if self.turn() != player {
            return Err("Wait for your turn.");
        }
        let slot = self.cells.get_mut(cell).ok_or("Invalid cell.")?;
        if slot.is_some() {
            return Err("That cell is occupied.");
        }
        *slot = Some(player);
        self.moves += 1;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rules_and_win() {
        let mut g = Game::default();
        assert!(g.play(Mark::O, 0).is_err());
        for cell in [0, 3, 1, 4, 2] {
            g.play(g.turn(), cell).unwrap();
        }
        assert_eq!(g.winner(), Some(Mark::X));
        assert!(g.play(Mark::O, 8).is_err());
    }
    #[test]
    fn draw_and_invalid_moves() {
        let mut g = Game::default();
        assert!(g.play(Mark::X, 9).is_err());
        g.play(Mark::X, 0).unwrap();
        assert!(g.play(Mark::O, 0).is_err());
        for cell in [1, 2, 4, 3, 5, 7, 6, 8] {
            g.play(g.turn(), cell).unwrap();
        }
        assert!(g.finished());
        assert_eq!(g.winner(), None);
    }
}
