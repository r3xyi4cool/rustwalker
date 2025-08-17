use walkdir::WalkDir;
use std::io;
use std::time::Instant;

fn walk(path:&str,min_size: u64) {
    for entry in WalkDir::new(path){
        let entry = match entry{
            Ok(e)=>e,
            Err(e)=> {print!("Error while reading : {} ",e); continue;}
        };
        if !entry.file_type().is_file(){
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => { println!("Error reading metadata: {}", e); continue; }
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
        println!("Error: Directory '{}' does not exist", path);
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