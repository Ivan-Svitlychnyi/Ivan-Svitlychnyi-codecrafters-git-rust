#[allow(unused_imports)]
use anyhow::{Context, Result};
use git_starter_rust::*;
use std::env;
use std::fs;






fn main() ->Result<()>{
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        panic!("enter the arguments!");
    }
    //-----------------------------------------------------------------------------------------------------
    if args[1] == "init" {
        git_init()?;
        // println!("{}", )
    //------------------------------------------------------------------------------------------------------
    } else if args[1] == "cat-file" && args[2] == "-p" {
        print!(
            "{}",
            String::from_utf8(read_git_object(&args[3])?)?
        );
    //-------------------------------------------------------------------------------------------------------
    } else if args[1] == "hash-object" && args[2] == "-w" {
        let file_data = fs::read(args[3].to_string())?;
        let (_, sha1_out) = write_hash_object(&file_data, "blob")?;

        println!("hash-object in: {}", sha1_out);
    //--------------------------------------------------------------------------------------------------------
    } else if args[1] == "ls-tree" && args[2] == "--name-only" {
        let result = read_tree(&args[3])?;

        for s in result {
            println!("{}", String::from_utf8(s)?);
        }
    //--------------------------------------------------------------------------------------------------------
    } else if args[1] == "write-tree" {
        let (_, sha1_out) = write_tree(&".".to_string())?;

        print!("{}", sha1_out);
    //---------------------------------------------------------------------------------------------------------
    } else if args[1] == "commit-tree" {
        print!("{}", create_commit(&args)?);
    //--------------------------------------------------------------------------------------------------------
    } else if args[1] == "clone" {
        clone_repo(&args)?;
    //---------------------------------------------------------------------------------------------------------
    } else {
        println!("unknown command: {:#?}", args)
    }
    Ok(())
}

/****************************************************************************************************************
 * **************************************************************************************************************
*/
