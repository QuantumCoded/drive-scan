use clap::{App, Arg};
use std::collections::HashMap;
use std::fs;
use std::io::stdout;
use std::io::Write;
use std::path::Path;
use std::string::String;
use walkdir::WalkDir;

fn size_string(bytes: u64) -> String {
    format!(
        "{},{:.2},{:.2},{:.2}",
        bytes as f64 / (1024 as u64).pow(0) as f64,
        bytes as f64 / (1024 as u64).pow(1) as f64,
        bytes as f64 / (1024 as u64).pow(2) as f64,
        bytes as f64 / (1024 as u64).pow(3) as f64
    )
}

fn size_string_display(bytes: u64) -> String {
    format!(
        "{} B, {:.2} KB, {:.2} MB, {:.2} GB",
        bytes as f64 / (1024 as u64).pow(0) as f64,
        bytes as f64 / (1024 as u64).pow(1) as f64,
        bytes as f64 / (1024 as u64).pow(2) as f64,
        bytes as f64 / (1024 as u64).pow(3) as f64
    )
}

fn main() {
    let mut stdout = stdout();

    let matches = App::new("Drive Scan")
        .version("0.1.0")
        .author("QuantumCoded <bfields32@student.cccs.edu>")
        .about("A program to document the size of files and folders")
        .arg(
            Arg::with_name("PATH")
                .help("The path to scan")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .help("The file to output to")
                .short("o")
                .long("last"),
        )
        .get_matches();

    let path = Path::new(matches.value_of("PATH").unwrap());
    let output = Path::new(matches.value_of("output").unwrap_or("output.csv"));

    if !path.exists() {
        stdout
            .write("Could not find path specified\r\n".as_bytes())
            .unwrap();
        stdout.flush().unwrap();
    }

    if output.exists() && output.is_file() {
        fs::remove_file(output).unwrap();
    }

    let mut output_file = fs::File::create(output).unwrap();
    let mut dir_size_map: HashMap<String, u64> = HashMap::new();

    // Report the byte count of all files
    for dir_entry in WalkDir::new(path).follow_links(false) {
        if let Ok(entry) = dir_entry {
            let entry_path = entry.path();

            if entry_path.is_file() {
                let file_size = fs::File::open(entry_path)
                    .unwrap()
                    .metadata()
                    .unwrap()
                    .len();

                stdout
                    .write(
                        format!(
                            "{} {}\r\n",
                            entry_path.to_str().unwrap(),
                            size_string_display(file_size)
                        )
                        .as_bytes(),
                    )
                    .unwrap();
                stdout.flush().unwrap();

                output_file
                    .write(
                        format!("F,{},{}\r\n", entry_path.display(), size_string(file_size))
                            .as_bytes(),
                    )
                    .unwrap();

                let keys: Vec<_> = dir_size_map.keys().map(|k| k.clone()).collect();

                // Add file size to all parent folders
                for dir in keys {
                    if entry_path
                        .to_str()
                        .unwrap()
                        .starts_with(&format!("{}\\", dir))
                    {
                        let dir_size = dir_size_map.get_mut(&dir).unwrap();
                        *dir_size += file_size;
                    }
                }

                continue;
            }

            if entry_path.is_dir() {
                dir_size_map.insert(entry_path.to_str().unwrap().to_string(), 0);
            }
        }
    }

    // Report the byte count of all folders
    for (dir_name, size) in dir_size_map.drain() {
        stdout
            .write(format!("{} {}\r\n", dir_name, size_string_display(size)).as_bytes())
            .unwrap();
        stdout.flush().unwrap();

        output_file
            .write(format!("D,{},{}\r\n", dir_name, size_string(size)).as_bytes())
            .unwrap();
    }

    output_file.flush().unwrap();
}
