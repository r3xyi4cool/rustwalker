use walkdir::WalkDir;
use std::fs::{self, File};
use std::{io::{self, BufReader}, path::Path};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json;
use tempfile::NamedTempFile;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Fileinfo{
    path:String,
    size:u64,
    date_modified:u64,
}
type CacheData = Vec<Fileinfo>;

fn get_file_time(path: &Path) -> Result<u64, std::io::Error> {
    let m = fs::metadata(path)?; 
    let modified: SystemTime = m.modified()?; 
    Ok(modified.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
}

fn load_cache(cache_file: &str) -> Result<CacheData,Box<dyn std::error::Error>>{
    let file = File::open(cache_file)?;
    let r = BufReader::new(file);
    Ok(serde_json :: from_reader(r)?)
}

fn save_cache(cache_file: &str, files: &[Fileinfo]) -> io::Result<()> {
    let mut temp = NamedTempFile::new()?;
    serde_json::to_writer_pretty(&mut temp, files)?;
    temp.persist(cache_file)
        .map_err(|e| e.error)?; 
    println!("Cache updated with {} files", files.len());
    Ok(())
}

fn is_file_uptodate(cached_file : &Fileinfo) -> bool{
    let path = Path::new(&cached_file.path);

    match get_file_time(path) {
        Ok(modifie) => modifie == cached_file.date_modified,
        Err(_) => false,
    }
} 

fn walk(path:&str,search_file:&str,cache_file: &str) {
    let entries: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .collect();

    let cache = load_cache(cache_file).unwrap_or_else(|e|{
        println!("No cache found ({}) creating a new vector",e);
        Vec::new()
    });

    let mut cachemap: std::collections::HashMap<String,Fileinfo> = cache.into_iter().map(|f| (f.path.clone(),f)).collect();
    let updated_cache_entries = Arc::new(Mutex::new(Vec::<Fileinfo>::new()));
    let current_matches = Arc::new(Mutex::new(Vec::<Fileinfo>::new()));
    let files_scanned = Arc::new(Mutex::new(0u32));
    let permission_denied = Arc::new(Mutex::new(0u32));
    let other_errors = Arc::new(Mutex::new(0u32));
    
    print!("Scanning .......... ");

    entries.par_iter().for_each(|entry|{
        {
            let mut count = files_scanned.lock().unwrap();
            *count += 1;
        }
        let pathz = entry.path().display().to_string();
        if let Some(cached_file) = cachemap.get(&pathz){
            if is_file_uptodate(cached_file) {
                {
                    let mut cache_entries = updated_cache_entries.lock().unwrap(); 
                    cache_entries.push(cached_file.clone());
                }

                if entry.file_name().to_string_lossy() == search_file{
                    let mut matches = current_matches.lock().unwrap();
                    matches.push(cached_file.clone());
                }
                return;
            }
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                if let Some(io_err) = e.io_error() {
                    match io_err.kind() {
                        std::io::ErrorKind::PermissionDenied => {
                            let mut count = permission_denied.lock().unwrap();
                            *count+=1;
                        }
                        _ => {
                            let mut count = other_errors.lock().unwrap();
                            *count+=1;
                        }
                    }
                } else {
                    let mut count = other_errors.lock().unwrap();
                    *count += 1;
                }
                return;
            }
        };

        if let Ok(date_modified) = get_file_time(entry.path()) {
            let file_info = Fileinfo {
                path: pathz.clone(),
                size: metadata.len(),
                date_modified,
            };
            {
                let mut cache_entries = updated_cache_entries.lock().unwrap();
                cache_entries.push(file_info.clone());
            }
            if entry.file_name().to_string_lossy() == search_file {
                let mut matches = current_matches.lock().unwrap();
                matches.push(file_info);
            }
        }
    });
    let t_cache_entries = updated_cache_entries.lock().unwrap();
    let t_current = current_matches.lock().unwrap();
    let t_files_scanned = *files_scanned.lock().unwrap();
    let t_permission_denied = *permission_denied.lock().unwrap();
    let t_other_errors = *other_errors.lock().unwrap();

    if let Err(e) = save_cache(cache_file, &t_cache_entries) {
        println!("Warning! Cache save error: {}", e);
    }

    if t_current.is_empty() {
        println!("No file named '{}' found.", search_file);
        return;
    }else {
        println!("Found {} matching file(s) out of {} scanned:", t_current.len(), t_files_scanned);
        for file in t_current.iter() { 
            let date = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(file.date_modified); 
            println!(" {} ({} bytes, modified: {:?})", file.path, file.size, date);
        }
    }
    println!("===== Final Scan Statistic  =====");
    println!("Files Scanned:                {}",t_files_scanned);
    println!("Permission Denied:            {}",t_permission_denied);
    println!("Other Errors:                 {}",t_other_errors);
    println!("Files Found:                  {}",t_current.len());
}

fn main() {
    let filename:&str ="cache.json";

    println!("Enter a the Director Path:");
    let mut path = String::new();
    io::stdin().read_line(&mut path).expect("Falied to read the path");
    let path = path.trim();
    if !std::path::Path::new(path).exists() {
        println!("Error: Directory '{}' not found", path);
        return;
    }

    println!("Enter the file to search with extension : ");
    let mut input_search = String::new();
    io::stdin().read_line(&mut input_search).expect("Falied to read the File Name ");
    let search = input_search.trim();
    if search.is_empty() {
        println!("Error : The search file cant be empty");
        return;
    }
    let start = Instant::now();
    walk(path, search,filename);
    let duration = start.elapsed();
    println!("Time taken: {:.2?} seconds",duration);
}