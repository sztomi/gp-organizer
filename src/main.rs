#![feature(plugin)]
#![plugin(docopt_macros)]
#[macro_use]
extern crate serde_derive;
extern crate glob;
extern crate docopt;
extern crate exif;

use std::path::Path;
use std::fs;
use std::io::BufReader;

use glob::glob_with;
use glob::MatchOptions;
use exif::{DateTime, Value, tag};

docopt!(Args derive Debug, "
Organizes your files into the structure that Google Photos uses
in Google Drive.

Usage:
  gp-organizer <input-dir> <output-dir>
  gp-organizer (-h | --help)
  gp-organizer --version

Options:
  -h --help     Show this screen.
  --version     Show version.
");

fn check_args(args: &Args) -> bool {
    if !Path::new(&args.arg_input_dir).exists() {
        println!("Input directory '{}' does not exist.", &args.arg_input_dir);
        return false
    }
    if !Path::new(&args.arg_output_dir).exists() {
        println!("Output directory '{}' does not exist.", &args.arg_output_dir);
        return false
    }
    true
}

fn copy_files(input_dir: &String, output_dir: &String) {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    for entry in glob_with(&*format!("{}/**/*.jpg", input_dir), &options).unwrap() {
        if let Ok(path) = entry {
            let file = fs::File::open(path.clone()).unwrap();
            match exif::Reader::new(&mut BufReader::new(&file)) {
                Ok(reader) => {
                    if let Some(field) = reader.get_field(tag::DateTime, false) {
                        match field.value {
                            Value::Ascii(ref vec) if !vec.is_empty() => {
                                if let Ok(datetime) = DateTime::from_ascii(vec[0]) {
                                    let out = &format!("{}/{}/{}/{}",
                                                                    output_dir, datetime.year,
                                                                    datetime.month, path.clone()
                                                                                    .file_name()
                                                                                    .unwrap()
                                                                                    .to_str()
                                                                                    .unwrap());
                                    let out_path = Path::new(out);
                                    if !out_path.parent().unwrap().exists() {
                                        println!("{} does not exist. Creating.", out_path.parent().unwrap().to_str().unwrap());
                                        fs::create_dir_all(out_path.parent().unwrap());
                                    }
                                    println!("{}", out_path.file_name().unwrap().to_str().unwrap());
                                    fs::copy(path, out_path);
                                }
                            },
                            _ => {},
                        }
                    }
                },
                Err(e) => {
                    fs::copy(&path, &*format!("{}/no_exif/{}", 
                                              output_dir, 
                                              path.clone().file_name().unwrap().to_str().unwrap()));
                }
            }
        }
    }
}

fn main() {
    let args: Args = Args::docopt().deserialize()
                           .unwrap_or_else(|e| e.exit());

    if !check_args(&args) {
        panic!("Invalid arguments");
    }

    copy_files(&args.arg_input_dir, &args.arg_output_dir);
}
