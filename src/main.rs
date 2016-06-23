extern crate getopts;
extern crate id3;
extern crate term;

use std::{env, fs};
use std::path::{Path, PathBuf};
use getopts::Options;
use id3::Tag;
use std::io::prelude::*;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        println!("No files were provided");
        return;
    }

    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print help menu");
    opts.optflag("", "rename-dir", "Also rename the directory where files are located");
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
        parse_dir(path, &matches);
    } else {
        match parse_file(path) {
            Ok(_) => println!("Done"),
            Err(e) => println!("Error: {}", e.to_string()),
        }
    }
}

fn parse_dir(path: &Path, matches: &getopts::Matches) {
    let mut t = term::stdout().unwrap();
    let mut new_path:PathBuf = path.to_owned();

    if matches.opt_present("rename-dir") {
        match rename_dir(path) {
            Ok(v) => {
                new_path = PathBuf::new();
                new_path.push(&v);
            },
            Err(e) => println!("Failed to rename dir {}", e.to_string()),
        };
    }

    for entry in fs::read_dir(new_path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            parse_dir(path.as_path(), &matches);
        } else {
            match parse_file(path.as_path()) {
                Ok(_) => {
                    t.fg(term::color::GREEN).unwrap();
                    write!(t, "[OK] ").unwrap();
                    t.reset().unwrap();
                    write!(t, "{}\n", path.file_name().unwrap().to_str().unwrap()).unwrap();
                },
                Err(e) => {
                    t.fg(term::color::RED).unwrap();
                    write!(t, "[Er] ").unwrap();
                    t.reset().unwrap();
                    write!(t, "{}: {}\n", path.file_name().unwrap().to_str().unwrap(), e).unwrap();
                }
            }
        }
    }
}

fn parse_file(path: &Path) -> Result<(), String> {
    let extension = path.extension().unwrap();
    if extension != "mp3" {
        return Err("Not an mp3 file".to_string());
    }

	let tag = match Tag::read_from_path(path) {
		Ok(v) => { v },
		Err(e) => { return Err(e.to_string()) }
	};

    let mut new_path = PathBuf::from(path);

    let song = match tag.title() {
        Some(v) => v,
        None => return Err("No title tag found".to_string()),
    };
    let mut track_number: String = String::from("");

    let track = match tag.track() {
        Some(v) => v.to_string(),
        None => return Err("No track tag found".to_string()),
    };
    // Add a leading zero for track between 1-9
    if track.len() < 2 {
        track_number = "0".to_owned();
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

/// TODO: Handle albums with multiple folders
fn rename_dir(path: &Path) -> Result<PathBuf, String> {
    let mut file = PathBuf::new();
    let mut new_path = PathBuf::from(path);
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() && path.extension().unwrap() == "mp3" {
            file = path;
            break;
        }
    }

    if file.exists() && file.extension().unwrap() == "mp3" {
        let tag = match Tag::read_from_path(file) {
            Ok(v) => { v },
            Err(e) => { return Err(e.to_string()) }
        };

        let year = match tag.year() {
            Some(v) => v.to_string(),
            None => return Err("No year tag found".to_string()),
        };

        let album = match tag.album() {
            Some(v) => v,
            None => return Err("No album tag found".to_string()),
        };
        new_path.set_file_name(year + " - " + album);

        match fs::rename(path, &new_path) {
            Ok(_) => () ,
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(new_path)
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}
