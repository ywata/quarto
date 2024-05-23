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
    Init {
        #[arg(long)]
        force: Option<bool>,
    },
    NewGame,
    Move {
        uuid: String,
        x: usize,
        y: usize,
        piece: String,
    },
}

async fn init_sqlite(db_url: &str) -> Result<SqliteQueryResult, sqlx::Error> {
    Sqlite::create_database(db_url).await?;

    let db: Pool<Sqlite> = SqlitePool::connect(db_url).await.unwrap();
    let result: Result<SqliteQueryResult, sqlx::Error> = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS game
        (
              id INTEGER PRIMARY KEY,
              uuid VARCHAR,
              assigned_1st BOOLEAN NOT NULL default false,
              assigned_2nd BOOLEAN NOT NULL default false,
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
            SqlxError::Database(_) => QuartoError::AnyOther,
            SqlxError::Protocol(_) => QuartoError::AnyOther,
            SqlxError::PoolTimedOut => QuartoError::AnyOther,
            SqlxError::PoolClosed => QuartoError::AnyOther,
            SqlxError::WorkerCrashed => QuartoError::AnyOther,
            SqlxError::Migrate(_) => QuartoError::AnyOther,
            SqlxError::Configuration(_) => QuartoError::AnyOther,
            SqlxError::Tls(_) => QuartoError::AnyOther,
            SqlxError::Io(_) => QuartoError::AnyOther,
            SqlxError::Decode(_) => QuartoError::AnyOther,
            //SqlxError::Encode(_) => QuartoError::InvalidPieceError,
            SqlxError::RowNotFound => QuartoError::AnyOther,
            //SqlxError::ArgumentCount => QuartoError::InvalidPieceError,
            SqlxError::ColumnIndexOutOfBounds { .. } => QuartoError::AnyOther,
            SqlxError::ColumnNotFound(_) => QuartoError::AnyOther,
            //SqlxError::Message(_) => QuartoError::InvalidPieceError,
            //SqlxError::NotFound => QuartoError::InvalidPieceError,
            //SqlxError::__Nonexhaustive => QuartoError::InvalidPieceError,
            _ => QuartoError::AnyOther,
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
    async fn search_game_by_uuid(db: &Pool<Sqlite>, uuid: &str) -> Option<Quarto> {
        #[cfg(not(feature = "init"))]
        {
            let result = sqlx::query!(
                r#"
                 SELECT uuid, next_piece, board_state, assigned_1st, assigned_2nd
                 FROM game
                 WHERE uuid = ?1
                 "#,
                uuid
            )
            .fetch_all(db)
            .await;
            println!("{:?}", result);
            None
        }
        #[cfg(feature = "init")]
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), QuartoError> {
    let args = Cli::parse();
    let db_url = env::var("DATABASE_URL").expect("DATABASEURL should be set");
    println!("{:?}", &args);

    let result: Result<(), QuartoError> = match args.command {
        Command::Init { force } => {
            if Sqlite::database_exists(&db_url).await.unwrap_or(false) || force.unwrap_or(true) {
                let result = init_sqlite(&db_url).await;
                result.map(|_| ()).map_err(|_| QuartoError::FileExists);
                return Ok(()); // XXX
            } else {
                return Ok(());
            }
        }
        Command::NewGame => {
            let db: Pool<Sqlite> = SqlitePool::connect(&db_url).await.unwrap();
            let uuid = Uuid::new_v4().to_string();
            let mut new_game = Quarto::new();
            // We are sure BSCF is valid Piece.
            let first_piece: Piece = Piece::try_from("BSCF".to_string()).unwrap();
            let result = new_game.insert_new_game(&db, &uuid, &first_piece).await;
            println!("{}", uuid);
            Ok(())
        }
        Command::Move { uuid, x, y, piece } => {
            if !((0..4).contains(&x) && (0..4).contains(&y)) {
                return Err(QuartoError::AnyOther);
            }
            if let Some(piece_str) = piece.clone().into() {
                if let Ok(piece) = Piece::try_from(piece_str) {
                    // OK
                } else {
                    return Err(QuartoError::AnyOther);
                }
            }
            println!("{:?}", uuid);
            let db: Pool<Sqlite> = SqlitePool::connect(&db_url).await.unwrap();
            if let Some(quarto) = Quarto::search_game_by_uuid(&db, &uuid).await {
                println!("{:?}", quarto);
            } else {
            }
            Ok(())
        }
    };
    result
}
