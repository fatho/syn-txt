//! `musicc` - pronounced *music-c*, is the compiler for syntxt files to wav files.

use std::io;
use std::path::PathBuf;

use simple_logger;
use structopt::StructOpt;

use syn_txt::musicc;

#[derive(Debug, StructOpt)]
#[structopt(name = "musicc", about = "Compiling code into music")]
struct Opt {
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// The source code of the music.
    #[structopt(parse(from_os_str))]
    source: PathBuf,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let level = if opt.verbose { log::Level::Trace } else { log::Level::Info };
    simple_logger::init_with_level(level).unwrap();

    let source = std::fs::read_to_string(&opt.source)?;
    let roll = musicc::eval::eval(&opt.source.to_string_lossy(), &source)?;
    musicc::translate::play(roll)
}
