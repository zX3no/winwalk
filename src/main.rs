use std::time::Instant;

use winwalk::*;

fn main() {
    let now = Instant::now();
    let files = walkdir("D:\\Opus").unwrap();
    println!("{:?} {}", now.elapsed(), files.len());

    for file in walkdir("D:\\Desktop").unwrap() {
        let system_time = file.last_write.system_time().unwrap();

        let pad = if file.is_dir() { "  " } else { "--" };
        println!("{pad}{}", file.name.to_string_lossy());
        println!("  {:?}", file.path);

        println!(
            "  Last Write Time: {:02}/{:02}/{} {:02}:{:02}:{:02}",
            system_time.wMonth,
            system_time.wDay,
            system_time.wYear,
            system_time.wHour,
            system_time.wMinute,
            system_time.wSecond
        );
        println!("  Size: {:?}", file.size);
        println!("  Attributes: {:?}", file.attributes);
        println!();
    }
}
