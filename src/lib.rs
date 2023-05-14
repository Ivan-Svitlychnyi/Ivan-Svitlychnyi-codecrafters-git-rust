use bytes::BufMut;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};

#[allow(unused_imports)]
use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::stdout;
use std::str;
use std::str::FromStr;

pub mod cli;
pub mod clone;
/******************************************************************************************************** */
pub fn git_init() -> Result<()> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/master\n")?;

    println!("Initialized git directory");
    Ok(())
}

/********************************************************************************************************* */
pub fn zlib_decode(enc_data: &Vec<u8>) -> Result<Vec<u8>, io::Error> {
    let mut enc_data = ZlibDecoder::new(&enc_data[..]);
    let mut dec_data = Vec::new();
    enc_data.read_to_end(&mut dec_data)?;

    Ok(dec_data)
}

/********************************************************************************************************** */
pub fn zlib_encode(data: &Vec<u8>) -> Result<Vec<u8>, io::Error> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&data[..])?;
    let compressed = e.finish()?;

    Ok(compressed)
}

/*********************************************************************************************************** */
pub fn make_hash(data: &Vec<u8>) -> Result<String, io::Error> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    let hex_result = hex::encode(&result[..]);
    Ok(hex_result)
}

//********************************************************************************************************** */
pub fn read_git_object(hash: &str) -> Result<()> {
    let full_path = format!(".git/objects/{}/{}", &hash[..2], &hash[2..]);
    let git_data = zlib_decode(&fs::read(full_path)?)?;
    // println!("git_data = {:#?}", &git_data);
    let git_data: Vec<&[u8]> = git_data[..].split(|c| *c == '/' as u8).collect();
    let git_data = git_data[git_data.len() - 1];
    let data_pos = git_data
        .iter()
        .position(|&r| r == '\x00' as u8)
        .unwrap_or(0);
    // println!("git_data_fin = {:#?}", &git_data);

    let git_data = &git_data[data_pos + 1..];
    stdout().write_all(git_data)?;

    Ok(())
}
/************************************************************************************************************* */
pub fn write_git_object(file_data: &Vec<u8>, file_type: &str) -> Result<String, io::Error> {
    /******************************************** */
    let mut store: Vec<u8> = Vec::new();
    store.put(file_type[..].as_bytes());
    store.put_u8(' ' as u8);
    store.put(file_data.len().to_string().as_bytes());
    store.put_u8('\x00' as u8);
    store.put(file_data.as_slice());
    /******************************************* */
    //println!("store_vec = {:#?}", &store_vec);

    let compressed = zlib_encode(&store)?;

    let hex_result = make_hash(&store)?;

    let sub_dir_path = format!(".git/objects/{}/", &hex_result[..2]);

    let full_path = format!("{sub_dir_path}{}", &hex_result[2..]);

    fs::create_dir_all(sub_dir_path)?;
    fs::write(full_path, compressed)?;

    Ok(hex_result)
}

/*************************************************************************************************************** */
pub fn read_tree(file_path: &str) -> Result<Vec<Vec<u8>>, io::Error> {
    const HASH_BYTES: usize = 20;

    let (sub_dir, sha_num) = (&file_path[..2], &file_path[2..]);

    let full_path = format!(".git/objects/{sub_dir}/{sha_num}");

    let mut file_content = zlib_decode(&fs::read(&full_path)?)?;

    let mut result: Vec<Vec<u8>> = Vec::new();
    let mut start_byte = 0;
    loop {
        if let Some(pos) = file_content[..].iter().position(|&r| r == '\x00' as u8) {
            let mut data_pos = file_content[start_byte..pos].split(|&r| r == ' ' as u8);
            if data_pos.next().ne(&Some("tree".as_bytes())) {
                result.push(data_pos.clone().last().unwrap().to_vec());
                // println!("result = {:#?}", String::from_utf8(result.last().unwrap().to_vec()));
                // println!("file_content = {:#?}", &String::from_utf8_lossy(&file_content[..]));
                start_byte = HASH_BYTES;
            }
            file_content = file_content[pos + 1..].to_vec();

        } else {
            break;
        }
    }

    Ok(result)
}
/******************************************************************************************************************* */
pub fn write_tree(file_path: &str) -> Result<String> {
    // let mut sha_out: String = "".to_string();
    let mut sha_out: Vec<u8> = Vec::new();
    let mut entries = fs::read_dir(file_path)?
        .map(|res| res.map(|e| e.path()))
        .filter(|path| path.as_ref().unwrap().clone().to_str().unwrap() != "./.git")
        .collect::<Result<Vec<_>, io::Error>>()?;

    entries.sort();

    for dir in entries {
        let mode;

        let path_name = dir.to_str().unwrap();
        //  println!("dir: {}", path_name);

        // if path_name == "./.git" {
        //     continue;
        // }
        let mut sha_file;
        if dir.is_dir() {
            mode = "40000";
            sha_file = hex::decode(write_tree(&String::from_str(path_name)?)?)?;
        } else
        /*if dir.is_file()*/
        {
            mode = "100644";
            let file_data = fs::read(&path_name)?;
            sha_file = hex::decode(write_git_object(&file_data, "blob")?)?;
        }

        let mut dir_sha_out: Vec<u8> = Vec::new();
        dir_sha_out.append(&mut mode.as_bytes().to_vec());
        dir_sha_out.push(' ' as u8);
        dir_sha_out.append(
            &mut dir
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .as_bytes()
                .to_vec(),
        );
        dir_sha_out.push('\x00' as u8);
        dir_sha_out.append(&mut sha_file);

        sha_out.append(&mut dir_sha_out);
    }
    let res = write_git_object(&mut &sha_out, "tree")?;

    Ok(res)
}

/************************************************************************************************************* */
pub fn create_commit(
    (tree_sha, parent_commit_sha, data): (&str, &str, &str),
) -> Result<String, io::Error> {
    // let (tree_sha, parent_commit_sha, data) = (hash, print, message);

    let user_metadata = "author Admin <admin@example.com> 1652217488 +0300\ncommitter Name <committer@example.com> 1652224514 +0300".to_string();

    let content =
        format!("tree {tree_sha}\nparent {parent_commit_sha}\n{user_metadata}\n\n{data}\n");

    //println!("content: {:?}", &content);
    let sha = write_git_object(&mut content.into_bytes(), "commit")?;

    Ok(sha)
}

/*************************************************************************************************************** */
