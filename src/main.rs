use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io;
use std::io::prelude::*;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        panic!("enter the arguments!");
    }
    if args[1] == "init" {
        println!("{}", git_init().unwrap())
    } else if args[1] == "cat-file" && args[2] == "-p" {
        print!("{}", read_git_object(&args[3]).unwrap());
    } else if args[1] == "hash-object" && args[2] == "-w" {
        println!("hash-object in: {:?}", write_hash_object(&args[3]).unwrap());
    } else if args[1] == "ls-tree" && args[2] == "--name-only" {
        //  -d '{"base_tree":"9fb037999f264ba9a7fc6274d15fa3ae2ab98312",
        //"tree":[{"path":"file.rb","mode":"100644","type":"blob","sha":"44b4fc6d56897b048c772eb4087f854f46256132"}]}'
        println!("sha: {}", args[3]);
        let chars: Vec<char> = args[3].chars().collect();
        let sub_dir = chars[..2].iter().collect::<String>();
        let sha_num = chars[2..].iter().collect::<String>();
        let full_path = format!(".git/objects/{}/{}", sub_dir, sha_num);

        let git_data = fs::read(full_path).unwrap();

        //println!("ls-tree: {}", String::from_utf8(git_data.clone()).unwrap());

        let mut git_data = ZlibDecoder::new(&git_data[..]);

        let mut file_content = Vec::new();

        git_data.read_to_end(&mut file_content).unwrap();

        // let d_len = file_content.len();

        let cursor = io::Cursor::new(file_content);

        let split_data = cursor.split(b'\x00').skip(1).map(|l| l.unwrap());

        // for i in range(1, len(data) - 1):
        // +            filename = data[i].split(b" ")[-1]
        // +            files.append(filename)
        // +        for file in files:
        // +            print(file.decode())

        let mut result = Vec::new();

        for i in split_data {
            let chars = String::from_utf8_lossy(&i);
            let chars = chars.split_whitespace();

            let x = chars.last().unwrap();          
            result.push(x.to_string() );
        }
        result.pop();

        for i in result{
        print!("{}", i);
        }
       

     

    } else {
        println!("unknown command: {:#?}", args)
    }
}
fn git_init() -> Result<String, io::Error> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;

    Ok("Initialized git directory".to_string())
}

fn sha1_parse(sha_1: &String) -> (String, String) {
    let chars: Vec<char> = sha_1.chars().collect();
    let sub_dir = chars[..2].iter().collect::<String>();
    let sha_num = chars[2..].iter().collect::<String>();
    (sub_dir, sha_num)
}

fn read_git_object(git_path: &String) -> Result<String, io::Error> {
    let (sub_dir, sha_num) = sha1_parse(&git_path);
    let full_path = format!(".git/objects/{}/{}", sub_dir, sha_num);

    let git_data = fs::read(full_path)?;
    let mut git_data = ZlibDecoder::new(&git_data[..]);

    let mut s_git_data = String::new();
    git_data.read_to_string(&mut s_git_data)?;

    let git_data_chars: Vec<char> = s_git_data.chars().collect();

    let git_data = git_data_chars[8..]
        .iter()
        .filter(|c| **c != '\n')
        .collect::<String>();

    Ok(git_data)
}
fn write_hash_object(file_path: &String) -> Result<String, io::Error> {
    let file_data = fs::read(file_path.to_string())?;

    let store = format!(
        "blob {}\x00{}",
        file_data.len(),
        String::from_utf8(file_data).unwrap()
    );

    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(store.as_bytes())?;
    let compressed = e.finish()?;

    let mut hasher = Sha1::new();
    hasher.update(store);
    let result = hasher.finalize();
    let result = hex::encode(&result[..]);

    let (sub_dir, sha_num) = sha1_parse(&result);

    let sub_dir_path = format!(".git/objects/{}/", sub_dir);

    let full_path = format!("{sub_dir_path}{}", sha_num);

    fs::create_dir(sub_dir_path)?;
    fs::write(full_path, compressed)?;
    Ok(result)
}
