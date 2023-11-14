```rs
use winwalk::*;

fn main() {
   for drive in drives().into_iter().flatten() {
        println!("Found Drive: {drive}");
    }

    println!();

    for file in walkdir("D:\\Desktop", 1).into_iter().flatten() {
        let pad = if file.is_folder { "  " } else { "--" };
        println!("{pad}{}", file.name);
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
        println!("  Directory: {:?}", file.is_folder);
        println!();
    }
}
```