use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::hash::Hash;

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;

use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum QuartoError {
    InvalidPieceError,
    FileExists,
    OutOfRange,
    InvalidQuarto,
    AnyOther,
}

/* Piece properties are ordered in enum name alphabetical order.
   Color -> Height -> Shape -> Top.
   It is used to represent board state as Text.
*/

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, Deserialize, Serialize, PartialEq)]
pub enum Color {
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
    type Error = QuartoError;
    fn try_from(c: &str) -> Result<Color, Self::Error> {
        match c {
            "B" => Ok(Color::Brown),
            "W" => Ok(Color::White),
            _ => Err(QuartoError::InvalidPieceError),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, Deserialize, Serialize, PartialEq)]
pub enum Height {
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
    type Error = QuartoError;
    fn try_from(c: &str) -> Result<Height, Self::Error> {
        match c {
            "S" => Ok(Height::Short),
            "T" => Ok(Height::Tall),
            _ => Err(QuartoError::InvalidPieceError),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, Deserialize, Serialize, PartialEq)]
pub enum Shape {
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
    type Error = QuartoError;
    fn try_from(c: &str) -> Result<Shape, Self::Error> {
        match c {
            "C" => Ok(Shape::Circle),
            "S" => Ok(Shape::Square),
            _ => Err(QuartoError::InvalidPieceError),
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, Deserialize, Serialize, PartialEq)]
pub enum Top {
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
    type Error = QuartoError;
    fn try_from(c: &str) -> Result<Top, Self::Error> {
        match c {
            "F" => Ok(Top::Flat),
            "H" => Ok(Top::Hole),
            _ => Err(QuartoError::InvalidPieceError),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Serialize, PartialEq)]
pub struct Piece {
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
    type Error = QuartoError;
    fn try_from(text: String) -> Result<Piece, Self::Error> {
        if text.len() != 4 {
            return Err(QuartoError::InvalidPieceError);
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
pub struct BoardState([[CellState; 4]; 4]);

impl TryFrom<&String> for BoardState {
    type Error = QuartoError;
    fn try_from(text: &String) -> Result<Self, Self::Error> {
        let mut bs = [
            [None, None, None, None],
            [None, None, None, None],
            [None, None, None, None],
            [None, None, None, None],
        ];

        let lines: Vec<&str> = text.lines().collect();
        if lines.len() != 4 {
            return Err(QuartoError::InvalidPieceError);
        }
        let mut x: usize = 0;
        let mut piece_count: HashMap<Piece, usize> = HashMap::new();
        for line in lines {
            if line.len() != 3 * (4 + 1) + 4 {
                return Err(QuartoError::InvalidPieceError);
            }

            for y in 0..4 {
                let piece_text = &line[5 * y..5 * y + 4];
                bs[x][y] = Piece::try_from(piece_text.to_string()).ok();
                if let Some(piece) = &bs[x][y] {
                    if let Some(_count) = piece_count.get(piece) {
                        return Err(QuartoError::InvalidPieceError);
                    } else {
                        piece_count.insert(piece.clone(), 0);
                    }
                }

                if y != 3 {
                    let spacer = &line[5 * y + 4..5 * y + 5];
                    if !spacer.eq(" ") {
                        /* spacer can be any character but this makes board state normalized */
                        return Err(QuartoError::InvalidPieceError);
                    }
                }
            }

            x += 1;
        }
        Ok(BoardState(bs))
    }
}

impl From<BoardState> for String {
    fn from(bs: BoardState) -> Self {
        let vv: String =
            bs.0.into_iter()
                .map(|r| {
                    r.into_iter()
                        .map(|c| c.map_or("    ".to_string(), Into::into))
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join("\n");
        vv
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Quarto {
    /* Only 4x4 board size is allowed */
    /* A piece resides one of board_state, avaiable_pieces or next_piece */
    pub board_state: BoardState,
    free_pieces: Vec<Piece>,
    pub next_piece: Option<Piece>,
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

impl TryFrom<&String> for Quarto {
    type Error = QuartoError;
    fn try_from(text: &String) -> Result<Self, Self::Error> {
        let mut quarto = Quarto::new();
        let bs = BoardState::try_from(text)?;
        quarto.free_pieces = Quarto::free_pieces(&bs);
        quarto.board_state = bs;
        Ok(quarto)
    }
}

impl Quarto {
    pub fn new() -> Self {
        Quarto {
            board_state: BoardState([[CellState::None; 4]; 4]),
            free_pieces: all_pieces(),
            next_piece: None,
        }
    }
    fn free_pieces(bs: &BoardState) -> Vec<Piece> {
        let mut pieces = all_pieces();
        for row in &bs.0 {
            for cell in row {
                if let Some(a_piece) = cell {
                    pieces.retain(|x| *x != *a_piece);
                }
            }
        }
        pieces
    }

    fn count_elements<S: Clone + Eq + PartialEq + Hash>(
        &self,
        coords: &[(usize, usize); 4],
        prop: fn(Piece) -> S,
    ) -> (bool, HashMap<Option<S>, usize>) {
        let picked: Vec<_> = coords
            .into_iter()
            .map(|(x, y)| self.board_state.0[*x][*y].clone())
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
        (found_none, hmap)
    }

    pub fn pick_piece(&mut self, p: &Piece) -> bool {
        if self.free_pieces.contains(p) {
            self.free_pieces.retain(|pc| *pc != *p);
            self.next_piece = Some(p.clone());
            true
        } else {
            false
        }
    }
    pub fn move_piece(&mut self, x: usize, y: usize) -> bool {
        if x >= 4 || y >= 4 {
            // Out of board access
            return false;
        }
        if let None = self.board_state.0[x][y] {
            if let Some(p) = &self.next_piece {
                assert!(!self.free_pieces.contains(&p));
                self.board_state.0[x][y] = Some(p.clone());
                self.next_piece = None;
                return true;
            } else {
                return false;
            }
        } else {
            // A piece already occupies the position
            return false;
        }
    }

    fn check_quarto<S: Eq + PartialEq + Hash>(ls: &(bool, HashMap<S, usize>)) -> bool {
        let set = ls.1.values().collect::<HashSet<_>>();
        !ls.0 && set.contains(&(4 as usize))
    }
    fn summarize(
        vv: &Vec<(
            [(usize, usize); 4],
            (
                (bool, HashMap<Option<Color>, usize>),
                (bool, HashMap<Option<Height>, usize>),
                (bool, HashMap<Option<Shape>, usize>),
                (bool, HashMap<Option<Top>, usize>),
            ),
        )>,
    ) -> Vec<[(usize, usize); 4]> {
        let r = vv
            .into_iter()
            .filter(|(_, (cls, hls, sls, tls))| {
                Self::check_quarto(cls)
                    || Self::check_quarto(hls)
                    || Self::check_quarto(sls)
                    || Self::check_quarto(tls)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(l, _)| l.clone())
            .collect::<Vec<_>>();
        r
    }
    pub fn is_quarto(&self) -> bool {
        let vs = self.parse_quarto(vec![
            [(0, 0), (0, 1), (0, 2), (0, 3)],
            [(1, 0), (1, 1), (1, 2), (1, 3)],
            [(2, 0), (2, 1), (2, 2), (2, 3)],
            [(3, 0), (3, 1), (3, 2), (3, 3)],
            [(0, 0), (1, 0), (2, 0), (3, 0)],
            [(0, 1), (1, 1), (2, 1), (3, 1)],
            [(0, 2), (1, 2), (2, 2), (3, 2)],
            [(0, 3), (1, 3), (2, 3), (3, 3)],
            [(0, 0), (1, 1), (2, 2), (3, 3)],
            [(3, 0), (2, 1), (1, 2), (0, 3)],
        ]);
        let res = Self::summarize(&vs);
        res.len() > 0
    }

    fn parse_quarto(
        &self,
        coords_vec: Vec<[(usize, usize); 4]>,
    ) -> Vec<(
        [(usize, usize); 4],
        (
            (bool, HashMap<Option<Color>, usize>),
            (bool, HashMap<Option<Height>, usize>),
            (bool, HashMap<Option<Shape>, usize>),
            (bool, HashMap<Option<Top>, usize>),
        ),
    )> {
        let mut ret: Vec<(
            [(usize, usize); 4],
            (
                (bool, HashMap<Option<Color>, usize>),
                (bool, HashMap<Option<Height>, usize>),
                (bool, HashMap<Option<Shape>, usize>),
                (bool, HashMap<Option<Top>, usize>),
            ),
        )> = Vec::new();
        for coords in coords_vec {
            let color_count = &self.count_elements(&coords, |piece| piece.color);
            let height_count = &self.count_elements(&coords, |piece| piece.height);
            let shape_count = &self.count_elements(&coords, |piece| piece.shape);
            let top_count = &self.count_elements(&coords, |piece| piece.top);
            let quarto = (
                color_count.clone(),
                height_count.clone(),
                shape_count.clone(),
                top_count.clone(),
            );
            ret.push((coords, quarto));
        }

        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    #[test]
    fn test_board_new() {
        let quarto = Quarto::new();
        assert_eq!(quarto.board_state.0.len(), 4);
        assert_eq!(quarto.board_state.0[0].len(), 4);
        assert_eq!(quarto.board_state.0[1].len(), 4);
        assert_eq!(quarto.board_state.0[2].len(), 4);
        assert_eq!(quarto.board_state.0[3].len(), 4);
        assert_eq!(quarto.board_state.0[0][0], None);
        assert_eq!(quarto.free_pieces.len(), 16);
    }

    #[test]
    fn test_parse_board_from_to() {
        /* WB TS SC HF */
        let board_text = indoc! {
        r#"BSCF BSCH BSSF BSSH
           BTCF BTCH BTSF BTSH
           WSCF WSCH WSSF WSSH
           WTCF WTCH WTSF WTSH"#};

        let quarto = Quarto::try_from(&board_text.to_string()).ok();
        let board_text2: String = BoardState::from(quarto.unwrap().board_state).into();
        assert_eq!(board_text, board_text2)
    }

    #[test]
    fn test_parse_all_pieces() {
        /* WB TS SC HF */
        let board_text = indoc! {
        r#"BSCF BSCH BSSF BSSH
               BTCF BTCH BTSF BTSH
               WSCF WSCH WSSF WSSH
               WTCF WTCH WTSF WTSH"#};

        let quarto = Quarto::try_from(&board_text.to_string()).ok();
        assert_ne!(quarto, None);

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
        assert_eq!(expected, quarto.clone().unwrap().board_state.0);
        assert_eq!(quarto.unwrap().free_pieces.len(), 0)
    }
    #[test]
    fn test_use_a_piece_multiple_times() {
        /* WB TS SC HF */
        let board_text = indoc! {
        r#"BSCF BSCH BSSF BSSH
           BTCF BTCH BTSF BTSH
           WSCF WSCH WSSF WSSH
           WTCF WTCH WTSF BSCF"#};

        let quarto = Quarto::try_from(&board_text.to_string()).ok();
        assert_eq!(quarto, None)
    }
    #[test]
    fn test_empty_board() {
        let dummy_text = indoc! {
        /* - will be replaced to space */
        r#"
                 ---- ---- ---- ----
                 ---- ---- ---- ----
                 ---- ---- ---- ----
                 ---- ---- ---- ----"#};
        let board_text = dummy_text.replace("-", " ");

        let quarto = Quarto::try_from(&board_text.to_string()).ok();
        let expected = vec![
            vec![None, None, None, None],
            vec![None, None, None, None],
            vec![None, None, None, None],
            vec![None, None, None, None],
        ];
        assert_eq!(expected, quarto.clone().unwrap().board_state.0);
        assert_eq!(quarto.unwrap().free_pieces.len(), 16);
    }

    #[test]
    fn test_is_quarto() {
        let dummy_text = indoc! {
        /* - will be replaced to space */
        r#"BSCF BSCH BSSF WTSH
           ---- ---- ---- ----
           ---- ---- ---- ----
           ---- ---- ---- ----"#};
        let board_text = dummy_text.replace("-", " ");

        let quarto = &mut Quarto::try_from(&board_text.to_string()).unwrap();
        let no_quarto = quarto.is_quarto();
        assert!(!no_quarto);

        let dummy_texts = vec![
            indoc! {
            r#"BSCF BSCH BSSF BTSH
                   ---- ---- ---- ----
                   ---- ---- ---- ----
                   ---- ---- ---- ----"#},
            indoc! {
            r#"---- ---- ---- ----
                   BSCF BSCH BSSF BTSH
                   ---- ---- ---- ----
                   ---- ---- ---- ----"#},
            indoc! {
                r#"---- ---- ---- ----
                   ---- ---- ---- ----
                   BSCF BSCH BSSF BTSH
                   ---- ---- ---- ----"#
            },
            indoc! {
                r#"---- ---- ---- ----
                   ---- ---- ---- ----
                   ---- ---- ---- ----
                   BSCF BSCH BSSF BTSH"#
            },
            indoc! {
                r#"BSCF ---- ---- ----
                   ---- BSCH ---- ----
                   ---- ---- BSSF ----
                   ---- ---- ---- BTSH"#
            },
            indoc! {
                r#"---- ---- ---- BTSH
                   ---- ---- BSSF ----
                   ---- BSCH ---- ----
                   BTCF ---- ---- ----"#

            },
            indoc! {
                r#"BSCF ---- ---- ----
                   BSCH ---- ---- ----
                   BSSF ---- ---- ----
                   BTSH ---- ---- ----"#

            },
            indoc! {
                r#"---- BSCF ---- ----
                   ---- BSCH ---- ----
                   ---- BSSF ---- ----
                   ---- BTSH ---- ----"#

            },
            indoc! {
                r#"---- ---- BSCF ----
                   ---- ---- BSCH ----
                   ---- ---- BSSF ----
                   ---- ---- BTSH ----"#

            },
            indoc! {
                r#"---- ---- ---- BSCF
                   ---- ---- ---- BSCH
                   ---- ---- ---- BSSF
                   ---- ---- ---- BTSH"#

            },
        ];

        for bt in dummy_texts {
            let board_text = bt.replace("-", " ");

            let quarto = &mut Quarto::try_from(&board_text.to_string()).unwrap();
            let yes_quarto = quarto.is_quarto();

            assert!(yes_quarto);
        }
    }

    #[test]
    fn test_pick_and_move() {
        let dummy_text = indoc! {
        /* - will be replaced to space */
        r#"
               ---- ---- ---- ----
               ---- ---- ---- ----
               ---- ---- ---- ----
               ---- ---- ---- ----"#};
        let board_text = dummy_text.replace("-", " ");

        let quarto = &mut Quarto::try_from(&board_text.to_string()).unwrap();
        let expected: Vec<Vec<Option<Piece>>> = vec![
            vec![None, None, None, None],
            vec![None, None, None, None],
            vec![None, None, None, None],
            vec![None, None, None, None],
        ];
        assert_eq!(expected, quarto.board_state.0);

        let bscf = Piece {
            color: Color::Brown,
            height: Height::Short,
            shape: Shape::Circle,
            top: Top::Flat,
        };

        let succeess = quarto.pick_piece(&bscf);
        assert_eq!(succeess, true);
        let fail = quarto.pick_piece(&bscf);
        assert_eq!(fail, false);
        let success = quarto.move_piece(0, 0);
        assert_eq!(success, true);

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
        assert_eq!(expected, quarto.board_state.0);

        let bssf = Piece {
            color: Color::Brown,
            height: Height::Short,
            shape: Shape::Square,
            top: Top::Flat,
        };
        let success = quarto.pick_piece(&bssf);
        assert!(success);
        let success = quarto.move_piece(0, 2);
        assert!(success);
    }
}
