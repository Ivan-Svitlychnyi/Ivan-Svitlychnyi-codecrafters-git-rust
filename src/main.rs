#[allow(unused_imports)]
use anyhow::{Context, Result};
use git_starter_rust::*;
use git_starter_rust::cli::{Commands, Cli, CreateBlobOptions};
//use std::env;
use std::fs;
use clap::Parser;





fn main() ->Result<()>{
    let cli = Cli::parse();  
    match &cli.command {
        Commands::Init => {
            println!("Init--------------------------------");
            git_init()?;
        }
        Commands::CatFile(read_options)=> {
       // println!("read-------------------------------");
            print!(
                 "{}",
                read_git_object(&read_options)?)  
        }
        Commands::HashObject(file)=> {
            println!("create-------------------------------");
        //let file = file  
        let file_data = fs::read(CreateBlobOptions::get_args(file))?;
        let sha1_out = write_git_object(&file_data, "blob")?;
        println!("hash-object in: {}", sha1_out);
            }
        Commands::LsTree(hash)=> {
          //  println!("read tree-------------------------------");
            let result = read_tree(&hash)?;
                for s in result {
                    println!("{}", String::from_utf8(s)?);
                }
            }
       Commands::WriteTree =>{
        
        let sha1_out = write_tree(&".".to_string())?;
        print!("{}", sha1_out);
       }
       Commands::CommitTree(args)=> {
        //  println!("commit tree-------------------------------");
          print!("{}", create_commit(&args)?);
          }

    //    _=> {
    //     panic!("enter the arguments!");
    //    }
    }



    // let args: Vec<String> = env::args().collect();
    // if args.is_empty() {
    //     panic!("enter the arguments!");
    // }
    

    // //-----------------------------------------------------------------------------------------------------
    // // if args[1] == "init" {
    // //     //println!("enter the arguments init: {:?}", &args);
    // //     git_init()?;
    //     // println!("{}", )
    // //------------------------------------------------------------------------------------------------------
    // if args[1] == "cat-file" && args[2] == "-p" {
    //    // println!("enter the arguments cat-file: {:?}", &args);
    //     // print!(
    //     //     "{}",
    //     //     String::from_utf8(read_git_object(&args[3])?)?
    //     // );
    //     print!(
    //         "{}",
    //        read_git_object(&args[3])?)
    //     ;

    // //-------------------------------------------------------------------------------------------------------
    // } else if args[1] == "hash-object" && args[2] == "-w" {
    //     //println!("enter the arguments hash-object: {:?}", &args);
    //     let file_data = fs::read(args[3].to_string())?;
    //     let sha1_out = write_hash_object(&file_data, "blob")?;

    //     println!("hash-object in: {}", sha1_out);
    // //--------------------------------------------------------------------------------------------------------
    // } else if args[1] == "ls-tree" && args[2] == "--name-only" {
    //     //println!("enter the arguments ls-tree: {:?}", &args);
    //     let result = read_tree(&args[3])?;

    //     for s in result {
    //         println!("{}", String::from_utf8(s)?);
    //     }
    // //--------------------------------------------------------------------------------------------------------
    // } else if args[1] == "write-tree" {
    //    // println!("enter the arguments write-tree: {:?}", &args);
    //     let sha1_out = write_tree(&".".to_string())?;

    //     print!("{}", sha1_out);
    // //---------------------------------------------------------------------------------------------------------
    // } else if args[1] == "commit-tree" {
    //    // println!("enter the arguments commit-tree: {:?}", &args);
    //     print!("{}", create_commit(&args)?);
    // //--------------------------------------------------------------------------------------------------------
    // } else if args[1] == "clone" {
    //     println!("enter the arguments clone: {:?}", &args);
    //     clone_repo(&args)?;
    // //---------------------------------------------------------------------------------------------------------
    // } else {
    //     println!("unknown command: {:#?}", args)
    // }
    Ok(())
}

/****************************************************************************************************************
 * **************************************************************************************************************
*/
