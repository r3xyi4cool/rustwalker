use walkdir::WalkDir;
use std::io::{self, Write};
use std::time::Instant;

fn walk(min_size: u64) {
    let mut error_count = 0;
    let mut files_found = 0;

    for entry in WalkDir::new(".") {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    match entry.metadata() {
                        Ok(metadata) => {
                            let size = metadata.len();
                            if size > min_size {
                                println!("File: {} (Size: {} bytes)", entry.path().display(), size);
                                files_found += 1;
                            }
                        }
                        Err(e) => {
                            error_count += 1;
                            eprintln!("Metadata error #{}: {} for {}", error_count, e, entry.path().display());
                        }
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Entry error #{}: {}", error_count, e);
            }
        }
    }

    println!("\nFound {} file(s) larger than {} bytes", files_found, min_size);
}


fn main() {
    print!("Enter the size of the files you want to filter: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");

    let input = input.trim();

    let num_part:String = input.chars().take_while(|c| c.is_numeric()).collect();
    let char_part:String = input.chars().skip_while(|c| c.is_numeric()).collect();

    if num_part.is_empty() {
        println!("Error: No number provided");
        return;
    }

    let num_value: u64 = match num_part.parse() {
        Ok(val) => val,
        Err(_) => {
            println!("Error: Invalid number");
            return;
        }
    };


    println!("Number : {}",num_part);
    println!("Number : {}",char_part);
    
    let size = match char_part.as_str() {
        "Gb" => num_value * 1024 * 1024 * 1024,
        "Mb" => num_value * 1024 * 1024 ,
        "Kb" => num_value * 1024,
        _=>{
            println!("Error Invalid unit");
            return;
        }

    };
    println!("Size : {}",size);

    let start = Instant::now();
    
    println!("Directory Walking:");
    walk(size);

    let duration = start.elapsed();
    println!(
        "Time taken: {:.2?} seconds",
        duration
    );
    
}
