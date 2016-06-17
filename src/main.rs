extern crate getopts;
extern crate id3;

use std::{env, fs};
use std::path::{Path, PathBuf};
use getopts::Options;
use id3::Tag;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        println!("Error: Not enough arguments");
        return;
    }

    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let path = Path::new(&args[1]);

    if ! path.exists() {
        println!("Error: File doesn't exist");
        return;
    }

    if path.is_dir() {
        parse_dir(path);
    } else {
        match parse_file(path) {
            Ok(_) => println!("Done"),
            Err(e) => println!("Error: {}", e.to_string()),
        }
    }

}

fn parse_dir(path: &Path) {
    // TODO: implement this
}

fn parse_file(path: &Path) -> Result<(), std::io::Error> {
    let tag = Tag::read_from_path(path).unwrap();
    let mut new_path = PathBuf::from(path);
    let song = tag.title().unwrap();
    let track = tag.track().unwrap();
    let mut track_number:String = "0".to_owned();
    track_number.push_str(&track.to_string());
    let new_filename = track_number + " - " + song;
    new_path.set_file_name(new_filename);

    match fs::rename(path, new_path) {
        Ok(_) => (),
        Err(e) => return Err(e),
    }

    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}
