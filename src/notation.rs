use crate::{
    board::{movegen::MoveList, Board},
    chess_move::ChessMove,
};
use core::fmt;
use mattis_types::PieceType;
use std::{fmt::Write, io};

pub struct FmtBridge<T>(pub T);

impl<T> fmt::Write for FmtBridge<T>
where
    T: io::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

pub trait Notation {
    fn write(w: &mut impl Write, cmove: ChessMove, board: &mut Board) -> std::fmt::Result;
}

pub struct SmithNotation;

impl SmithNotation {
    pub fn write(w: &mut impl Write, cmove: ChessMove) -> std::fmt::Result {
        if cmove.is_nomove() {
            return write!(w, "0000");
        }

        write!(w, "{}{}", cmove.start(), cmove.end())?;

        if let Some(pt) = cmove.promoted() {
            write!(w, "{}", pt.to_char())?;
        }

        Ok(())
    }
}

impl Notation for SmithNotation {
    fn write(w: &mut impl Write, cmove: ChessMove, _board: &mut Board) -> std::fmt::Result {
        SmithNotation::write(w, cmove)
    }
}

pub struct AlgebraicNotation;

impl AlgebraicNotation {
    pub fn write(w: &mut impl Write, cmove: ChessMove, board: &mut Board) -> std::fmt::Result {
        if cmove.is_nomove() {
            return write!(w, "0000");
        } else if cmove.is_kingside_castle() {
            return write!(w, "0-0");
        } else if cmove.is_queenside_castle() {
            return write!(w, "0-0-0");
        }

        let moving_piece = board.pieces[cmove.start()].unwrap();

        let mut movelist = MoveList::new();
        board.generate_all_moves(&mut movelist);

        let mut ambiguities = movelist
            .iter()
            .filter(|m| **m != cmove && board.pieces[m.start()].unwrap() == moving_piece && m.end() == cmove.end());

        if moving_piece.piece_type() != PieceType::Pawn {
            write!(w, "{}", moving_piece.to_char().to_uppercase())?;

            if ambiguities.clone().count() != 0 {
                if ambiguities.clone().all(|m| m.start().file() != cmove.start().file()) {
                    write!(w, "{}", cmove.start().file())?;
                } else if ambiguities.all(|m| m.start().rank() != cmove.start().rank()) {
                    write!(w, "{}", cmove.start().rank())?;
                } else {
                    write!(w, "{}{}", cmove.start().file(), cmove.start().rank())?;
                }
            }
        }

        if cmove.is_capture() {
            if moving_piece.piece_type() == PieceType::Pawn {
                write!(w, "{}", cmove.start().file())?;
            }

            write!(w, "x")?;
        }

        write!(w, "{}", cmove.end())?;

        if let Some(promoted) = cmove.promoted() {
            write!(w, "{}", promoted.to_char().to_uppercase())?;
        }

        assert!(board.make_move(cmove));

        if board.in_check() {
            let mut movelist = MoveList::new();
            board.generate_all_moves(&mut movelist);

            if movelist.is_empty() {
                write!(w, "#")?;
            } else {
                write!(w, "+")?;
            }
        }

        board.take_move();

        Ok(())
    }
}

impl Notation for AlgebraicNotation {
    fn write(w: &mut impl Write, cmove: ChessMove, board: &mut Board) -> std::fmt::Result {
        AlgebraicNotation::write(w, cmove, board)
    }
}
