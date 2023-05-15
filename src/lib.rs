#[allow(unused_imports)]
use anyhow::{anyhow, Error, Result as AnyResult};
#[allow(unused_imports)]
use anyhow::{Context, Result};
use bytes::BufMut;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::stdout;
use std::path::PathBuf;
use std::str;
//use std::str::FromStr;

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
pub fn write_git_object_target_dir(
    data_type: &str,
    content: &Vec<u8>,
    target_dir: &str,
) -> Result<String, io::Error> {
    let mut obj_write_data: Vec<u8> = Vec::new();
    obj_write_data.put(data_type[..].as_bytes());
    obj_write_data.put_u8(' ' as u8);
    obj_write_data.put(content.len().to_string().as_bytes());
    obj_write_data.put_u8('\0' as u8);
    obj_write_data.put(content.as_slice());
    //-----------------------
    let hex_result = make_hash(&obj_write_data)?;
    // //  println!("hex_result: {:?}", hex_result);
    let compressed = zlib_encode(&obj_write_data)?;

    let f_path = target_dir.to_owned() + &format!("{}/", &hex_result[..2]);

    fs::create_dir_all(&f_path)?;
    let f_path = f_path + &hex_result[2..];
    //println!(" f_path: {:?}", &f_path);
    fs::write(f_path, &compressed)?;

    Ok(hex_result)
}

/*************************************************************************************************************** */
pub fn read_tree(file_path: &str) -> Result<Vec<Vec<u8>>> {
    const HASH_BYTES: usize = 20;

    let full_path = format!(".git/objects/{}/{}", &file_path[..2], &file_path[2..]);

    let mut file_content = zlib_decode(&fs::read(&full_path)?)?;

    let mut result: Vec<Vec<u8>> = Vec::new();

    //println!("file_content in = {:#?}", &String::from_utf8_lossy(&file_content[..]));

    if let Some(pos) = file_content[..].iter().position(|&r| r == '\x00' as u8) {
        let mut data_pos = file_content[..pos].split(|&r| r == ' ' as u8);

        if data_pos.next().ne(&Some("tree".as_bytes())) {
            return Err(anyhow!("Not tree object"));
        }
        file_content = file_content[pos + 1..].to_vec();
    } else {
        return Err(anyhow!("Do not posible to split data"));
    }
    loop {
        if let Some(pos) = file_content[..].iter().position(|&r| r == '\x00' as u8) {
            let data_pos = file_content[..pos].split(|&r| r == ' ' as u8);
            result.push(data_pos.clone().last().unwrap().to_vec());
            // println!("file_content = {:#?}", &String::from_utf8_lossy(&file_content[..]));
            file_content = file_content[pos + 1 + HASH_BYTES..].to_vec();
        } else {
            break;
        }
    }

    return Ok(result);
}
/******************************************************************************************************************* */
pub fn write_tree(file_path: &PathBuf) -> Result<String> {
    // let mut sha_out: String = "".to_string();
    let mut sha_out: Vec<u8> = Vec::new();
    let mut entries = fs::read_dir(file_path)?
        .map(|res| res.map(|e| e.path()))
        .filter(|path| path.as_ref().unwrap().clone().to_str().unwrap() != "./.git")
        .collect::<Result<Vec<_>, io::Error>>()?;

    entries.sort();

    for dir in entries {
        #[allow(unused_assignments)]
        let mut mode = "";
        #[allow(unused_assignments)]
        let mut sha_file = Vec::new();
        if dir.is_dir() {
            mode = "40000";
            sha_file = hex::decode(write_tree(&dir)?)?;
        } else if dir.is_file()
        /*if dir.is_file()*/
        {
            mode = "100644";
            let file_data = fs::read(&dir)?;
            sha_file = hex::decode(write_git_object_target_dir(
                "blob",
                &file_data,
                ".git/objects/",
            )?)?;
        } else {
            return Err(anyhow!("This is not relevant path"));
        }

        let mut dir_sha_out: Vec<u8> = Vec::new();
        dir_sha_out.extend_from_slice(&mode.as_bytes());
        dir_sha_out.push(' ' as u8);
        dir_sha_out.extend_from_slice(
            &dir.file_name()
                .ok_or(anyhow!("file name is not valid"))?
                .to_str()
                .ok_or(anyhow!("file name did not convert to str"))?
                .as_bytes(),
        );
        dir_sha_out.push('\x00' as u8);
        dir_sha_out.extend_from_slice(&sha_file);
        sha_out.extend_from_slice(&dir_sha_out);
    }
    let res = write_git_object_target_dir("tree", &sha_out, ".git/objects/")?;

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
    let sha = write_git_object_target_dir("commit", &mut content.into_bytes(), ".git/objects/")?;

    Ok(sha)
}

/*************************************************************************************************************** */
