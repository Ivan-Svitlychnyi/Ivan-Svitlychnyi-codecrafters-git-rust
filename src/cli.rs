use clap::{Parser, Subcommand};
use thiserror::Error;
use clap::{arg, Args};
use std::{iter::zip, str::FromStr};




#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]

pub struct Cli {
    #[command(subcommand)]

    pub command: Commands,

}

#[derive(Subcommand)]
pub enum Commands {
/// git init
/// ["/tmp/codecrafters-git-target/release/git-starter-rust", "init"]
Init,
//cat-file: ["/tmp/codecrafters-git-target/release/git-starter-rust", "cat-file", "-p", "8a68edea4924829fe663c18dfd9b2ffb3b773e65"]
//hash-object: ["/tmp/codecrafters-git-target/release/git-starter-rust", "hash-object", "-w", "dooby.txt"]
//ls-tree: ["/tmp/codecrafters-git-target/release/git-starter-rust", "ls-tree", "--name-only", "f2d1f407ac46465b8107db3f2671b97d191cbfa8"]
// write-tree: ["/tmp/codecrafters-git-target/release/git-starter-rust", "write-tree"]




}

