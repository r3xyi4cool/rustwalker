use walkdir::WalkDir;
use std::io;
use std::time::Instant;


fn walk(path:&str,min_size: u64) {
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
    let start = Instant::now();
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

    walk(path, size);
    let duration = start.elapsed();
    println!(
        "Time taken: {:.2?} seconds",
        duration
    );
}