use gtprust::api;

use board;
use randomplay;
use statics;

pub struct ClockGoBot {
    goban: board::Board,
    komi: f32,
}

impl ClockGoBot {
    pub fn new() -> ClockGoBot {
        ClockGoBot {
            goban: board::Board::new(),
            komi: 5.5f32
        }
    }
}

impl api::GoBot for ClockGoBot{

    fn gtp_name(&self) -> String {
        String::from_str(statics::clockgo_name)
    }

    fn gtp_version(&self) -> String {
        String::from_str(statics::clockgo_version)
    }

    fn gtp_clear_board(&mut self) {
        self.goban.clear();
    }

    fn gtp_komi(&mut self, komi: f32) {
        self.komi = komi;
    }

    fn gtp_boardsize(&mut self, size: uint) -> Result<(), api::GTPError> {
        match self.goban.resize(size) {
            true => Ok(()),
            false => Err(api::InvalidBoardSize)
        }
    }

    fn gtp_play(&mut self, move: api::ColouredMove) -> Result<(), api::GTPError> {
        match move {
            api::ColouredMove{player: col, move: api::Pass} => {
                self.goban.pass(match col { api::White => board::White, api::Black => board::Black});
                Ok(())},
            api::ColouredMove{player: col, move: api::Stone(vrtx)} => {
                let (x, y) = vrtx.to_coords();
                match self.goban.play(match col { api::White => board::White, api::Black => board::Black}, x as uint, y as uint) {
                        true => Ok(()),
                        false => Err(api::InvalidMove)
                    }
                },
            api::ColouredMove{player: _, move: api::Resign} => {Ok(())}
        }
    }

    fn gtp_genmove(&mut self, player: api::Colour) -> api::Move {
        match randomplay::genmove(&mut self.goban,
                match player { api::Black => board::Black, api::White => board::White }
            ) {
            board::Put(x, y) => api::Stone(api::Vertex::from_coords(x as u8, y as u8).unwrap()),
            board::Pass => api::Pass
        }
    }

    fn gtp_undo(&mut self) -> Result<(), api::GTPError> {
        if self.goban.undo() {
            Ok(())
        } else {
            Err(api::CannotUndo)
        }
    }

    fn gtp_showboard(&self) -> Result<(uint, Vec<api::Vertex>, Vec<api::Vertex>, uint, uint), api::GTPError> {
        let mut black_stones = Vec::new();
        let mut white_stones = Vec::new();
        let &stones = self.goban.get_board();
        let size = self.goban.get_size();
        for i in range(0, size) {
            for j in range(0, size) {
                match stones[i][j] {
                    board::Stone(board::Black, _) => {
                        black_stones.push(api::Vertex::from_coords((i+1) as u8, (j+1) as u8).unwrap());
                    },
                    board::Stone(board::White, _) => {
                        white_stones.push(api::Vertex::from_coords((i+1) as u8, (j+1) as u8).unwrap());
                    },
                    board::Empty => {}
                }
            }
        }
        let (bd, wd) = self.goban.get_deads();
        Ok((size, black_stones, white_stones, bd, wd))
    }
}
