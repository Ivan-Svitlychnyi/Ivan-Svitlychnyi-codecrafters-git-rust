use clap::{Parser, Subcommand};
//use thiserror::Error;
use clap::{arg, Args};
//use std::{iter::zip, str::FromStr};




#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]

pub struct Cli {
    #[command(subcommand)]

    pub command: Commands,

}
//["/tmp/codecrafters-git-target/release/git-starter-rust", "init"]

/*cat-file: ["/tmp/codecrafters-git-target/release/git-starter-rust", "cat-file", "-p", "8a68edea4924829fe663c18dfd9b2ffb3b773e65"]
----------------------------------------------------------------------------------------------------------------------------------------------------------------------------
hash-object: ["/tmp/codecrafters-git-target/release/git-starter-rust", "hash-object", "-w", "dooby.txt"]
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
ls-tree: ["/tmp/codecrafters-git-target/release/git-starter-rust", "ls-tree", "--name-only", "f2d1f407ac46465b8107db3f2671b97d191cbfa8"]
----------------------------------------------------------------------------------------------------------------------------------------------------------------------------
write-tree: ["/tmp/codecrafters-git-target/release/git-starter-rust", "write-tree"]
----------------------------------------------------------------------------------------------------------------------------------------------------------------------------
commit-tree: ["/tmp/codecrafters-git-target/release/git-starter-rust", "commit-tree", "a5023aefc54252953ae544d75d9e7073ca4a33ca", "-p",
 "e82b99fbf78f96399ef39c37d87edeebefdd2a0e", "-m", "horsey dumpty monkey vanilla yikes dooby"]
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
clone: ["/tmp/codecrafters-git-target/release/git-starter-rust", "clone", "https://github.com/codecrafters-io/git-sample-1", "test_dir"]
*/
#[derive(Subcommand)]
pub enum Commands {
/// git init
Init,
///Read a blob object
CatFile(ReadBlobOptions), 
///Create a blob object
HashObject(CreateBlobOptions),
///Read a tree object
LsTree(ReadTreeOptions),
///Write a tree object
WriteTree,
///Create a commit
CommitTree(CommitTreeOptions),
///Clone a repository
Clone(CloneRepOptions),
}
#[derive(Args)]
pub struct ReadBlobOptions {
    /// print
    #[arg(short = 'p')]
    pub print: Option<String>,
    
}
#[derive(Args)]
pub struct CreateBlobOptions {
    //Create a blob object
    #[arg(short = 'w')]
    pub file: Option<String>,   
}
impl CreateBlobOptions{
pub fn get_args(&self)-> &str{
  let file = self.file.as_deref().unwrap();
  file  
}

}
#[derive(Args)]
pub struct ReadTreeOptions {
    //Create a blob object
    #[arg(long = "name-only")]
    pub hash: Option<String>,   
}
#[derive(Args)]
pub struct CommitTreeOptions{
    pub hash: Option<String>,
    #[arg(short = 'p')]
    pub print: Option<String>,
    #[arg(short = 'm')]
    pub message: Option<String>,
}
#[derive(Args)]
pub struct CloneRepOptions {

    pub url: Option<String>,
    pub dir: Option<String>,
    
}