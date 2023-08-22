use winwalk::*;

fn main() {
    for file in walkdir("D:\\Desktop", 1).into_iter().flatten() {
        let pad = if file.is_folder() { "  " } else { "--" };
        println!("{pad}{}", file.name.to_string_lossy(),);
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
