#[allow(unused_imports)]
use anyhow::{Context, Result};
use git_starter_rust::cli::{Cli, Commands, CreateBlobOptions, ReadBlobOptions, ReadTreeOptions, CommitTreeOptions, CloneRepOptions};
use git_starter_rust::*;
//use std::env;
use clap::Parser;
use git_starter_rust::clone::{clone_repo};
use std::fs;
use std::io::{stdout, Write};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Init => {
           // println!("Init--------------------------------");
          //  if let Err(err) = git_init() {
            //    eprintln!("ERROR in Init operation: {}", err);
             //   std::process::exit(1);
           // }
            
             git_init()?;
        }
        Commands::CatFile(read_options) => {
            // println!("read-------------------------------");
           read_git_object(ReadBlobOptions::read(read_options)?)?;
        }
        Commands::HashObject(file) => {
            println!("create-------------------------------");
            let file_data = fs::read(CreateBlobOptions::read(file)?)?;
            let sha1_out = write_git_object_target_dir("blob", &file_data,".git/objects/")?;
           // let sha1_out = write_git_object(&file_data, "blob")?;
            println!("hash-object in: {}", sha1_out);
        }
        Commands::LsTree(hash) => {
            //  println!("read tree-------------------------------");
            let result = read_tree(ReadTreeOptions::read(hash)?)?;
            for s in result {
                //println!("{}", String::from_utf8(s)?);
                stdout().write_all(s.as_slice())?;
                stdout().write_all(&[b'\n'])?;
            }
        }
        Commands::WriteTree => {
            let sha1_out = write_tree(&PathBuf::from("."))?;
            print!("{}", sha1_out);
        }
        Commands::CommitTree(args) => {
           // println!("commit tree-------------------------------");
         
            print!("{}", create_commit(CommitTreeOptions::read(args)?)?);           
        }
        Commands::Clone(args) => {
            //  println!("clone-------------------------------");

            clone_repo(CloneRepOptions::read(args)?)?;

        } 
    }
    Ok(())
}

/****************************************************************************************************************
 * **************************************************************************************************************
*/
