#![cfg_attr(feature = "nightly", feature(iter_intersperse))]

use crate::quarto::BoardState;
use crate::quarto::{Piece, Quarto, QuartoError};
use sqlx::sqlite::SqliteQueryResult;

use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::convert::TryFrom;
use std::env;
use std::error::Error;

use clap::{Parser, Subcommand};
use uuid::Uuid;
mod quarto;

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

async fn init_sqlite(db_url: &str) -> Result<SqliteQueryResult, sqlx::Error> {
    Sqlite::database_exists(db_url).await.unwrap_or(false);
    Sqlite::create_database(db_url).await?;

    let db: Pool<Sqlite> = SqlitePool::connect(db_url).await.unwrap();
    let result: Result<SqliteQueryResult, sqlx::Error> = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS game
        (
          id INTEGER PRIMARY KEY,
          uuid VARCHAR,
          assigned_1st BOOLEAN default false,
          assigned_2nd BOOLEAN default false,
          next_piece VARCHAR,
          board_state VARCHAR
        );"#,
    )
    .execute(&db)
    .await;
    result
}

use sqlx::Error as SqlxError;

impl From<SqlxError> for QuartoError {
    fn from(err: SqlxError) -> Self {
        match err {
            SqlxError::Database(_) => QuartoError::InvalidPieceError,
            SqlxError::Protocol(_) => QuartoError::InvalidPieceError,
            SqlxError::PoolTimedOut => QuartoError::InvalidPieceError,
            SqlxError::PoolClosed => QuartoError::InvalidPieceError,
            SqlxError::WorkerCrashed => QuartoError::InvalidPieceError,
            SqlxError::Migrate(_) => QuartoError::InvalidPieceError,
            SqlxError::Configuration(_) => QuartoError::InvalidPieceError,
            SqlxError::Tls(_) => QuartoError::InvalidPieceError,
            SqlxError::Io(_) => QuartoError::InvalidPieceError,
            SqlxError::Decode(_) => QuartoError::InvalidPieceError,
            //SqlxError::Encode(_) => QuartoError::InvalidPieceError,
            SqlxError::RowNotFound => QuartoError::InvalidPieceError,
            //SqlxError::ArgumentCount => QuartoError::InvalidPieceError,
            SqlxError::ColumnIndexOutOfBounds { .. } => QuartoError::InvalidPieceError,
            SqlxError::ColumnNotFound(_) => QuartoError::InvalidPieceError,
            //SqlxError::Message(_) => QuartoError::InvalidPieceError,
            //SqlxError::NotFound => QuartoError::InvalidPieceError,
            //SqlxError::__Nonexhaustive => QuartoError::InvalidPieceError,
            _ => QuartoError::InvalidPieceError,
        }
    }
}

impl Quarto {
    pub async fn insert_new_game(&mut self, db: &Pool<Sqlite>, uuid: &String, piece: &Piece) -> () {
        #[cfg(not(feature = "init"))]
        {
            if !self.pick_piece(piece) {
                return ();
            }
            let piece: String = Piece::from(self.next_piece.as_ref().unwrap().clone()).into();
            let board_state: String = (BoardState::from(self.board_state.clone())).into();
            let result = sqlx::query!(
                r#"
                INSERT INTO game (uuid, next_piece, board_state)
                VALUES (?1, ?2, ?3);
                "#,
                uuid,
                piece,
                board_state
            )
            //Quarto::format_board_state(self.board_state))
            .execute(db)
            .await
            .unwrap();
            print!("Insert record: {:?}", result);
        }

        ()
    }
}

#[tokio::main]
async fn main() -> Result<(), QuartoError> {
    let args = Cli::parse();
    let db_url = env::var("DATABASE_URL").expect("DATABASEURL should be set");
    println!("{:?}", &args);

    let result: Result<(), QuartoError> = match args.command {
        Command::Init => {
            let result = init_sqlite(&db_url).await;
            result.map(|_| ()).map_err(|_| QuartoError::FileExists)
        }
        Command::NewGame => {
            let db: Pool<Sqlite> = SqlitePool::connect(&db_url).await.unwrap();
            let uuid = Uuid::new_v4().to_string();
            let mut new_game = Quarto::new();
            // We are sure BSCF is valid Piece.
            let first_piece: Piece = Piece::try_from("BSCF".to_string()).unwrap();
            let result = new_game.insert_new_game(&db, &uuid, &first_piece).await;

            Ok(())
        }
    };
    Ok(())
}
