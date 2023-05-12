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
body = "001e# service=git-upload-pack\n0000015547b37f1a82bfe85f6d8df52b6258b75e4343b7fd HEAD\0multi_ack thin-pack side-band side-band-64k 
ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag multi_ack_detailed allow-tip-sha1-in-want allow-reachable-sha1-in-want
no-done symref=HEAD:refs/heads/master filter object-format=sha1 agent=git/github-aebc4fa63a74\n003f47b37f1a82bfe85f6d8df52b6258b75e4343b7fd refs/heads/master\n0000"
*/
/// git init
Init,
///Read a blob object
Cat_file(ReadBlobOptions), 

}

#[derive(Args)]
pub struct ReadBlobOptions {
    /// print
    #[arg(short = 'p')]
    pub print: String,
    /// hash
    #[arg(long)]
    pub hasn: Option<String>,
}