use clap::Args;
use crate::engine::Engine;

#[derive(Args, Debug)]
pub struct ContextArgs {
    /// Name of the symbol to query
    #[arg(long)]
    pub name: String,
}

pub fn run(args: ContextArgs, _engine: &Engine) -> Result<(), String> {
    // Task 3 will implement JSON output parity.
    println!("Context routing wired for {}", args.name);
    Ok(())
}