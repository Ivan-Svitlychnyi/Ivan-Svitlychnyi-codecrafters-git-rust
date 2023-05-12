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
Init,



}

