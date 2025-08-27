use walkdir::WalkDir;
use std::fs::{self, metadata, File};
use std::{io::{self, BufReader}, path::Path};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json;
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize, Debug)]
struct Fileinfo{
    path:String,
    size:u64,
    date_modifed:u64,
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
        Ok(modifie) => modifie == cached_file.date_modifed,
        Err(_) => false,
    }
} 

fn walk(path:&str,min_size: u64,cache_file: &str) {
    let mut files_scanned = 0;
    let mut permission_denied = 0;
    let mut other_errors = 0;
    let mut found_files: Vec<(std::path::PathBuf, u64)> = Vec::new();

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

        if metadata.len() > min_size {
            found_files.push((entry.path().to_path_buf(),metadata.len()));
        }
    }

    if found_files.is_empty() {
        println!("No files larger than {} bytes found.", min_size);
        return;
    }else {
        println!("Found {} large files out of {} scanned:",found_files.len(),files_scanned);
        for (path,size) in &found_files {
            println!("  - {} ({} bytes)", path.display(), size);
        }
    }
    println!("===== Final Scan Statistic  =====");
    println!("Files Scanned:                {}",files_scanned);
    println!("Permission Denied:            {}",permission_denied);
    println!("Other Errors:                 {}",other_errors);
    println!("Files Found:                  {}",found_files.len());
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