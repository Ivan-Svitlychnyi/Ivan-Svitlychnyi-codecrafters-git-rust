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
//use std::fmt::format;
//use std::fmt::format;
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

        print!("{}", String::from_utf8(read_git_object(&args[3]).unwrap()).unwrap());

    } else if args[1] == "hash-object" && args[2] == "-w" {
        let file_data = fs::read(args[3].to_string()).unwrap();
        let (_, sha1_out) = write_hash_object(file_data, "blob").unwrap();
        println!("hash-object in: {}", sha1_out);

    } else if args[1] == "ls-tree" && args[2] == "--name-only" {
        let result = read_tree(&args[3]).unwrap();
        for s in result {
            println!("{}", String::from_utf8(s).unwrap());
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

fn zlib_decode(enc_data:Vec<u8>) -> Result<Vec<u8>, io::Error>{

    let mut enc_data = ZlibDecoder::new(&enc_data[..]);
    let mut dec_data = Vec::new();
    enc_data.read_to_end(&mut dec_data)?;

     Ok(dec_data)
}

fn zlib_encode(data:Vec<u8>) -> Result<Vec<u8>, io::Error>{

    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&data[..])?;
    let compressed = e.finish()?;

   Ok(compressed)
}
fn make_hash(data:Vec<u8>)->Result<(Vec<u8>, String), io::Error>{

    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    let hex_result = hex::encode(&result[..]);

    Ok((result.to_vec(), hex_result))
}

fn read_git_object(git_path: &String) -> Result<Vec<u8>, io::Error> {
    let (sub_dir, sha_num) = sha1_parse(&git_path);
    let full_path = format!(".git/objects/{}/{}", sub_dir, sha_num);
    let git_data = fs::read(full_path)?;

    let git_data = zlib_decode(git_data)?;

   let git_data: Vec<u8> = git_data[8..].iter().filter(|c| **c  != '\n' as u8).map(|x| *x as u8).collect();
  
    Ok(git_data)

}

fn write_hash_object(mut file_data: Vec<u8>, file_type: &str) -> Result<(Vec<u8>, String), io::Error> {
    
    // #[allow(unsafe_code)]
    // let store = format!("{file_type} {}\x00{}", file_data.len(), unsafe {
    //     String::from_utf8_unchecked(file_data)
    // }).to_string();
/******************************************** */
let mut store: Vec<u8> = Vec::new();
store.append(&mut file_type.as_bytes().to_vec());
store.push(' ' as u8);
store.append(&mut file_data.len().to_le_bytes().to_vec());
store.push('\x00' as u8);
store.append(&mut file_data);
/******************************************* */
    let compressed = zlib_encode(store.clone().into())?;

    let (result, hex_result) = make_hash(store.into())?;

    let (sub_dir, sha_file) = sha1_parse(&hex_result);

    let sub_dir_path = format!(".git/objects/{}/", sub_dir).to_string();

    let full_path = format!("{sub_dir_path}{}", sha_file).to_string();

    fs::create_dir_all(sub_dir_path)?;
    fs::write(full_path, compressed)?;

    Ok((result, hex_result))

}


fn read_tree(file_path: &String) -> Result<Vec<Vec<u8>>, io::Error> {

    let (sub_dir, sha_num) = sha1_parse(&file_path);

    let full_path = format!(".git/objects/{}/{}", sub_dir, sha_num);

    let file_content = fs::read(full_path).unwrap();
    
    let file_content = zlib_decode(file_content)?;

    let split_data = file_content[..].split(|x| *x == '\x00' as u8).skip(1);

    let mut result:Vec<Vec<u8>> = Vec::new();

    for i in split_data { 

    let  parts = i.split(|x| *x == ' ' as u8);
    let x = parts.last().unwrap();
     result.push(x.to_vec());

    }
    result.pop();

    Ok(result)
}


fn write_tree(file_path: &String) -> Result<(Vec<u8>, String), io::Error> {
   // let mut sha_out: String = "".to_string();
    let mut sha_out: Vec<u8> = Vec::new();
    let mut entries = fs::read_dir(file_path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    entries.sort();

    for dir in entries {
        let mut mode;

        let path_name = dir.to_str().unwrap();
        //  println!("dir: {}", path_name);

        if path_name == "./.git" {
            continue;
        }
        let mut sha_file;
        if dir.is_dir() {
            mode = "40000".as_bytes().to_vec();
            (sha_file, _) = write_tree(&String::from_str(path_name).unwrap()).unwrap();
        } else
        /*if dir.is_file()*/
        {
            mode = "100644".as_bytes().to_vec();
            let file_data = fs::read(&path_name).unwrap();
            (sha_file, _) = write_hash_object(file_data, "blob").unwrap();
        }
      
        let mut dir_sha_out: Vec<u8> = Vec::new();
        dir_sha_out.append(&mut mode);
        dir_sha_out.push(' ' as u8);
        dir_sha_out.append(&mut dir.file_name().unwrap().to_str().unwrap().as_bytes().to_vec());
        dir_sha_out.push('\x00' as u8);
        dir_sha_out.append(&mut sha_file);

        sha_out.append(&mut dir_sha_out);  
    }
    let res = write_hash_object(sha_out, "tree");
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

    fs::create_dir(target_dir.clone() + "/.git/refs")?;

    fs::write(
        target_dir.to_owned() + "/.git/HEAD",
        "ref: refs/heads/master\n",
    )?;

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
    let data = format!("0032want {pack_hash}\n00000009done\n").to_string();

    //println!("0032 = {:#?}", &data);
    let client = reqwest::blocking::Client::new();
    //let data = data.as_bytes();
    let res = client.post(post_url).headers(headers).body(data);

    let res_send = res.send().unwrap();

    if !res_send.status().is_success() {
        println!("Something else happened. Status: {:?}", res_send.status());
    } else {
        println!("success!");
        let res_data = res_send.bytes().unwrap();
     
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
            //  println!("obj_type: {:?}", obj_type);
            while data_bytes[seek] > 128 {
                seek += 1;
            }
            seek += 1;
            // println!("seek : {:?}", seek);
            if obj_type < 7 {
                let mut git_data = ZlibDecoder::new(&data_bytes[seek..]);
                //  let decompressed = &data_bytes[seek..];

                //println!("git_data");
                let mut v_git_data = Vec::new();

                git_data.read_to_end(&mut v_git_data).unwrap();

                #[allow(unsafe_code)]
                let s_git_data = unsafe { String::from_utf8_unchecked(v_git_data) };

                let data_type = ["", "commit", "tree", "blob", "", "tag", "ofs_delta"];

                let mut obj_write_data =
                    format!("{} {}\0", data_type[obj_type], &s_git_data.len()).to_string();

                obj_write_data.push_str(&s_git_data);

                let mut hasher = Sha1::new();
                hasher.update(obj_write_data.as_bytes());

                let result = hasher.finalize();

                let hex_result = hex::encode(&result[..]);
                // println!("hex_result if: {:?}", hex_result);

                let f_path =
                    target_dir.to_owned() + &format!("/.git/objects/{}/", &hex_result[..2]);

                fs::create_dir_all(&f_path)?;

                let f_path = f_path + &hex_result[2..];

                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());

                e.write_all(obj_write_data.as_bytes())?;

                let compressed = e.finish()?;

                // println!(" f_path: {:?}", &f_path);
                fs::write(f_path, compressed)?;

                objs.insert(hex_result, (s_git_data.clone(), obj_type));

                seek += git_data.total_in() as usize;
                //seek += decompressed.len() as usize;
            } else {
                println!("else !!!!!!!!!!!!!!!!");

                let k = &data_bytes[seek..seek + 20];
                // println!("k data: {:#?}", k);
                let k = hex::encode(k);
                //  println!("k: {:#?}", k);
                let (base, elem_num) = objs[&k].to_owned();

                seek += 20;

                let mut delta = ZlibDecoder::new(&data_bytes[seek..]);
                // let decompressed = &data_bytes[seek..];

                let mut v_delta = Vec::new();
                delta.read_to_end(&mut v_delta).unwrap();

                let content = identify(&v_delta, base);
                obj_type = elem_num;
                //println!("content else: {:#?}", &content);
                // println!("obj_type else: {:#?}", &obj_type);

                let data_type = [
                    "",
                    "commit",
                    "tree",
                    "blob",
                    "",
                    "tag",
                    "ofs_delta",
                    "refs_delta",
                ];

                let mut obj_write_data =
                    format!("{} {}\0", data_type[obj_type], content.len()).to_string();

                //  println!("obj_write_data : {:?}", obj_write_data);

                obj_write_data += &content;
                //-----------------------

                let mut hasher = Sha1::new();
                hasher.update(obj_write_data.as_bytes());
                let result = hasher.finalize();

                let hex_result = hex::encode(&result[..]);
                //  println!("hex_result: {:?}", hex_result);

                let f_path = target_dir.to_owned() + &format!("/.git/objects/{}", &hex_result[..2]);

                let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
                e.write_all(obj_write_data.as_bytes())?;
                let compressed = e.finish().unwrap();

                fs::create_dir_all(&f_path).unwrap();
                let f_path = f_path + "/" + &hex_result[2..];
                //println!(" f_path: {:?}", &f_path);
                fs::write(f_path, &compressed).unwrap();

                // println!("objs k else: {:#?}", hex_result);
                objs.insert(hex_result, (content.clone(), obj_type));

                seek += delta.total_in() as usize;
                // seek += decompressed.len() as usize;
            }
        }
        let git_path = target_dir.to_owned()
            + &format!("/.git/objects/{}/{}", &pack_hash[..2], &pack_hash[2..]);

        let git_data = fs::read(git_path).unwrap();

        let mut data = ZlibDecoder::new(&git_data[0..]);

        let mut v_delta = Vec::new();

        data.read_to_end(&mut v_delta).unwrap();

        let s_delta = unsafe { String::from_utf8_unchecked(v_delta) };

        let mut data = s_delta.split("\n");

        let data = data.next().unwrap();
        let data = data.split(" ");
        // println!("data: {:?}", &data);
        let tree_sha = data.clone().nth(data.count() - 1).unwrap();

        println!("tree_sha: {}", &tree_sha);

        checkout_tree(
            tree_sha.to_owned(),
            target_dir.to_string(),
            target_dir.to_string(),
        );
    }
    //-2-------------------------------------------------------------------------------
    Ok(" ".to_owned())
}

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
    //content = "".to_string();
    println!("content: {:?}", &content);
    let delta_len = delta.len();
    // println!(" delta_len: {:?}", &delta_len);
    while seek < delta_len {
        let instr_byte = delta[seek];
        seek += 1;
        //  println!(" instr_byte: {:?}", &instr_byte);

        if instr_byte >= 128 {
            let offset_key = instr_byte & 0b00001111;

            let mut offset_bytes: [u8; 8] = [0; 8];

            for n in 0..8 {
                let b = offset_key >> n & 1;

                // println!("b offset_key: {}", b);
                if b == 1 {
                    offset_bytes[n] = delta[seek];
                    //  println!("offset_bytes delta[seek]:{}", delta[seek]);
                    seek += 1
   
                }
            }
            // println!("offset_bytes: {:?}", &offset_bytes);

            let offset = usize::from_le_bytes(offset_bytes);
            // println!("offset: {:?}", &offset);

            let len_key = (instr_byte & 0b01110000) >> 4;

            let mut len_bytes: [u8; 8] = [0; 8];
            for n in 0..8 {
                let b = len_key >> n & 1;

                //  println!("b len_key:{}", b);
                if b == 1 {
                    len_bytes[n] = delta[seek];
                    //  println!("len_bytes delta[seek]{}", delta[seek]);
                    seek += 1
                }
            }

            let len_int = usize::from_le_bytes(len_bytes);

            //  println!("len_int: {:?}", &len_int);
            content += &base[offset..(offset + len_int)];

            // println!("content : {:?}", &content );
        } else {
            println!("instr_byte:{}", instr_byte);
            let num_bytes = instr_byte & 0b01111111;
            println!("num_bytes u8:{}", num_bytes);
            let num_bytes = usize::from(num_bytes);

            println!("seek usize:{}", seek);
            content += &String::from_utf8_lossy(&delta[seek..(seek + num_bytes)]);

            seek += num_bytes;
        }
    }
    content
}

