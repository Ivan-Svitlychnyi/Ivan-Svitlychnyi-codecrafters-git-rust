#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
//use std::io;
use flate2::read::ZlibDecoder;
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        panic!("enter the arguments!");
    }
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")

    } else if args[1] == "cat-file" && args[2] == "-p" {
        
        let chars: Vec<char> = args[3].chars().collect();
        let sub_dir = chars[..2].iter().collect::<String>();
        let sha_num = chars[2..].iter().collect::<String>();

        let new_path = &format!(".git/objects/{}/{}", sub_dir, sha_num);

        let git_data = fs::read(new_path).unwrap();
        let mut git_data = ZlibDecoder::new(&git_data[..]);

        let mut s_git_data = String::new();
        git_data.read_to_string(&mut s_git_data).unwrap();

        let s_chars: Vec<char> = s_git_data.chars().collect();
        let s = s_chars[8..].iter().collect::<String>();

        print!("{}", s);
    } else {
        println!("unknown command: {}", args[1])
    }
}

// fn read_git_objects(args: &[String]) -> String {

//     let chars : Vec<char> = args[3].chars().collect();
//     let sub_dir = chars[..2].to_vec().iter().cloned().collect::<String>();
//     let sha_num = chars[2..].to_vec().iter().cloned().collect::<String>();

//     let mut new_path = ".git/objects/".to_string();
//     new_path.push_str(&format!("{}/{}",sub_dir, sha_num));

//      let git_data = fs::read(new_path).unwrap();
//      let mut z = ZlibDecoder::new(&git_data[..]);
//      let mut s = String::new();
//      z.read_to_string(&mut s).unwrap();

//      let chars : Vec<char> = s.chars().collect();
//      let s = chars[8..].iter().filter(|c| **c != '\n').collect::<String>();

// }
