use flate2::read::ZlibDecoder;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;
use std::collections::HashMap;
use std::rc::Rc;

use crate::*;
#[allow(unused_imports)]
use anyhow::{Context, Result};
use std::fs;
use std::io;
//use std::io::prelude::*;
use std::str;

/************************************************************************************************************************** */
pub fn clone_repo((url, target_dir): (&str, &str)) -> Result<()> {
    create_dirs(&target_dir)?;
    let target_dir_git_dir = target_dir.to_owned() + "/.git/objects/";

    //------------------------------------------------------------------------------------
    let url_adr = url.to_owned() + &"/info/refs?service=git-upload-pack";
    let pack_hash = get_pack_hash(&url_adr)?;
    //----------------------------------------------------------------------------------
    let post_url = url.to_owned() + &"/git-upload-pack";

    let data = format!("0032want {pack_hash}\n00000009done\n");

    let res_data = post_to_git_data(&post_url, &data)?;

    //---------------------------------------------------------------------------------------

    let res_data_size = res_data.len() - 20;

    println!("res_data_size: {:?}", res_data_size);

    let entries_bytes = res_data[16..20].try_into()?;

    //  println!("entries_bytes: {:#?}", entries_bytes);
    let num = u32::from_be_bytes(entries_bytes);
    println!("num: {:?}", num);
    let data_bytes: Vec<u8> = res_data[20..res_data_size].try_into()?;
    // println!("data_bytes: {:?}", data_bytes);

    let mut objs = HashMap::new();
    let mut seek = 0;
   // let mut objs_count = 0;

    for _ in 0..num {
       // objs_count += 1;
        let first = data_bytes[seek];
        let mut obj_type: usize = ((first & 112) >> 4).into();
        //  println!("obj_type: {:?}", obj_type);
        while data_bytes[seek] > 128 {
            seek += 1;
        }
        seek += 1;
        // println!("seek : {:?}", seek);
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

        if obj_type < 7 {
            let mut git_data = ZlibDecoder::new(&data_bytes[seek..]);
            let mut v_git_data = Vec::new();
            git_data.read_to_end(&mut v_git_data)?;

            let hex_result =
                write_git_object_target_dir(data_type[obj_type], &v_git_data, &target_dir_git_dir)?;

            objs.insert(hex_result, (v_git_data, obj_type));

            seek += git_data.total_in() as usize;
        } else {
            let k = &data_bytes[seek..seek + 20];
            // println!("k data: {:#?}", k);
            let k = hex::encode(k);
            //  println!("k: {:#?}", k);
            let (base, elem_num) = objs[&k].to_owned();

            seek += 20;

            let mut delta = ZlibDecoder::new(&data_bytes[seek..]);
            let mut v_delta = Vec::new();
            delta.read_to_end(&mut v_delta)?;

            let content = identify(&v_delta, &base)?;
            obj_type = elem_num;
            //println!("content else: {:#?}", &content);
            // println!("obj_type else: {:#?}", &obj_type);
            let hex_result =
                write_git_object_target_dir(data_type[obj_type], &content, &target_dir_git_dir)?;
            // println!("objs k else: {:#?}", hex_result);
            objs.insert(hex_result, (content, obj_type));

            seek += delta.total_in() as usize;
        }
    }
    let git_path =
        target_dir_git_dir.to_owned() + &format!("{}/{}", &pack_hash[..2], &pack_hash[2..]);

    let git_data = fs::read(git_path)?;
    let v_delta = zlib_decode(&git_data[..].to_vec())?;

    // let s_delta = unsafe { &String::from_utf8_unchecked(v_delta.to_vec()) };
    let data = v_delta
        .split(|b| *b == '\n' as u8)
        .next()
        .unwrap()
        .split(|b| *b == ' ' as u8);
    let tree_sha = data.clone().nth(data.count() - 1).unwrap();
    // println!("tree_sha: {:?}", &tree_sha);
    let tree_sha = String::from_utf8_lossy(tree_sha);
    checkout_tree(&tree_sha, &target_dir, &target_dir_git_dir)?;

    Ok(())
}

