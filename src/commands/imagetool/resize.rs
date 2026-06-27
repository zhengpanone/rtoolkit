use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
pub struct ResizeArgs {
    pub input: PathBuf,
    pub output: PathBuf,
    pub width: u32,
    pub height: u32,
}

#[derive(thiserror::Error, Debug)]
pub enum ResizeError {}

impl ResizeArgs {
    pub fn run(self) -> Result<(), ResizeError> {
        println!("{:#?}", self);
        Ok(())
    }
}
