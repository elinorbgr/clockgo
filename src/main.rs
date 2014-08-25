extern crate gtprust;

pub mod board;
pub mod gtp;
pub mod statics;

pub mod randomplay;

fn main() {
    let mut bot = gtp::ClockGoBot::new();
    gtprust::main_loop(&mut bot);
}
