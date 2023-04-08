use crate::args::get_args_or_exit;
use crate::cpr::cpr;

mod args;
mod cpr;
mod types;
mod util;

fn main() {
    cpr(get_args_or_exit())
}
