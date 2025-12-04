use dotenv::dotenv;
use family_graph::run;
use std::{env, path::Path};

fn main() {
    dotenv().ok();
    match dotenv() {
        Ok(path) => println!("Loaded .env from: {:?}", path),
        Err(e) => println!("Could not load .env file: {}", e),
    }
    println!(
        "Current working directory: {:?}",
        env::current_dir().unwrap()
    );
    let path = "./Wistoft familien.xls";

    let static_dir = Path::new("./static");
    if static_dir.exists() {
        println!("✓ Static directory exists");
        // List contents of static directory
        if let Ok(entries) = std::fs::read_dir(static_dir) {
            println!("Contents of static directory:");
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  - {}", entry.file_name().to_string_lossy());
                }
            }
        }
    } else {
        println!("✗ Static directory does NOT exist");
    }

    let full_path = Path::new(&path);
    println!("Looking for file: {}", full_path.display());
    if full_path.exists() {
        println!("File exists at the specified path");
        run(full_path);
    } else {
        println!("File does NOT exist at the specified path");

        // If it's a relative path, show what the absolute path would be
        if let Ok(absolute_path) = Path::new(&path).canonicalize() {
            println!("Absolute path would be: {}", absolute_path.display());
        } else {
            println!("Cannot determine absolute path (file doesn't exist)");
            if let Ok(entries) = std::fs::read_dir("./static") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let filename = entry.file_name().to_string_lossy().to_lowercase();
                        if filename == "wistoft_familien.xls" {
                            println!(
                                "Found file with different case: {}",
                                entry.file_name().to_string_lossy()
                            );
                        }
                    }
                }
            }
        }
    }
}
