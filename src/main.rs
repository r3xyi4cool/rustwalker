use std::{
    collections::HashMap, 
    fs::{self},                     
    io::{self, ErrorKind},  
    path::PathBuf,
    time::{Instant, SystemTime}, 
};
use serde::{Deserialize, Serialize};  
use walkdir::WalkDir;       

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileMetadata {
    last_modified: Option<SystemTime>,
    file_size: u64
}

type CacheFile = HashMap<PathBuf, FileMetadata>;

fn cache_save(cache: &CacheFile, path: &str) -> io::Result<()> {
    let converter = serde_json::to_string(cache)
        .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
    fs::write(path, converter)?;
    Ok(())
}

fn cache_load(path: &str) -> io::Result<CacheFile> {
    let data = fs::read_to_string(path)?;
    let cache: CacheFile = serde_json::from_str(&data)
        .map_err(|e| io::Error::new(ErrorKind::Other, e))?;
    Ok(cache)
}

fn walk(path: &str, min_size: u64, cache: &mut CacheFile) {
    let mut files_scanned = 0;
    let mut permission_denied = 0;
    let mut other_errors = 0;
    let mut found_files: Vec<(PathBuf, u64)> = Vec::new();
 
    for entry in WalkDir::new(path) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                match e.io_error() {
                    Some(io_err) => match io_err.kind() {
                        io::ErrorKind::PermissionDenied => {
                            permission_denied += 1;
                        }
                        _ => {
                            other_errors += 1;
                        }
                    },
                    None => {
                        other_errors += 1;
                    }
                }
                continue; 
            },
        };
        
        let bufpath = entry.path().to_path_buf();

        if !entry.file_type().is_file() {
            continue;
        }
        
        files_scanned += 1;
        
        let use_cache = match cache.get(&bufpath) {
            Some(meta) => {
                match entry.metadata() {
                    Ok(m) => {
                        let last_modi = m.modified().ok();
                        let sz = m.len();
                        meta.last_modified == last_modi && meta.file_size == sz
                    }
                    Err(_) => false,
                }
            }
            None => false,
        };
        
        let size = if use_cache {
            cache.get(&bufpath).unwrap().file_size
        } else {
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    if let Some(io_err) = e.io_error() {
                        match io_err.kind() {
                            std::io::ErrorKind::PermissionDenied => {
                                permission_denied += 1;
                            }
                            _ => {
                                other_errors += 1;
                            }
                        }
                    } else {
                        other_errors += 1;
                    }
                    continue;
                }
            };
            
            let sz = metadata.len();
            let meta_file = FileMetadata {
                last_modified: metadata.modified().ok(),
                file_size: sz,
            };
            cache.insert(bufpath.clone(), meta_file);
            sz
        };
        
        if size >= min_size {
            found_files.push((bufpath, size));
        }
    }

    if found_files.is_empty() {
        println!("No files larger than {} bytes found.", min_size);
    } else {
        println!("Found {} large files out of {} scanned:", found_files.len(), files_scanned);
        for (path, size) in &found_files {
            println!("  - {} ({} bytes)", path.display(), size);
        }
    }

    println!("===== Final Scan Statistics =====");
    println!("Files Scanned:                {}", files_scanned);
    println!("Permission Denied:            {}", permission_denied);
    println!("Other Errors:                 {}", other_errors);
    println!("Files Found:                  {}", found_files.len());
}

fn parse_size_input(input: &str) -> Result<u64, String> {
    let input = input.trim();
    let num_part: String = input.chars().take_while(|c| c.is_ascii_digit()).collect();
    let unit_part: String = input.chars().skip_while(|c| c.is_ascii_digit()).collect();
    let unit_part = unit_part.trim();

    if num_part.is_empty() {
        return Err("No number found in input".to_string());
    }

    let number: u64 = num_part.parse()
        .map_err(|_| "Invalid number format".to_string())?;

    let multiplier = match unit_part.to_lowercase().as_str() {
        "gb" => 1024 * 1024 * 1024,
        "mb" => 1024 * 1024,
        "kb" => 1024,
        "" => 1,
        _ => return Err(format!("Invalid unit: '{}'", unit_part)),
    };

    Ok(number * multiplier)
}

fn main() {
    let cache_file_path = "file_cache.json";
    let mut cache = cache_load(cache_file_path).unwrap_or_else(|_| HashMap::new());
    
    println!("Enter the directory path:");
    let mut path = String::new();
    io::stdin().read_line(&mut path).expect("Failed to read the path");
    let path = path.trim();
    
    if !std::path::Path::new(path).exists() {
        println!("Error: Directory '{}' not found", path);
        return;
    }

    println!("Enter the size (e.g., 10GB, 20MB, 500KB): ");
    let mut input_size = String::new();
    io::stdin().read_line(&mut input_size).expect("Error reading the size");
    
    let size = match parse_size_input(&input_size) {
        Ok(s) => s,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    let start = Instant::now();
    walk(path, size, &mut cache);
    
    if let Err(e) = cache_save(&cache, cache_file_path) {
        eprintln!("Warning: Failed to save cache: {}", e);
    }
       
    let duration = start.elapsed();
    println!("Time taken: {:.2?}", duration);
}