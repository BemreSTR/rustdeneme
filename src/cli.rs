use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "rustkv",
    version,
    about = "A tiny embedded key-value database written in Rust",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new database directory
    Init {
        /// Path to the database directory
        path: PathBuf,
    },
    /// Store a key-value pair
    Put {
        /// Path to the database directory
        path: PathBuf,
        /// Key to store
        key: String,
        /// Value to store
        value: String,
    },
    /// Retrieve the value for a key
    Get {
        /// Path to the database directory
        path: PathBuf,
        /// Key to look up
        key: String,
    },
    /// Delete a key from the database
    Delete {
        /// Path to the database directory
        path: PathBuf,
        /// Key to delete
        key: String,
    },
    /// List all stored keys and values
    List {
        /// Path to the database directory
        path: PathBuf,
    },
    /// Show database statistics
    Stats {
        /// Path to the database directory
        path: PathBuf,
    },
    /// Compact the log by rewriting only live records
    Compact {
        /// Path to the database directory
        path: PathBuf,
    },
}
