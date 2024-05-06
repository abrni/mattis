use crate::eval::Eval;
use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("input text is empty")]
    EmptyText,
    #[error("unknown gui command")]
    UnknownCommand,
    #[error("debug must be set to `on` or `off`")]
    DebugInvalid,
    #[error("position must be `startpos` or `fen <fenstring>`")]
    PositionInvalid,
    #[error("go command contained an unknown setting")]
    UnknownGoSetting,
}

#[derive(Debug)]
pub enum GuiMessage {
    Uci,
    Debug(bool),
    Isready,
    // TODO: Setoption {
    //     id: String,
    //     value: String,
    // },
    // TODO: Register
    Ucinewgame,
    Position { pos: Position, moves: Vec<String> },
    Go(Go),
    Stop,
    Ponderhit,
    Quit,
}

#[derive(Debug, Default, Clone)]
pub struct Go {
    pub searchmoves: Vec<String>,
    pub ponder: bool,
    pub wtime: Option<u32>,
    pub btime: Option<u32>,
    pub winc: Option<u32>,
    pub binc: Option<u32>,
    pub movestogo: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u32>,
    pub mate: Option<u32>,
    pub movetime: Option<u32>,
    pub infinite: bool,
}

#[derive(Debug, Clone)]
pub enum Position {
    Startpos,
    Fen(String),
}

impl GuiMessage {
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let (command, rest) = split_whitespace_once(text).ok_or(ParseError::EmptyText)?;

        match command {
            "uci" => Ok(Self::Uci),
            "isready" => Ok(Self::Isready),
            "ucinewgame" => Ok(Self::Ucinewgame),
            "go" => Ok(Self::Go(Go::parse(rest)?)),
            "stop" => Ok(GuiMessage::Stop),
            "ponderhit" => Ok(GuiMessage::Ponderhit),
            "quit" => Ok(GuiMessage::Quit),
            "debug" => match rest.trim() {
                "on" => Ok(Self::Debug(true)),
                "off" => Ok(Self::Debug(false)),
                _ => Err(ParseError::DebugInvalid),
            },
            "position" => {
                let (pos, moves) = parse_position(rest)?;
                Ok(Self::Position { pos, moves })
            }
            _ => Err(ParseError::UnknownCommand),
        }
    }
}

fn parse_position(text: &str) -> Result<(Position, Vec<String>), ParseError> {
    let (pos_kind, rest) = split_whitespace_once(text).ok_or(ParseError::PositionInvalid)?;

    let pos = match pos_kind {
        "startpos" => Position::Startpos,
        "fen" => {
            let split_moves = rest.split_once("moves");
            let fen = if let Some((fen, _)) = split_moves { fen } else { rest };
            Position::Fen(fen.trim().to_owned())
        }
        _ => return Err(ParseError::PositionInvalid),
    };

    let split_moves = rest.split_once("moves");
    let moves = if let Some((_, moves)) = split_moves {
        moves.split_whitespace().map(str::to_owned).collect()
    } else {
        vec![]
    };

    Ok((pos, moves))
}

impl Go {
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let mut go = Go::default();
        let mut parts = text.split_whitespace();

        while let Some(p) = parts.next() {
            match p {
                "ponder" => go.ponder = true,
                "infinite" => go.infinite = true,
                "wtime" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.wtime = Some(t);
                }
                "btime" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.btime = Some(t);
                }
                "winc" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.winc = Some(t);
                }
                "binc" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.binc = Some(t);
                }
                "movestogo" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.movestogo = Some(t);
                }
                "depth" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.depth = Some(t);
                }
                "nodes" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.nodes = Some(t);
                }
                "mate" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.mate = Some(t);
                }
                "movetime" => {
                    let t = parts.next().ok_or(ParseError::UnknownGoSetting)?;
                    let t = t.parse().map_err(|_| ParseError::UnknownGoSetting)?;
                    go.movetime = Some(t);
                }
                "searchmoves" => todo!(),
                _ => return Err(ParseError::UnknownGoSetting),
            }
        }

        Ok(go)
    }
}

#[derive(Debug)]
pub enum EngineMessage {
    Id(Id),
    Uciok,
    Readyok,
    Bestmove { move_: String, ponder: Option<String> },
    // TODO: Copyprotection
    // TODO: Registration
    Info(Info),
    // TODO: Option
}

#[derive(Debug, Default)]
pub struct Info {
    pub depth: Option<u32>,
    pub seldepth: Option<u32>,
    pub time: Option<u32>,
    pub nodes: Option<u32>,
    pub pv: Vec<String>,
    // TODO: multipv
    pub score: Option<Score>,
    pub currmove: Option<String>,
    pub currmovenumber: Option<String>,
    pub hashfull: Option<u32>,
    pub nps: Option<u32>,
    pub tbhits: Option<u32>,
    pub cpuload: Option<u32>,
    pub string: Option<String>,
    pub refutation: Vec<String>,
    pub currline: Vec<String>,
}

#[derive(Debug)]
pub enum Id {
    Name(String),
    Author(String),
}

#[derive(Debug)]
pub struct Score(pub Eval);

impl Display for EngineMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineMessage::Id(id) => match id {
                Id::Name(name) => write!(f, "id name {name}"),
                Id::Author(author) => write!(f, "id author {author}"),
            },
            EngineMessage::Uciok => write!(f, "uciok"),
            EngineMessage::Readyok => write!(f, "readyok"),

            EngineMessage::Info(info) => {
                write!(f, "{info}")
            }
            EngineMessage::Bestmove { move_, ponder } => {
                write!(f, "bestmove {move_}")?;

                if let Some(ponder) = ponder {
                    write!(f, " ponder {ponder}")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ply) = self.0.mate_ply() {
            let moves = ((ply + 1) / 2) as i16;
            let moves = if self.0 > Eval::DRAW { moves } else { -moves };
            write!(f, "mate {}", moves)
        } else {
            write!(f, "cp {}", self.0.inner())
        }
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn write_field<T: Display>(f: &mut std::fmt::Formatter<'_>, name: &str, value: Option<T>) -> std::fmt::Result {
            if let Some(v) = value {
                write!(f, " {name} {v}")?;
            }
            Ok(())
        }

        write!(f, "info")?;
        write_field(f, "depth", self.depth)?;
        write_field(f, "seldepth", self.seldepth)?;
        write_field(f, "time", self.time)?;
        write_field(f, "nodes", self.nodes)?;
        write_field(f, "score", self.score.as_ref())?;
        write_field(f, "currmove", self.currmove.as_ref())?;
        write_field(f, "currmovenumber", self.currmovenumber.as_ref())?;
        write_field(f, "hashfull", self.hashfull)?;
        write_field(f, "nps", self.nps)?;
        write_field(f, "tbhits", self.tbhits)?;
        write_field(f, "cpuload", self.cpuload)?;

        if !self.pv.is_empty() {
            write!(f, " pv")?;

            for m in &self.pv {
                write!(f, " {m}")?;
            }
        }

        write_field(f, "string", self.string.as_ref())?;

        Ok(())
    }
}

fn split_whitespace_once(text: &str) -> Option<(&str, &str)> {
    let (first, rest) = text.split_once(char::is_whitespace)?;
    Some((first, rest.trim_start()))
}
