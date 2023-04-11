#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
use std::io;
use flate2::read::ZlibDecoder;
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
     let args: Vec<String> = env::args().collect();
     println!("{:?}", args);
     if args[1] == "init" {
         fs::create_dir(".git").unwrap();
         fs::create_dir(".git/objects").unwrap();
         fs::create_dir(".git/refs").unwrap();
         fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
         println!("Initialized git directory")

     } //else if args.contains(&"cat-file".to_string()) && args.contains(&"-p".to_string()) {
       else if args[1] =="cat-file" && args[2]=="-p" {

            let mut new_path = ".git/objects/pack/".to_string();
            new_path.push_str(&format!("{}", args[3]));

            println!("Command: {}", new_path);  

            let git_data = fs::read(new_path).unwrap();
            let mut z = ZlibDecoder::new(&git_data[..]);
            let mut s = String::new();
            z.read_to_string(&mut s).unwrap();
            print!("{}", s);
        
     }
        else{
         println!("unknown command: {}", args[1])  
        }

}