fn checkout_tree(sha: String, file_path: String, target_dir: String) {

    println!("target_dir: {target_dir}");
    println!("file_path: {file_path}");

    fs::create_dir_all(&file_path).unwrap();

    let git_data =
        fs::read(target_dir.clone() + &format!("/.git/objects/{}/{}", &sha[..2], &sha[2..]))
            .unwrap();
    let mut git_data = ZlibDecoder::new(&git_data[..]);

    let mut v_git_data = Vec::new();
    git_data.read_to_end(&mut v_git_data).unwrap();

    let mut enteries = Vec::new();
    //println!("enteries: {:#?}", &s_git_data);

    let pos = v_git_data.iter().position(|&r| r == '\x00' as u8).unwrap();

    let mut tree = &v_git_data[pos + 1..];

    while tree.len() > 0 {
        let pos = tree.iter().position(|&r| r == '\x00' as u8).unwrap();

        println!("position: {:#?}", &pos);

        let mode_name = &tree[..pos];

        let mut mode_name = mode_name.split(|&num| num == ' ' as u8);

        //println!("mode_name: {:#?}", &mode_name);

        let mode = mode_name.next().unwrap();
        let name = mode_name.next().unwrap();

        tree = &tree[pos + 1..];

        let sha = &tree[..20];

        tree = &tree[20..];

        //println!("tree: {:#?}", &tree);

        let sha = hex::encode(&sha[..]);
        let mode = String::from_utf8_lossy(mode);
        let name = String::from_utf8_lossy(name);

        println!("mode: {:#?}", &mode);
        println!("name: {:#?}", &name);
        println!("sha: {:#?}", &sha);

        enteries.push((mode.clone(), name.clone(), sha.clone()));
    }

    for entry in enteries {
        if entry.0 == "40000" {
            println!("blob_sha 40000: {:#?}", &entry.1);
            checkout_tree(
                entry.2.to_string(),
                file_path.clone() + &format!("/{}", entry.1).to_string(),
                target_dir.to_string(),
            );
        } else {
            let blob_sha = entry.2;

            println!("blob_sha: {}", &blob_sha);

            let curr_dir = target_dir.clone()
                + &format!("/.git/objects/{}/{}", &blob_sha[..2], &blob_sha[2..]);

            println!("curr_dir: {}", &curr_dir);

            let git_data = fs::read(curr_dir).unwrap();

            let mut git_data = ZlibDecoder::new(&git_data[..]);

            let mut v_git_data = Vec::new();
            git_data.read_to_end(&mut v_git_data).unwrap();

            let pos = v_git_data.iter().position(|&r| r == '\x00' as u8).unwrap();

            let content = &v_git_data[pos + 1..];

            fs::write(file_path.clone() + &format!("/{}", entry.1), content).unwrap();
        }
    }
}