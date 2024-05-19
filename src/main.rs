#![cfg_attr(feature = "nightly", feature(iter_intersperse))]

use sqlx::SqlitePool;
use crate::quarto::QuartoError;
use clap::{Parser, Subcommand};
use tokio::{main};
mod quarto;
mod cli;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    sqlite_url: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    Init,
    NewGame,
}


async fn init_sqlite(db_url: &str) -> Result<(), QuartoError>{
    use sqlx::{migrate::MigrateDatabase, Sqlite};

    if Sqlite::database_exists(db_url).await.unwrap_or(false) {
        return Err(QuartoError::FileExists);
    }
    if let Err(_) = Sqlite::create_database(db_url).await {
        return Err(QuartoError::FileExists);
    }

    let db = SqlitePool::connect(db_url).await.unwrap();
    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users
        (id INTEGER PRIMARY KEY NOT NULL,
        name VARCHAR(250) NOT NULL);"#)
        .execute(&db).await.unwrap();
    println!("Create user table result: {:?}", result);
    Ok(())
}



#[tokio::main]
async fn main() -> Result<(), QuartoError> {
    let args = Cli::parse();
    println!("{:?}", args);
    let result = match args.command {
        Command::Init => init_sqlite(&args.sqlite_url).await,
        _ => Ok(()),
    };

    Ok(())
}
