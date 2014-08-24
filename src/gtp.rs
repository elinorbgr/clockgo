use gtprust::api;
use board;

use statics;

pub struct ClockGoBot {
    goban: board::Board
}

impl ClockGoBot {
    pub fn new() -> ClockGoBot {
        ClockGoBot {
            goban: board::Board::new()
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
        fail!("Not implemented.")
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
            api::ColouredMove{player: col, move: api::Resign} => {Ok(())}
        }
    }

    fn gtp_genmove(&mut self, player: api::Colour) -> api::Move {
        fail!("Not implemented.")
    }

    fn gtp_undo(&mut self) -> Result<(), api::GTPError> {
        if self.goban.undo() {
            Ok(())
        } else {
            Err(api::CannotUndo)
        }
    }

    fn gtp_showboard(&self) -> Result<(uint, Vec<api::Vertex>, Vec<api::Vertex>, uint, uint), api::GTPError> {
        let (size, bst, wst, bd, wd) = self.goban.list_stones();
        let mut black_st = Vec::with_capacity(bst.len());
        for &(x,y) in bst.iter() {
            black_st.push(api::Vertex::from_coords((x+1) as u8, (y+1) as u8).unwrap());
        }
        let mut white_st = Vec::with_capacity(wst.len());
        for &(x,y) in wst.iter() {
            white_st.push(api::Vertex::from_coords((x+1) as u8, (y+1) as u8).unwrap());
        }
        Ok((size, black_st, white_st, bd, wd))
    }
}
