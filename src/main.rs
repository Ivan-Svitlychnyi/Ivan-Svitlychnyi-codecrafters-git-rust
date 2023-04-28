use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use reqwest::header;
use reqwest::header::CONTENT_TYPE;
use sha1::{Digest, Sha1};
#[allow(unused_imports)]
use std::env;
//use std::fmt::Error;
use std::collections::HashMap;
#[allow(unused_imports)]
use std::fs;
use std::io;
use std::io::prelude::*;
use std::str;
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
        let (_, sha1_out) = write_hash_object(file_data, "blob").unwrap();
        println!("hash-object in: {}", sha1_out);
    } else if args[1] == "ls-tree" && args[2] == "--name-only" {
        let result = read_tree(&args[3]).unwrap();
        for s in result {
            println!("{}", s);
        }
    } else if args[1] == "write-tree" {
        let (_, sha1_out) = write_tree(&".".to_string()).unwrap();
        print!("{}", sha1_out);
    } else if args[1] == "commit-tree" {
        print!("{}", create_commit(&args).unwrap());
    } else if args[1] == "clone" {
        print!("{}", clone_repo(&args).unwrap());
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
fn write_hash_object(file_data: Vec<u8>, file_type: &str) -> Result<(Vec<u8>, String), io::Error> {
    #[allow(unsafe_code)]
    let store = format!("{file_type} {}\x00{}", file_data.len(), unsafe {
        String::from_utf8_unchecked(file_data)
    });

    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

    e.write_all(store.as_bytes())?;
    let compressed = e.finish()?;

    let mut hasher = Sha1::new();
    hasher.update(store.as_bytes());

    let result = hasher.finalize();

    let hex_result = hex::encode(&result[..]);

    let (sub_dir, sha_num) = sha1_parse(&hex_result);

    let sub_dir_path = format!(".git/objects/{}/", sub_dir);

    let full_path = format!("{sub_dir_path}{}", sha_num);

    fs::create_dir(sub_dir_path)?;
    fs::write(full_path, compressed)?;
    Ok((result.to_vec(), hex_result))
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

fn write_tree(file_path: &String) -> Result<(Vec<u8>, String), io::Error> {
    let mut sha_out: String = "".to_string();

    let mut entries = fs::read_dir(file_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    entries.sort();

    for dir in entries {
        let mode;

        let path_name = dir.to_str().unwrap();
        //  println!("dir: {}", path_name);

        if path_name == "./.git" {
            continue;
        }
        let sha_file;
        if dir.is_dir() {
            mode = "40000";
            (sha_file, _) = write_tree(&String::from_str(path_name).unwrap()).unwrap();
        } else
        /*if dir.is_file()*/
        {
            mode = "100644";

            let file_data = fs::read(&path_name).unwrap();

            (sha_file, _) = write_hash_object(file_data, "blob").unwrap();
        }

        #[allow(unsafe_code)]
        let s = unsafe { String::from_utf8_unchecked(sha_file) };
        sha_out += &format!(
            "{mode} {}\x00{}",
            dir.file_name().unwrap().to_str().unwrap(),
            s
        );
    }
    let res = write_hash_object(sha_out.into_bytes(), "tree");
    res
}

fn create_commit(args: &[String]) -> Result<String, io::Error> {
    let (tree_sha, parent_commit_sha, data) = (&args[2], &args[4], &args[6]);

    let user_metadata = "author Admin <admin@example.com> 1652217488 +0300\ncommitter Name <committer@example.com> 1652224514 +0300".to_string();

    let content =
        format!("tree {tree_sha}\nparent {parent_commit_sha}\n{user_metadata}\n\n{data}\n");

    //println!("content: {:?}", &content);
    let (_, sha) = write_hash_object(content.into_bytes(), "commit")?;

    Ok(sha)
}

fn clone_repo(args: &[String]) -> Result<String, io::Error> {
    // ["/tmp/codecrafters-git-target/release/git-starter-rust",
    // "clone",
    // "https://github.com/codecrafters-io/git-sample-2",
    // "test_dir",]

    let (url, target_dir) = (&args[2], &args[3]);

    fs::create_dir(&target_dir).unwrap();

    fs::create_dir(target_dir.clone() + "/.git").unwrap();

    fs::create_dir(target_dir.clone() + "/.git/objects/")?;

    fs::write(
        target_dir.to_owned() + "/.git/HEAD",
        "ref: refs/heads/master\n",
    )?;

    // let body = reqwest::get(url + "/info/refs?service=git-upload-pack")
    // .await?
    // .text()
    // .await?;
    let body = reqwest::blocking::get(url.clone() + "/info/refs?service=git-upload-pack")
        .unwrap()
        .text()
        .unwrap();

    println!("body = {:#?}", body);
    let content = body.split("\n");

    let mut pack_hash = String::new();

    for c in content.clone() {
        if c.contains("refs/heads/master") && c.contains("003f") {
            let tup = c.split(" ").enumerate();

            for (num, value) in tup {
                if num == 0 {
                    pack_hash = value[4..].to_string();
                }
            }
        }
    }

    println!("pack_hash = {}", pack_hash);
    let post_url = url.to_owned() + "/git-upload-pack";

    let mut headers = header::HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        header::HeaderValue::from_static("application/x-git-upload-pack-request"),
    );
    let data = format!("0032want {pack_hash}\n00000009done\n");

    println!("0032 = {}", data);
    let client = reqwest::blocking::Client::new();
    //let data = data.as_bytes();
    let res = client.post(post_url).headers(headers).body(data);

    let res_send = res.send().unwrap();

    if !res_send.status().is_success() {
        println!("Something else happened. Status: {:?}", res_send.status());
    } else {
        println!("success!");
        let body = res_send.bytes().unwrap();
        let res_data = body.to_vec();
        let res_data_size = res_data.len() - 20;
        println!("res_data_size: {:?}", res_data_size);

        let entries_bytes = res_data[16..20].try_into().unwrap();
      //  println!("entries_bytes: {:#?}", entries_bytes);
        let num = u32::from_be_bytes(entries_bytes);
        println!("num: {:?}", num);
        let data_bytes: Vec<u8> = res_data[20..res_data_size].try_into().unwrap();

        // println!("data_bytes: {:?}", data_bytes);
        let mut objs = HashMap::new();

        let mut seek = 0;
        let mut objs_count = 0;

        while objs_count != num {
            objs_count += 1;
            let first = data_bytes[seek];
            let mut obj_type: usize = ((first & 112) >> 4).into();
            println!("obj_type: {:?}", obj_type);
            while data_bytes[seek] > 128 {
                seek += 1;
            }
            seek += 1;
           // println!("seek : {:?}", seek);
            if obj_type < 7 {

                let mut git_data = ZlibDecoder::new(&data_bytes[seek..]);

                //println!("git_data");

                let mut v_git_data = Vec::new();
               

                git_data.read_to_end(&mut v_git_data).unwrap();

                #[allow(unsafe_code)]
                let s_git_data = unsafe {String::from_utf8_unchecked(v_git_data) };

                let data_type = ["", "commit", "tree", "blob","", "tag","ofs_delta"];

      
                let mut obj_write_data = format!("{} {}\0", data_type[obj_type], &s_git_data.len());
               // println!("obj_write_data if: {:?}", obj_write_data);

                obj_write_data += &s_git_data;

               // println!("obj_write_data & git_data if: {:?}", obj_write_data);

                let mut hasher = Sha1::new();
                hasher.update(obj_write_data.as_bytes().to_vec());

                let result = hasher.finalize();

                let hex_result = hex::encode(&result[..]);
               // println!("hex_result if: {:?}", hex_result);

                let f_path = target_dir.to_owned() + &format!("/.git/objects/{}", &hex_result[..2]);
              //  println!("f_path if: {:?}", &f_path);

                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

                e.write_all(obj_write_data.as_bytes())?;
                let compressed = e.finish()?;
                //  if !does_folder_exist_in_current_directory(f_path.clone()).unwrap(){
                fs::create_dir_all(&f_path)?;
                //}
                let f_path = f_path + "/" + &hex_result[2..];
              //  println!("f_path if: {:?}", &f_path);
                fs::write(f_path, compressed.to_vec())?;
                

                objs.insert(hex_result, (s_git_data.clone(), obj_type));
                //println!("objs if: {:#?}", objs);

                // let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                // e.write_all(s_git_data.as_bytes()).unwrap();

                // let compressed = e.finish().unwrap();

                seek += git_data.total_in() as usize;
            } else {
                println!("else !!!!!!!!!!!!!!!!");
                let k = &data_bytes[seek..seek + 20];
                let k = hex::encode(k);
                println!("k: {:#?}", k);

                let (base, elem_num) = objs[&k].to_owned();

                seek += 20;

                let mut delta = ZlibDecoder::new(&data_bytes[seek..]);

                let mut v_delta = Vec::new();
                delta.read_to_end(&mut v_delta).unwrap();

            //     let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

            //     e.write_all(&v_delta).unwrap();

            //    let compressed_data = e.finish().unwrap();
                // println!("v_delta: {:#?}", &v_delta);
                let content = identify(&v_delta, base);
                obj_type = elem_num;
                //println!("content else: {:#?}", &content);
                //println!("obj_type else: {:#?}", &obj_type);

                let data_type = ["", "commit", "tree", "blob","", "tag","ofs_delta","refs_delta"];

                let mut obj_write_data = format!("{} {}\0", data_type[obj_type], content.len());

              //  println!("obj_write_data : {:?}", obj_write_data);

                obj_write_data += &content;

                //-----------------------

                let mut hasher = Sha1::new();
                hasher.update(obj_write_data.as_bytes());
                let result = hasher.finalize();

                let hex_result = hex::encode(&result[..]);
              //  println!("hex_result: {:?}", hex_result);

                let f_path = target_dir.to_owned() + &format!("/.git/objects/{}", &hex_result[..2]);
              //  println!(" f_path: {:?}", &f_path);

                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

                e.write_all(obj_write_data.as_bytes())?;
                let compressed = e.finish().unwrap();

               // if !does_folder_exist_in_current_directory(f_path.clone()).unwrap() {
                    fs::create_dir(f_path).unwrap();
                //}
                fs::write(
                    target_dir.to_owned()
                        + &format!("/.git/objects/{}/{}", &hex_result[..2], &hex_result[2..]),
                    &compressed,
                )?;

                objs.insert(hex_result, (content.clone(), obj_type));
               // println!("objs: {:#?}", objs);
                seek += delta.total_in() as usize;
            }
        }
        let git_path = format!("/.git/objects/{}/{}", &pack_hash[..2], &pack_hash[2..]);
        let git_data = fs::read(git_path).unwrap();

        let mut data = ZlibDecoder::new(&git_data[..]);

        let mut v_delta = String::new();

        data.read_to_string(&mut v_delta)?;

        



        let data = v_delta.split("/");
        let data = data.clone().nth(0).unwrap().split(" ");
        let tree_sha = data.clone().nth(data.count() -1).unwrap();

        println!("tree_sha: {}", &tree_sha);

        //let path_f = target_dir.to_owned() + &format!("/.git/objects/{}/{}",&tree_sha[..2],&tree_sha[2..]);
       // let (_, sha1_out) = write_tree(&path_f).unwrap();
       // print!("{}", sha1_out);
    }
    //-2-------------------------------------------------------------------------------

    Ok(" ".to_owned())
}

// fn does_folder_exist_in_current_directory(cur_dir: String) -> Result<bool, io::Error> {
//     Ok(fs::read_dir(cur_dir)?.any(|x| {
//         let x = x.unwrap();
//         x.file_type().unwrap().is_dir()
//     }))
// }
//***************************************************************************************************** */
fn identify(delta: &[u8], base: String) -> String {
    println!("fidentify !!!!!!!!!!!");
    let mut seek: usize = 0;
   // println!("delta: {:#?}", delta);
    while delta[seek] > 128 {
        seek += 1;
    }
    seek += 1;
    while delta[seek] > 128 {
        seek += 1;
    }
    seek += 1;
    let mut content = String::new();

    let delta_len = delta.len();
    println!(" delta_len: {:?}", &delta_len);
    while seek < delta_len {
        let instr_byte = delta[seek];
        seek += 1;
       println!(" instr_byte: {:?}", &instr_byte);

        if instr_byte >= 128 {
            let offset_key = instr_byte & 15;
            println!("offset_key: {:?}", & offset_key);
            //let offset_key_bin_str = offset_key;

            let length = offset_key.count_ones() + offset_key.count_zeros();
            println!("length: {:?}", &length);

            let mut offset_bytes = Vec::new();
            for n in 2..length {
                
                let b = offset_key >> n & 1;

                println!("b offset_key: {}", b);

                if b == 1 {
                    offset_bytes.push(delta[seek]);
                    seek += 1
                } else {
                    offset_bytes.push(0);
                }
            }
            println!("offset_bytes: {:?}", &offset_bytes);
            offset_bytes.reverse();
            let offset = usize::from_le_bytes(offset_bytes.try_into().unwrap());

            println!("offset: {:?}", &offset);

            let len_key = (instr_byte & 0b01110000) >> 4;
            let length = len_key.count_ones() + len_key.count_zeros();
            println!("  length key: {:?}", &length);

            let mut len_bytes = Vec::new();
            for n in 2..length {
                let b = len_key >> n & 1;

                println!("b len_key:{}", b);

                if b == 1 {
                    len_bytes.push(delta[seek]);
                    seek += 1
                } else {
                    len_bytes.push(0);
                }
            }
            let len_int = usize::from_le_bytes(len_bytes.try_into().unwrap());

            content += &base[offset..offset + len_int];
        } else {
            println!("instr_byte:{}", instr_byte);
            let num_bytes = instr_byte & 0b01111111;
            let num_bytes = usize::from(num_bytes);

            content += &String::from_utf8_lossy(&delta[seek..seek + num_bytes]);
            seek += num_bytes
        }
    }
    content
}