/************************************************************************************************************************ */
fn create_dirs(target_dir: &str) -> Result<(), io::Error> {
    fs::create_dir(&target_dir)?;

    fs::create_dir(target_dir.to_owned() + "/.git")?;

    fs::create_dir(target_dir.to_owned() + "/.git/objects/")?;

    fs::create_dir(target_dir.to_owned() + "/.git/refs")?;

    fs::write(
        target_dir.to_owned() + "/.git/HEAD",
        "ref: refs/heads/master\n",
    )?;

    Ok(())
}
fn get_pack_hash(url: &str) -> Result<String> {
    let body = reqwest::blocking::get(url)?.text()?;

    //println!("body = {:#?}", body);

    let content = body
        .split("\n")
        .filter(|c| c.contains("refs/heads/master") && c.contains("003f"))
        .collect::<String>();
    let content = content.split(" ").next().ok_or(anyhow!("Data not found"))?;
    if content.len() != 44 {
        return Err(anyhow!("Data is not a sha"));
    }
    let pack_hash = String::from(&content[4..]);
    println!("pack_hash = {}", pack_hash);
    Ok(pack_hash)
}

/**************************************************************************************************************** */

fn post_to_git_data(url: &str, data: &str) -> Result<bytes::Bytes> {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-git-upload-pack-request"),
    );
    // println!("url = {:#?}", &url);
    // println!("0032 = {:#?}", &data);
    // println!("headers = {:#?}", &headers);
    let client = reqwest::blocking::Client::new();

    let res = client.post(url).headers(headers).body(String::from(data));
    // println!("res = {:#?}", &res);
    let res_send = res.send()?;
    // println!(" res_send = {:#?}", &res_send);
    if !res_send.status().is_success() {
        return Err(anyhow!(
            "Something happened with Response. Status: {:?}",
            res_send.status()
        ));
    }

    println!("success!");

    let res_data = res_send.bytes()?;

    Ok(res_data)
}

//************************************************************************************************************************* */
fn identify(delta: &[u8], base: &[u8]) -> Result<Vec<u8>, io::Error> {
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
    let mut content = Vec::new();
    //content = "".to_string();
    //  println!("content: {:?}", &content);
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
            content.extend_from_slice(&base[offset..(offset + len_int)]);

            // println!("content : {:?}", &content );
        } else {
            // println!("instr_byte:{}", instr_byte);
            let num_bytes = instr_byte & 0b01111111;
            //  println!("num_bytes u8:{}", num_bytes);
            let num_bytes = usize::from(num_bytes);

            // println!("seek usize:{}", seek);
            content.extend_from_slice(&delta[seek..(seek + num_bytes)]);

            seek += num_bytes;
        }
    }
    Ok(content)
}
/*************************************************************************************************************************** */
fn checkout_tree(sha: &str, file_path: &str, target_dir: &str) -> Result<(), std::io::Error> {
    println!("target_dir: {target_dir}");
    println!("file_path: {file_path}");
    let target_dir = Rc::new(target_dir);

    fs::create_dir_all(&file_path)?;

    let git_data = fs::read(target_dir.to_string() + &format!("{}/{}", &sha[..2], &sha[2..]))?;

    let v_git_data = zlib_decode(&git_data[..].to_vec())?;

    let mut enteries = Vec::new();

    let pos = v_git_data.iter().position(|&r| r == '\x00' as u8).unwrap();

    let mut tree = &v_git_data[pos + 1..];

    while tree.len() > 0 {
        let pos = tree.iter().position(|&r| r == '\x00' as u8).unwrap();

        // println!("position: {:#?}", &pos);

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

        // println!("mode: {:#?}", &mode);
        // println!("name: {:#?}", &name);
        //println!("sha: {:#?}", &sha);

        enteries.push((mode.clone(), name.clone(), sha.clone()));
    }

    for entry in enteries {
        if entry.0 == "40000" {
            //  println!("blob_sha 40000: {:#?}", &entry.1);
            checkout_tree(
                &entry.2,
                &(file_path.to_owned() + &format!("/{}", entry.1)),
                &target_dir,
            )?;
        } else {
            let blob_sha = entry.2;

            // println!("blob_sha: {}", &blob_sha);

            let curr_dir =
                target_dir.to_string() + &format!("{}/{}", &blob_sha[..2], &blob_sha[2..]);

            // println!("curr_dir: {}", &curr_dir);

            let git_data = fs::read(curr_dir)?;
            let v_git_data = zlib_decode(&git_data)?;

            let pos = v_git_data.iter().position(|&r| r == '\x00' as u8).unwrap();

            let content = &v_git_data[pos + 1..];

            fs::write(file_path.to_owned() + &format!("/{}", entry.1), content)?;
        }
    }
    Ok(())
}
/***************************************************************************************************************
 * **************************************************************************************************************
*/
