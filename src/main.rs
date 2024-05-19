#![cfg_attr(feature = "nightly", feature(iter_intersperse))]


use std::env;
use sqlx::{Pool, SqlitePool, Sqlite};
use sqlx::migrate::MigrateDatabase;


use crate::quarto::QuartoError;
use clap::{Parser, Subcommand};
use uuid::Uuid;
mod quarto;
mod cli;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    Init,
    NewGame,
}



async fn init_sqlite(db_url: &str) -> Result<(), QuartoError>{
    if Sqlite::database_exists(db_url).await.unwrap_or(false) {
        println!("duplicated: {}", db_url);
        return Err(QuartoError::FileExists);
    }
    if let Err(err) = Sqlite::create_database(db_url).await {
        println!("{:?}", err);
        return Err(QuartoError::FileExists);
    }

    let db: Pool<Sqlite> = SqlitePool::connect(db_url).await.unwrap();
    let result = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS game
        (
          id INTEGER PRIMARY KEY,
          uuid VARCHAR,
          assigned_1st BOOLEAN default false,
          assigned_2nd BOOLEAN default false,
          board_state VARCHAR
        );"#)
        .execute(&db).await.unwrap();
    println!("create table: {:?}", result);
    Ok(())
}


async fn insert_new_game(db: &Pool<Sqlite>, uuid: &String) -> (){
    #[cfg(not(feature = "init"))] {
        let result = sqlx::query!(
        r#"
        INSERT INTO game (uuid)
        VALUES (?1);
        "#, uuid
        ).execute(db).await.unwrap();
        print!("Insert record: {:?}", result);
    }

    ()
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASEURL should be set");
    println!("{:?}", &args);

    let result = match args.command {
        Command::Init => init_sqlite(&db_url).await,
        Command::NewGame => {
            let db: Pool<Sqlite> = SqlitePool::connect(&db_url).await.unwrap();
            let uuid = Uuid::new_v4().to_string();
            let result = insert_new_game(&db, &uuid).await;
            println!("{:?} {:?}", result, uuid);
            Ok(())
        }
    };

    Ok(())
}
