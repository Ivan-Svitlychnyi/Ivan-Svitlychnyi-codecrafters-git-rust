#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
use flate2::read::GzDecoder;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
     let args: Vec<String> = env::args().collect();
     if args[1] == "init" {
         fs::create_dir(".git").unwrap();
         fs::create_dir(".git/objects").unwrap();
         fs::create_dir(".git/refs").unwrap();
         fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
         println!("Initialized git directory")

     } else if args.contains(&"cat-file".to_string()) && args.contains(&"-p".to_string()) {

    let mut d = GzDecoder::new(".git/objects".as_bytes());
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();
    print!("{}", s);


        // println!("unknown command: {}", args[1])
     }
  

}
