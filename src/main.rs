#[allow(unused_imports)]
use anyhow::{Context, Result};
use git_starter_rust::cli::{Cli, Commands, CreateBlobOptions, ReadBlobOptions, ReadTreeOptions, CommitTreeOptions, CloneRepOptions};
use git_starter_rust::*;
//use std::env;
use clap::Parser;
use std::fs;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init => {
            println!("Init--------------------------------");
            git_init()?;
        }
        Commands::CatFile(read_options) => {
            // println!("read-------------------------------");
            print!("{}", read_git_object(ReadBlobOptions::read(&read_options)?)?)
        }
        Commands::HashObject(file) => {
            println!("create-------------------------------");
            let file_data = fs::read(CreateBlobOptions::read(file)?)?;
            let sha1_out = write_git_object(&file_data, "blob")?;
            println!("hash-object in: {}", sha1_out);
        }
        Commands::LsTree(hash) => {
            //  println!("read tree-------------------------------");
            let result = read_tree(ReadTreeOptions::read(&hash)?)?;
            for s in result {
                println!("{}", String::from_utf8(s)?);
            }
        }
        Commands::WriteTree => {
            let sha1_out = write_tree(&".".to_string())?;
            print!("{}", sha1_out);
        }
        Commands::CommitTree(args) => {
           // println!("commit tree-------------------------------");
         
            print!("{}", create_commit(CommitTreeOptions::read(&args)?)?);           
        }
        Commands::Clone(args) => {
            //  println!("clone-------------------------------");

            if let Err(err) = clone_repo(CloneRepOptions::read(&args)?){
                eprintln!("ERROR in clone repo: {}", err);
                err.chain().skip(1).for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }

        } 
    }
    Ok(())
}

/****************************************************************************************************************
 * **************************************************************************************************************
*/
