use std::collections::HashMap;
use std::hash::Hash;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug)]
enum QuatroIoError {
    InvalidPieceError,
}

/* Piece properties are ordered in enum name alphabetical order.
   Color -> Height -> Shape -> Top.
   It is used to represent board state as Text.
*/

#[derive(Clone, Debug, EnumIter, Eq, Hash, Deserialize, Serialize, PartialEq)]
enum Color {
    Brown,
    White,
}

impl From<Color> for String {
    fn from(c: Color) -> Self {
        match c {
            Color::Brown => Self::from("B"),
            Color::White => Self::from("W"),
        }
    }
}

impl TryFrom<&str> for Color {
    type Error = QuatroIoError;
    fn try_from(c: &str) -> Result<Color, Self::Error> {
        match c {
            "B" => Ok(Color::Brown),
            "W" => Ok(Color::White),
            _ => Err(QuatroIoError::InvalidPieceError),
        }
    }
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, Deserialize, Serialize, PartialEq)]
enum Height {
    Short,
    Tall,
}

impl From<Height> for String {
    fn from(h: Height) -> Self {
        match h {
            Height::Short => Self::from("S"),
            Height::Tall => Self::from("T"),
        }
    }
}

impl TryFrom<&str> for Height {
    type Error = QuatroIoError;
    fn try_from(c: &str) -> Result<Height, Self::Error> {
        match c {
            "S" => Ok(Height::Short),
            "T" => Ok(Height::Tall),
            _ => Err(QuatroIoError::InvalidPieceError),
        }
    }
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, Deserialize, Serialize, PartialEq)]
enum Shape {
    Circle,
    Square,
}

impl From<Shape> for String {
    fn from(s: Shape) -> Self {
        match s {
            Shape::Circle => Self::from("C"),
            Shape::Square => Self::from("S"),
        }
    }
}

impl TryFrom<&str> for Shape {
    type Error = QuatroIoError;
    fn try_from(c: &str) -> Result<Shape, Self::Error> {
        match c {
            "C" => Ok(Shape::Circle),
            "S" => Ok(Shape::Square),
            _ => Err(QuatroIoError::InvalidPieceError),
        }
    }
}

#[derive(Clone, Debug, EnumIter, Eq, Hash, Deserialize, Serialize, PartialEq)]
enum Top {
    Flat,
    Hole,
}

impl From<Top> for String {
    fn from(t: Top) -> Self {
        match t {
            Top::Flat => Self::from("F"),
            Top::Hole => Self::from("H"),
        }
    }
}

