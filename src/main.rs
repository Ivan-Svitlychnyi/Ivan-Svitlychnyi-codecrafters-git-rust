use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
//use sha1::digest;
use sha1::{Digest, Sha1};
#[allow(unused_imports)]
use std::env;
//use std::fmt::Error;
#[allow(unused_imports)]
use std::fs;
use std::io;
use std::io::prelude::*;
use std::str::FromStr;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        panic!("enter the arguments!");
    }
    if args[1] == "init" {
        git_init().unwrap();
        // println!("{}", )
    } else if args[1] == "cat-file" && args[2] == "-p" {
        print!("{}", read_git_object(&args[3]).unwrap());
    } else if args[1] == "hash-object" && args[2] == "-w" {
        let file_data = fs::read(args[3].to_string()).unwrap();
        println!(
            "hash-object in: {:?}",
            write_hash_object(file_data, "blob").unwrap()
        );
    } else if args[1] == "ls-tree" && args[2] == "--name-only" {
        let result = read_tree(&args[3]).unwrap();
        for s in result {
            println!("{}", s);
        }
    } else if args[1] == "write-tree" {
        print!("{}", write_tree(&".".to_string()).unwrap());
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
fn write_hash_object(file_data: Vec<u8>, file_type: &str) -> Result<String, io::Error> {
    #[allow(unsafe_code)]
    let s = unsafe { String::from_utf8_unchecked(file_data.clone()) };

    let store = format!("{file_type} {}\x00{}", file_data.len(), s);

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

fn read_tree(file_path: &String) -> Result<Vec<String>, io::Error> {
    let (sub_dir, sha_num) = sha1_parse(&file_path);

    let full_path = format!(".git/objects/{}/{}", sub_dir, sha_num);

    let git_data = fs::read(full_path).unwrap();

    let mut git_data = ZlibDecoder::new(&git_data[..]);

    let mut file_content = Vec::new();

    git_data.read_to_end(&mut file_content).unwrap();

    let cursor = io::Cursor::new(file_content);

    let split_data = cursor.split(b'\x00').skip(1).map(|l| l.unwrap());

    let mut result = Vec::new();

    for i in split_data {
        let chars = String::from_utf8_lossy(&i);
        let chars = chars.split_whitespace();
        let x = chars.last().unwrap();
        result.push(x.to_string());
    }
    result.pop();

    Ok(result)
}

fn write_tree(file_path: &String) -> Result<String, io::Error> {
    let mut sha_out: String = "".to_string();

    let mut entries = fs::read_dir(file_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    entries.sort();

    for dir in entries {
        let mode;

        let path_name = dir.as_path().to_str().unwrap();

        if path_name == "./.git" {
            continue;
        }
        let sha_file;
        if dir.is_dir() {
            // println!("dir: {}", path_name);
            mode = "40000";
            let sha_file1 = write_tree(&String::from_str(path_name).unwrap());

            sha_file = hex::decode(sha_file1.unwrap()).unwrap();
        } else
        /*if dir.is_file()*/
        {
            mode = "100644";
            //  println!("file: {}",  path_name);

            let file_data = fs::read(&path_name).unwrap();

            let sha_file1 = write_hash_object(file_data, "blob");

            sha_file = hex::decode(&sha_file1.unwrap()).unwrap();

            // println!("file out: {:?}", &sha_file);
        }

        // println!("sha_file: {:?}", &sha_file);
        #[allow(unsafe_code)]
        let s = unsafe { String::from_utf8_unchecked(sha_file) };
        sha_out += &format!(
            "{mode} {}\x00{}",
            dir.file_name().unwrap().to_str().unwrap(),
            s
        );

        // println!("sha_out: {:?}", sha_out);
    }
    let res = write_hash_object(sha_out.into_bytes(), "tree");
    res
}
