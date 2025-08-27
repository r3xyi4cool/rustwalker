use walkdir::WalkDir;
use std::fs::{self, File};
use std::{io::{self, BufReader}, path::Path};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json;
use tempfile::NamedTempFile;

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

fn walk(path:&str,min_size: u64,cache_file: &str) {
    let mut files_scanned = 0;
    let mut permission_denied = 0;
    let mut other_errors = 0;

    let cache = load_cache(cache_file).unwrap_or_else(|e|{
        println!("No cache found ({}) creating a new vector",e);
        Vec::new()
    });

    let mut cachemap: std::collections::HashMap<String,Fileinfo> = cache.into_iter().map(|f| (f.path.clone(),f)).collect();
    let mut current = Vec::new();

    print!("Scanning .......... ");

    for entry in WalkDir::new(path){
        let entry = match entry{
            Ok(e)=>e,
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
        if !entry.file_type().is_file(){
            continue;
        }
        files_scanned += 1;
        let pathz = entry.path().display().to_string();

        if let Some(cache_filed) = cachemap.get(&pathz) {
            if is_file_uptodate(cache_filed) {
                if cache_filed.size >= min_size {
                    current.push(cache_filed.clone());
                }
                continue;
            }
        }
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

        if metadata.len() >= min_size {
            if let Ok(date_modified) = get_file_time(entry.path()) {
                let file_info = Fileinfo {
                    path: pathz.clone(),
                    size: metadata.len(),
                    date_modified,
                };
                current.push(file_info.clone());
                cachemap.insert(pathz, file_info);
            }
        }
    }
    let all_data: Vec<Fileinfo> = cachemap.into_values().collect();
    if let Err(e) = save_cache(cache_file,&all_data) {
        println!("Warning ! Error : {} ",e);
    }

    if current.is_empty() {
        println!("No files larger than {} bytes found.", min_size);
        return;
    }else {
        println!("Found {} large files out of {} scanned:",current.len(),files_scanned);
        for file in &current {
            let date = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(file.date_modified);
            println!("  {} ({} bytes, modified: {:?})", file.path, file.size, date);
        }
    }
    println!("===== Final Scan Statistic  =====");
    println!("Files Scanned:                {}",files_scanned);
    println!("Permission Denied:            {}",permission_denied);
    println!("Other Errors:                 {}",other_errors);
    println!("Files Found:                  {}",current.len());
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

    println!("Enter the size (eg:10Gb,20Mb,500Kb) : ");
    let mut input_size = String::new();

    io::stdin().read_line(&mut input_size).expect("Error Reading the size");
    let num_part: String = input_size.chars().take_while(|c| c.is_numeric()).collect();
    let unit_part: String = input_size.chars().skip_while(|c| c.is_numeric()).collect();
    let unit_part = unit_part.trim();

    let number:u64 = match num_part.parse() {
        Ok(n) => n,
        Err(_)=>{println!("Error : Invalid Number"); return;}
        
    };
    
    let size = match unit_part.to_lowercase().as_str(){
        "gb" => number*1024*1024*1024,
        "mb" => number*1024*1024,
        "kb" => number*1024,
        "" => number,
        _ => {
            println!("Error Invalid Unit");
            return;
        }
    };

    let start = Instant::now();
    walk(path, size,filename);
    let duration = start.elapsed();
    println!("Time taken: {:.2?} seconds",duration);
}