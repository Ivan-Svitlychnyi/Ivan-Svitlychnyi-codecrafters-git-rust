
use clap::{Parser, Subcommand};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
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
/*init ["/tmp/codecrafters-git-target/release/git-starter-rust", "init"]
--------------------------------------------------------------------------------------------------------------------------------------------------------------------------
cat-file: ["/tmp/codecrafters-git-target/release/git-starter-rust", "cat-file", "-p", "8a68edea4924829fe663c18dfd9b2ffb3b773e65"]
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
    ///Git init
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


//---Read a blob object args handling-----------------------------------------------------------------
#[derive(Args)]
pub struct ReadBlobOptions {
    /// print
    #[arg(short = 'p')]
    blob_sha: Option<String>,
}
impl ReadBlobOptions {
    pub fn read(&self) -> Result<&str, ArgsReadError> {
        if let Some(blob_sha) = self.blob_sha.as_deref() {
            print!("blob_sha.len(): {}", blob_sha.len());  
            return Ok(&blob_sha);
        }
        Err(ArgsReadError::ReadBlobCommandError)
    }

}

//-------------------------------------------------------------------------------------------------
//---Create a blob object args handling------------------------------------------------------------
#[derive(Args)]
pub struct CreateBlobOptions {
    //Create a blob object
    #[arg(short = 'w')]
    blob_sha: Option<String>,
}

impl CreateBlobOptions {
    pub fn read(&self) -> Result<&str, ArgsReadError> {
        if let Some(blob_sha) = self.blob_sha.as_deref() {
            return Ok(&blob_sha);
        }
        Err(ArgsReadError::CreateBlobCommandError)
    }
}
//---------------------------------------------------------------------------------------------
//---Read a tree object args handling----------------------------------------------------------
#[derive(Args)]
pub struct ReadTreeOptions {
    //Create a blob object
    #[arg(long = "name-only")]
    tree_sha: Option<String>,
}

impl ReadTreeOptions {
    pub fn read(&self) -> Result<&str, ArgsReadError> {
        if let Some(tree_sha) = self.tree_sha.as_deref() {
            return Ok(&tree_sha);
        }
        Err(ArgsReadError::ReadTreeCommandError)
    }
}
//-------------------------------------------------------------------------------------------------
//---Create a commit-------------------------------------------------------------------------------
#[derive(Args)]
pub struct CommitTreeOptions {
    tree_sha: Option<String>,
    #[arg(short = 'p')]
    commit_sha: Option<String>,
    #[arg(short = 'm')]
    message: Option<String>,
}

impl CommitTreeOptions {
    pub fn read(&self) -> Result<(&str, &str, &str), ArgsReadError> {
       // println!("In read");

        if let Some(tree_sha) = self.tree_sha.as_deref() {
           // println!("In read Some 1");
            if let Some(commit_sha) = self.commit_sha.as_deref() {
               // println!("In read Some 2");
                if let Some(message) = self.message.as_deref() {
                  //  println!("In read Some 3");
                    return Ok((tree_sha, commit_sha, message));
                }
                else {
                    return Err(ArgsReadError::CommitTreeCommandErrorArgThree);
            }
            }
            else {
            return Err(ArgsReadError::CommitTreeCommandErrorArgTwo);
        }
        }
        else {
        Err(ArgsReadError::CommitTreeCommandErrorArgOne)
    }
    }
}
//-----------------------------------------------------------------------------------------------
//-- Clone repo----------------------------------------------------------------------------------
#[derive(Args)]
pub struct CloneRepOptions {
    url: Option<String>,
    dir: Option<String>,
}

impl CloneRepOptions{
    pub fn read(&self) -> Result<(&str, &str), ArgsReadError> {
        if let Some(url) = self.url.as_deref() {
            if let Some(dir) = self.dir.as_deref() {
                return Ok((url,dir));
            }   
           return  Err(ArgsReadError::CloneRepCommandErrorArgTwo);
    }
    Err(ArgsReadError::CloneRepCommandErrorArgOne)
}
}


//-------------------------------------------------------------------------------------------
//-----------Error handling------------------------------------------------------------------
pub enum ArgsReadError {
    ReadBlobCommandError,
    CreateBlobCommandError,
    ReadTreeCommandError,
    CommitTreeCommandErrorArgOne,
    CommitTreeCommandErrorArgTwo,
    CommitTreeCommandErrorArgThree,
    CloneRepCommandErrorArgOne,
    CloneRepCommandErrorArgTwo,
}


impl Display for ArgsReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}
impl Debug for ArgsReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}
impl Error for ArgsReadError {}
impl ArgsReadError {
    fn message(&self) -> &str {
        match self {
            Self::ReadBlobCommandError => "ERROR: Read a blob object: blob sha not found",
            Self::CreateBlobCommandError => "ERROR: Create a blob objec: blob sha not found",
            Self::ReadTreeCommandError => "ERROR: Read a tree object: tree sha not found",
            Self::CommitTreeCommandErrorArgOne=> "ERROR: Commit a tree object: tree sha not found",
            Self::CommitTreeCommandErrorArgTwo=> "ERROR: Commit a tree object: commit sha not found",
            Self::CommitTreeCommandErrorArgThree=> "ERROR: Commit a tree object: message not found",
            Self::CloneRepCommandErrorArgOne => "ERROR: Clone a repository: url not found",
            Self::CloneRepCommandErrorArgTwo => "ERROR: Clone a repository: dir not found",
        }
    }
}
//---------------------------------------------------------------------------------------------------------
//---------------------------------------------------------------------------------------------------------
