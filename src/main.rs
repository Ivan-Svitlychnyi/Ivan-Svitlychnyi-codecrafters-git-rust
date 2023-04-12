#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::prelude::*;
//use std::io;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};


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
        let full_path = format!(".git/objects/{}/{}", sub_dir, sha_num);

        print!("{}", read_git_object(&full_path));
    } else if args[1] == "hash-object" && args[2] == "-w" {
        println!("hash-object in: {:?}", &args);

        let file_data = fs::read(args[3].to_string()).unwrap();
        //let metadata = fs::metadata(args[3].to_string()).unwrap().len();
        //println!("metadata: {:?}", &metadata);
        let header = format!("blob {}\x00", file_data.len());

      
        let store = header + &format!("{}",String::from_utf8(file_data).unwrap());

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(store.as_bytes()).unwrap();
        let compressed = e.finish().unwrap();

       // println!("compressed: {:?}", &compressed);

        let mut hasher = Sha1::new();
        hasher.update(store);
        let result = hasher.finalize();
        println!("hasher: {:?}", &result);
        let result = hex::encode(&result[..]);
        println!("SHA: {:?}", &result);

        let chars: Vec<char> = result.chars().collect();
        let sub_dir = chars[..2].iter().collect::<String>();
        let sha_num = chars[2..].iter().collect::<String>();
        let sub_dir_path = format!(".git/objects/{}/", sub_dir);
        let full_path = format!("{sub_dir_path}{}", sha_num);
        println!("full_path: {:?}", &full_path);

        fs::create_dir(sub_dir_path).unwrap();
        fs::write(full_path, compressed).unwrap();

    } else {
        println!("unknown command: {}", args[1])
    }
}

fn read_git_object(git_path: &String) -> String {
    let git_data = fs::read(git_path).unwrap();
    let mut git_data = ZlibDecoder::new(&git_data[..]);

    let mut s_git_data = String::new();
    git_data.read_to_string(&mut s_git_data).unwrap();

    let git_data_chars: Vec<char> = s_git_data.chars().collect();

    let git_data = git_data_chars[8..]
        .iter()
        .filter(|c| **c != '\n')
        .collect::<String>();

    git_data
}
