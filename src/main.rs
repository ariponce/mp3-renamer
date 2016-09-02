extern crate getopts;
extern crate id3;
extern crate metaflac;
extern crate term;

use std::{env, fs};
use std::path::{Path, PathBuf};
use id3::Tag;
use getopts::Options;
use metaflac::Tag as FlacTag;
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
    opts.optopt("f", "format", "Format to use when renaming files", "%n - %t");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

	let mut format = String::from("%n - %t");
	if matches.opt_present("f") {
		format = matches.opt_str("f").unwrap();
	}

    let path = Path::new(&args[1]);

    if ! path.exists() {
        println!("Error: File doesn't exist");
        return;
    }

    let needed_tags = parse_format(&format);

    if path.is_dir() {
        parse_dir(path, &matches, &needed_tags);
    } else {
        match parse_file(path, &needed_tags) {
            Ok(_) => println!("Done"),
            Err(e) => println!("Error: {}", e.to_string()),
        }
    }
}

fn parse_dir(path: &Path, matches: &getopts::Matches, needed_tags: &Vec<&str>) {
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
            parse_dir(path.as_path(), &matches, &needed_tags);
        } else {
            match parse_file(path.as_path(), &needed_tags) {
                Ok(_) => {
                    t.fg(term::color::GREEN).unwrap();
                    write!(t, "[OK] ").unwrap();
                    t.reset().unwrap();
                    write!(t, "{}\n", path.file_name().unwrap().to_str().unwrap()).unwrap();
                },
                Err(e) => {
                    if e != "Not a music file" {
                        t.fg(term::color::RED).unwrap();
                        write!(t, "[Er] ").unwrap();
                        t.reset().unwrap();
                        write!(t, "{}: {}\n", path.file_name().unwrap().to_str().unwrap(), e).unwrap();
                    }
                }
            }
        }
    }
}

fn parse_file(path: &Path, needed_tags: &Vec<&str>) -> Result<(), String> {
    match path.extension().unwrap().to_str().unwrap() {
        "mp3" => parse_mp3(path, needed_tags),
        "flac" => parse_flac(path, needed_tags),
        _ => return Err("Not a music file".to_string())
    }
}

/// Get ID3 tags from a MP3 file and rename it based on them
fn parse_mp3(path: &Path, needed_tags: &Vec<&str>) -> Result<(), String> {
    let extension = path.extension().unwrap();
	let tag = match Tag::read_from_path(path) {
		Ok(v) => { v },
		Err(e) => { return Err(e.to_string()) }
	};

    let mut new_path = PathBuf::from(path);
    let mut new_filename: String = String::from("");

	for t in needed_tags {
		match *t {
			"track" => {
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
				new_filename = new_filename + track_number.as_str();
			},
			"title" => {
				let mut title = match tag.title() {
                    Some(v) => { v },
					None => return Err("No title tag found".to_string()),
				};
				// Sometimes tiles contains slashes, which cannot be used
				// on filenames, so we need to replace them
                if title.contains("/") {
                    let formatted_title = title.replace('/', "-");
                    new_filename = new_filename + &formatted_title;
                } else {
                    new_filename = new_filename + title;
                }

			},
			_ => {
				new_filename = new_filename + " " + t + " ";
			}
		}
	}

	new_filename = new_filename + "." + extension.to_str().unwrap();
    new_path.set_file_name(new_filename);

    match fs::rename(path, new_path) {
        Ok(_) => (),
        Err(e) => return Err(e.to_string()),
    }

    Ok(())
}

/// Get vorbis comments from a FLAC file and rename it based on them
fn parse_flac(path: &Path, needed_tags: &Vec<&str>) -> Result<(), String> {
    let extension = path.extension().unwrap();
	let tag = match FlacTag::read_from_path(path) {
		Ok(v) => { v },
		Err(e) => { return Err(e.to_string()) }
	};

	let tags = tag.vorbis_comments().unwrap();

    let mut new_path = PathBuf::from(path);
    let mut new_filename: String = String::from("");

    for t in needed_tags {
        match *t {
            "track" => {
                let mut track_number: String = String::from("");

                let track = match tags.track() {
                    Some(v) => v.to_string(),
                    None => return Err("No track tag found".to_string(),)
                };
                // Add a leading zero for track between 1-9
                if track.len() < 2 {
                    track_number = "0".to_owned();
                }


                track_number.push_str(&track);
                new_filename = new_filename + track_number.as_str();
            },
            "title" => {
                let title = match tags.title() {
                    Some(v) => v[0].as_str(),
                    None => return Err("No title tag found".to_string()),
                };
                new_filename = new_filename + title;
            },
            "artist" => {
                let artist = match tags.artist() {
                    Some(v) => v[0].as_str(),
                    None => return Err("No artist tag found".to_string()),
                };
                new_filename = new_filename + artist;
            },
            "year" => {
                let year = match tags.get("DATE") {
                    Some(v) => v[0].as_str(),
                    None => return Err("No year tag found".to_string()),
                };
                new_filename = new_filename + year;
            },
            "album" => {
                let album = match tags.album() {
                    Some(v) => v[0].as_str(),
                    None => return Err("No album tag found".to_string()),
                };
                new_filename = new_filename + album;
            },
			_ => {
				new_filename = new_filename + " " + t + " ";
			}
        }
    }

	new_filename = new_filename + "." + extension.to_str().unwrap();
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
    let (year, album): (String, String);

    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() && (path.extension().unwrap() == "mp3" || path.extension().unwrap() == "flac")  {
            file = path;
            break;
        }
    }

    if file.exists() {
        match file.extension().unwrap().to_str().unwrap() {
            "mp3" => {
                let tag = match Tag::read_from_path(file) {
                    Ok(v) => { v },
                    Err(e) => { return Err(e.to_string()) }
                };

                 match tag.year() {
                    Some(v) => year = v.to_string(),
                    None => return Err("No year tag found".to_string()),
                };

                 match tag.album() {
                    Some(v) => album = v.to_string(),
                    None => return Err("No album tag found".to_string()),
                };
            },
            "flac" => {
                let tag = match FlacTag::read_from_path(file) {
                    Ok(v) => { v },
                    Err(e) => { return Err(e.to_string()) }
                };

                let tags = tag.vorbis_comments().unwrap();

                match tags.get("DATE") {
                    Some(v) => year = v[0].to_string(),
                    None => return Err("No year tag found".to_string()),
                };

                match tags.album() {
                    Some(v) => album =  v[0].to_string(),
                    None => return Err("No album tag found".to_string()),
                };
            },
            _ => { return Err("Cannot renamed dir".to_string()) }
        }

        new_path.set_file_name(year + " - " + &album);

        match fs::rename(path, &new_path) {
            Ok(_) => () ,
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(new_path)
}

fn parse_format(format: &str) -> Vec<&str> {
	let v: Vec<&str> = format.split(' ').collect();
	let mut result: Vec<&str> = vec!();

	for arg in v.iter() {
		if arg.is_empty() {
			continue;
		} else if arg.ends_with("n") {
			result.push("track");
		} else if arg.ends_with("t") {
			result.push("title");
		} else if arg.ends_with("a") {
			result.push("artist");
		} else if arg.ends_with("y") {
			result.push("year");
		} else if arg.ends_with("b") {
			result.push("album");
		} else {
			result.push(arg);
		}
	}

	result
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}
