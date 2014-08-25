extern crate gtprust;

pub mod board;
pub mod gtp;
pub mod statics;

fn main() {
    let mut bot = gtp::ClockGoBot::new();
    gtprust::main_loop(&mut bot);
}
