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
    Position {
        pos: Position,
        moves: Vec<String>,
    },
    Go {
        searchmoves: Vec<String>,
        ponder: bool,
        wtime: Option<u32>,
        btime: Option<u32>,
        winc: Option<u32>,
        binc: Option<u32>,
        movestogo: Option<u32>,
        depth: Option<u32>,
        nodes: Option<u32>,
        mate: Option<u32>,
        movetime: Option<u32>,
        infinite: bool,
    },
    Stop,
    Ponderhit,
    Quit,
}

#[derive(Debug)]
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
                    f => {
                        let mut fen = f.to_owned();
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
                };

                let moves = if let Some("moves") = parts.next() {
                    parts.map(str::to_owned).collect()
                } else {
                    vec![]
                };

                Ok(Self::Position { pos, moves })
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum EngineMessage {
    Id(Id),
    Uciok,
    Readyok,
    Bestmove {
        move_: String,
        ponder: Option<String>,
    },
    // TODO: Copyprotection
    // TODO: Registration
    Info {
        depth: Option<u32>,
        seldepth: Option<u32>,
        time: Option<u32>,
        nodes: Option<u32>,
        pv: Vec<String>,
        // TODO: multipv
        score: Option<Score>,
        currmove: Option<String>,
        currmovenumber: Option<String>,
        hashfull: Option<u32>,
        nps: Option<u32>,
        tbhits: Option<u32>,
        cpuload: Option<u32>,
        string: Option<String>,
        refutation: Vec<String>,
        currline: Vec<String>,
    },
    // TODO: Option
}

#[derive(Debug)]
pub enum Id {
    Name(String),
    Author(String),
}

#[derive(Debug)]
pub enum Score {
    Cp(u32),
    Mate(u32),
    // TODO: Lowerbound, Upperbound
}
