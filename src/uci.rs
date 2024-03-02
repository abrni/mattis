use std::fmt::Display;

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
    pub fn parse(text: &str) -> Result<Self, ()> {
        let mut parts = text.trim().split_ascii_whitespace();

        match parts.next().ok_or(())? {
            "uci" => Ok(Self::Uci),
            "debug" => match parts.next().ok_or(())? {
                "on" => Ok(Self::Debug(true)),
                "off" => Ok(Self::Debug(false)),
                _ => Err(()),
            },
            "isready" => Ok(Self::Isready),
            "ucinewgame" => Ok(Self::Ucinewgame),
            "position" => {
                let pos = match parts.next().ok_or(())? {
                    "startpos" => Position::Startpos,
                    "fen" => {
                        let mut fen = parts.next().unwrap().to_owned();
                        fen.push(' ');
                        fen.push_str(parts.next().unwrap());
                        fen.push(' ');
                        fen.push_str(parts.next().unwrap());
                        fen.push(' ');
                        fen.push_str(parts.next().unwrap());
                        fen.push(' ');
                        fen.push_str(parts.next().unwrap());
                        fen.push(' ');
                        fen.push_str(parts.next().unwrap());
                        Position::Fen(fen)
                    }
                    _ => return Err(()),
                };

                let moves = if let Some("moves") = parts.next() {
                    parts.map(str::to_owned).collect()
                } else {
                    vec![]
                };

                Ok(Self::Position { pos, moves })
            }
            "go" => Ok(Self::Go(Go::parse(&mut parts)?)),
            "stop" => Ok(GuiMessage::Stop),
            "ponderhit" => Ok(GuiMessage::Ponderhit),
            "quit" => Ok(GuiMessage::Quit),
            _ => Err(()),
        }
    }
}

impl Go {
    pub fn parse<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<Self, ()> {
        let mut go = Go::default();

        while let Some(p) = parts.next() {
            match p {
                "ponder" => go.ponder = true,
                "infinite" => go.infinite = true,
                "wtime" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.wtime = Some(t);
                }
                "btime" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.btime = Some(t);
                }
                "winc" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.winc = Some(t);
                }
                "binc" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.binc = Some(t);
                }
                "movestogo" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.movestogo = Some(t);
                }
                "depth" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.depth = Some(t);
                }
                "nodes" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.nodes = Some(t);
                }
                "mate" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.mate = Some(t);
                }
                "movetime" => {
                    let t = parts.next().ok_or(())?;
                    let t = t.parse().map_err(|_| ())?;
                    go.movetime = Some(t);
                }
                "searchmoves" => todo!(),
                _ => return Err(()),
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
pub enum Score {
    Cp(i32),
    Mate(i32),
    // TODO: Lowerbound, Upperbound
}

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
        match self {
            Self::Cp(cp) => write!(f, "cp {cp}"),
            Self::Mate(mate) => write!(f, "mate {mate}"),
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
