extern crate getopts;
extern crate id3;

use std::{env, fs};
use std::path::{Path, PathBuf};
use getopts::Options;
use id3::Tag;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        println!("No files were provided");
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
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        match parse_file(path.as_path()) {
            Ok(_) => println!("Processed {:?}", path.file_name().unwrap()),
            Err(e) => println!("Failed to process {:?}: {}", path.file_name().unwrap(), e)
        }
    }
}

fn parse_file(path: &Path) -> Result<(), String> {
    let extension = path.extension().unwrap();
    if extension != "mp3" {
        return Err("Not an mp3 file".to_string());
    }
    let tag = Tag::read_from_path(path).unwrap();
    let mut new_path = PathBuf::from(path);
    let song = tag.title().unwrap();
    let mut track_number: String = String::from("");
    let track = tag.track().unwrap().to_string();
    if !track.starts_with("0") || !track.len() >= 2 {
        let mut track_number:String = "0".to_owned();
    }
    track_number.push_str(&track);
    let new_filename = track_number + " - " + song + "." + extension.to_str().unwrap();
    new_path.set_file_name(new_filename);

    match fs::rename(path, new_path) {
        Ok(_) => (),
        Err(e) => return Err(e.to_string()),
    }

    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}
