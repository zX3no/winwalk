use std::time::Instant;

use winwalk::*;

fn main() {
    let now = Instant::now();
    let files = walkdir("D:\\Opus", Some(1)).unwrap();
    println!("{:?} {}", now.elapsed(), files.len());

    for file in walkdir("D:\\Desktop", Some(1)).unwrap() {
        let pad = if file.is_folder() { "  " } else { "--" };
        println!("{pad}{}", file.name.to_string_lossy());
        println!("  {:?}", file.path);

        println!(
            "  Last Write Time: {:02}/{:02}/{} {:02}:{:02}:{:02}",
            file.last_write.day,
            file.last_write.month,
            file.last_write.year,
            file.last_write.hour,
            file.last_write.minute,
            file.last_write.second,
        );
        println!("  Size: {:?}", file.size);
        println!("  Attributes: {:?}", file.attributes);
        println!();
    }
}
