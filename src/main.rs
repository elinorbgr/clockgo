extern crate gtprust;

mod board;
mod gtp;
mod statics;

fn main() {
    let mut bot = gtp::ClockGoBot::new();
    gtprust::main_loop(&mut bot);
}
