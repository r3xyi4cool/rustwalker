use walkdir::WalkDir;
use std::io;
use std::time::Instant;
use std::fs::File;
use std::io::ErrorKind;

fn walk(path:&str,min_size: u64) {
    let mut file_num = 0;
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
            Err(_) => continue,
        };
        if metadata.len() > min_size {
            println!("File larger than {} bytes: {:?}", min_size, entry.path());
        }
    }
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