impl TryFrom<&str> for Top {
    type Error = QuatroIoError;
    fn try_from(c: &str) -> Result<Top, Self::Error> {
        match c {
            "F" => Ok(Top::Flat),
            "H" => Ok(Top::Hole),
            _ => Err(QuatroIoError::InvalidPieceError),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
struct Piece {
    color: Color,
    height: Height,
    shape: Shape,
    top: Top,
}

impl From<Piece> for String {
    fn from(p: Piece) -> Self {
        return format!(
            "{}{}{}{}",
            Self::from(p.color),
            Self::from(p.height),
            Self::from(p.shape),
            Self::from(p.top)
        )
        .to_string();
    }
}

impl TryFrom<String> for Piece {
    type Error = QuatroIoError;
    fn try_from(text: String) -> Result<Piece, Self::Error> {
        if text.len() != 4 {
            return Err(QuatroIoError::InvalidPieceError);
        }
        let color = &text[0..1];
        let height = &text[1..2];
        let shape = &text[2..3];
        let top = &text[3..4];

        Ok(Piece {
            color: Color::try_from(color)?,
            height: Height::try_from(height)?,
            shape: Shape::try_from(shape)?,
            top: Top::try_from(top)?,
        })
    }
}

/* Nothing corresponded to empty cell */
type CellState = Option<Piece>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct Board {
    /* Only 4x4 board size is allowed */
    board: Vec<Vec<CellState>>,
    available_pieces: Vec<Piece>,
}

fn all_pieces() -> Vec<Piece> {
    let mut pieces: Vec<Piece> = Vec::new();
    for c in Color::iter() {
        for s in Shape::iter() {
            for t in Top::iter() {
                for h in Height::iter() {
                    pieces.push(Piece {
                        color: c.clone(),
                        shape: s.clone(),
                        top: t.clone(),
                        height: h.clone(),
                    });
                }
            }
        }
    }
    pieces
}

impl Board {
    pub fn new() -> Self {
        Board {
            board: vec![vec![CellState::None; 4]; 4],
            available_pieces: all_pieces(),
        }
    }
    fn count_elements<S: Clone + Eq + PartialEq + Hash>(
        &self,
        coords: &Vec<(usize, usize)>,
        prop: fn(Piece) -> S,
    ) -> (bool, usize) {
        let picked: Vec<_> = coords
            .into_iter()
            .map(|(x, y)| self.board[*x][*y].clone())
            .collect();
        let picked_property: Vec<Option<S>> = picked
            .clone()
            .iter()
            .map(|opt| opt.as_ref().map(|p| prop(p.clone())))
            .collect();

        let mut hmap: HashMap<Option<S>, usize> = HashMap::new();
        let mut found_none = false;
        for v in picked_property {
            if let None = v {
                found_none = true;
            }
            if let Some(count) = hmap.get(&v) {
                hmap.insert(v, count + 1);
            } else {
                hmap.insert(v, 1);
            }
        }
        (found_none, hmap.len())
    }

    pub fn move_piece(&mut self, p: Piece, x: usize, y: usize) -> bool {
        if x >= 4 || y >= 4 {
            // Out of board access
            return false;
        }
        if let None = self.board[x][y] {
            self.available_pieces.retain(|piece| *piece != p);
            self.board[x][y] = Some(p);
            return true;
        } else {
            // A piece already occupies the position
            return false
        }
    }
    pub fn parse_quatro(
        self,
        coords_vec: Vec<Vec<(usize, usize)>>,
    ) -> Vec<(
        Vec<(usize, usize)>,
        ((bool, usize), (bool, usize), (bool, usize), (bool, usize)),
    )> {
        let mut ret: Vec<(
            Vec<(usize, usize)>,
            ((bool, usize), (bool, usize), (bool, usize), (bool, usize)),
        )> = Vec::new();
        for coords in coords_vec {
            let color_count = &self.count_elements(&coords, |piece| piece.color);
            let shape_count = &self.count_elements(&coords, |piece| piece.shape);
            let height_count = &self.count_elements(&coords, |piece| piece.height);
            let top_count = &self.count_elements(&coords, |piece| piece.top);
            let quatro = (
                color_count.clone(),
                shape_count.clone(),
                height_count.clone(),
                top_count.clone(),
            );
            ret.push((coords.clone(), quatro));
        }

        ret
    }

    pub fn parse_board_text(text: &String) -> Option<Board> {
        let mut game = Board::new();
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() != 4 {
            return None;
        }
        let mut x: usize = 0;
        for line in lines {
            if line.len() != 3 * (4 + 1) + 4 {
                return None;
            }

            for y in 0..4 {
                let piece_text = &line[5 * y..5 * y + 4];
                if piece_text.eq("    ") {
                    game.board[x][y] = None;
                } else {
                    let piece = Piece::try_from(piece_text.to_string()).ok()?;
                    if !game.move_piece(piece, x, y) {
                        // use a piece multiple times
                        return None;
                    }
                }
                if y != 3 {
                    let spacer = &line[5 * y + 4..5 * y + 5];
                    if !spacer.eq(" ") {
                        /* spacer can be any character but this makes board state normalized */
                        return None;
                    }
                }
            }

            x += 1;
        }
        Some(game)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use env_logger::fmt::style::AnsiColor::Black;
    use indoc::indoc;
    #[test]
    fn test_board_new() {
        let board = Board::new();
        assert_eq!(board.board.len(), 4);
        assert_eq!(board.board[0].len(), 4);
        assert_eq!(board.board[1].len(), 4);
        assert_eq!(board.board[2].len(), 4);
        assert_eq!(board.board[3].len(), 4);
        assert_eq!(board.board[0][0], None);
        assert_eq!(board.available_pieces.len(), 16);
    }
    #[test]
    fn test_parse_all_pieces() {
        /* WB TS SC HF */
        let board_text = indoc! {
        r#"BSCF BSCH BSSF BSSH
               BTCF BTCH BTSF BTSH
               WSCF WSCH WSSF WSSH
               WTCF WTCH WTSF WTSH
               "#};

        let board = Board::parse_board_text(&board_text.to_string());

        let expected = vec![
            vec![
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Short,
                    shape: Shape::Circle,
                    top: Top::Flat,
                }),
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Short,
                    shape: Shape::Circle,
                    top: Top::Hole,
                }),
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Short,
                    shape: Shape::Square,
                    top: Top::Flat,
                }),
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Short,
                    shape: Shape::Square,
                    top: Top::Hole,
                }),
            ],
            vec![
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Tall,
                    shape: Shape::Circle,
                    top: Top::Flat,
                }),
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Tall,
                    shape: Shape::Circle,
                    top: Top::Hole,
                }),
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Tall,
                    shape: Shape::Square,
                    top: Top::Flat,
                }),
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Tall,
                    shape: Shape::Square,
                    top: Top::Hole,
                }),
            ],
            vec![
                Some(Piece {
                    color: Color::White,
                    height: Height::Short,
                    shape: Shape::Circle,
                    top: Top::Flat,
                }),
                Some(Piece {
                    color: Color::White,
                    height: Height::Short,
                    shape: Shape::Circle,
                    top: Top::Hole,
                }),
                Some(Piece {
                    color: Color::White,
                    height: Height::Short,
                    shape: Shape::Square,
                    top: Top::Flat,
                }),
                Some(Piece {
                    color: Color::White,
                    height: Height::Short,
                    shape: Shape::Square,
                    top: Top::Hole,
                }),
            ],
            vec![
                Some(Piece {
                    color: Color::White,
                    height: Height::Tall,
                    shape: Shape::Circle,
                    top: Top::Flat,
                }),
                Some(Piece {
                    color: Color::White,
                    height: Height::Tall,
                    shape: Shape::Circle,
                    top: Top::Hole,
                }),
                Some(Piece {
                    color: Color::White,
                    height: Height::Tall,
                    shape: Shape::Square,
                    top: Top::Flat,
                }),
                Some(Piece {
                    color: Color::White,
                    height: Height::Tall,
                    shape: Shape::Square,
                    top: Top::Hole,
                }),
            ],
        ];
        assert_eq!(expected, board.clone().unwrap().board);
        assert_eq!(board.unwrap().available_pieces.len(), 0)
    }
    #[test]
    fn test_empty_board() {
        let dummy_text = indoc! {
        /* - will be replaced to space */
        r#"
            BSCF ---- ---- ----
            ---- ---- ---- ----
            ---- ---- ---- ----
            ---- ---- ---- ----"#};
        let board_text = dummy_text.replace("-", " ");

        let board = Board::parse_board_text(&board_text.to_string());
        let expected = vec![
            vec![
                Some(Piece {
                    color: Color::Brown,
                    height: Height::Short,
                    shape: Shape::Circle,
                    top: Top::Flat,
                }),
                None,
                None,
                None,
            ],
            vec![None, None, None, None],
            vec![None, None, None, None],
            vec![None, None, None, None],
        ];
        assert_eq!(expected, board.clone().unwrap().board);
        assert_eq!(board.unwrap().available_pieces.len(), 15)
    }
    #[test]
    fn test_judge_quatro() {
        let board_text = indoc! {
        r#"BSCF WWSB BSSW WWSB
           BSSW WWSB BSSW WWSB
           BSSW WWSB BSSW WWSB
           BSSW WWSB BSSW WWSB
           "#};
    }
}
