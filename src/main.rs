use anyhow::{anyhow, Result};
use clap::Parser;

use rustkv::cli::{Cli, Command};
use rustkv::storage::Database;

fn main() -> Result<()> {
    run(Cli::parse())
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Init { path } => {
            let (_, is_new) = Database::init(path)?;
            if is_new {
                println!("database initialized");
            } else {
                println!("database already exists");
            }
        }
        Command::Put { path, key, value } => {
            let mut database = Database::open_or_create(path)?;
            database.put(key, value)?;
            println!("ok");
        }
        Command::Get { path, key } => {
            if key.is_empty() {
                return Err(anyhow!("invalid input: key cannot be empty"));
            }
            let database = Database::open_or_create(path)?;
            let value = database
                .get(&key)
                .ok_or_else(|| anyhow!("key not found: {key}"))?;
            println!("{value}");
        }
        Command::Delete { path, key } => {
            let mut database = Database::open_or_create(path)?;
            let existed = database.delete(key)?;
            if existed {
                println!("deleted");
            } else {
                println!("key was already absent");
            }
        }
        Command::List { path } => {
            let database = Database::open_or_create(path)?;
            for (key, value) in database.list() {
                println!("{key}\t{value}");
            }
        }
        Command::Stats { path } => {
            let database = Database::open_or_create(path)?;
            let stats = database.stats()?;
            println!("{}", stats.summary());
        }
        Command::Compact { path } => {
            let mut database = Database::open_or_create(path)?;
            database.compact()?;
            println!("compacted");
        }
    }

    Ok(())
}
