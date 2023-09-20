```rs
use winwalk::*;

fn main() {
    let depth = 0; //Recursively walk directory.
    for file in walkdir("D:\\Desktop", depth).into_iter().flatten() {
        println!("{file:?}");
    }
}
```