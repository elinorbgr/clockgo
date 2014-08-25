//! A bot playing randomly, but still following the rules.

use std::rand::{task_rng, Rng};

use board;

pub fn genmove(goban: &mut board::Board, player: board::Colour) -> board::Vertex {
    let size = goban.get_size();
    let mut rng = task_rng();
    let mut i = 0u;
    // try at most 10 random moves
    while i < 10 {
        let (x, y) = (rng.gen_range(1u, size+1), rng.gen_range(1u, size+1));
        if goban.play(player, x, y){
            return board::Put(x,y);
        }
        i = i+1;
    }
    // if we reach this point, random failed, we go for a more deterministic
    // approach
    for x in range(1u, size+1) {
        for y in range(1u, size+1) {
            if goban.play(player, x, y){
                return board::Put(x,y);
            }
        }
    }
    // can play nothing ?
    board::Pass
}
