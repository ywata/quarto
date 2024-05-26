#![cfg_attr(feature = "nightly", feature(iter_intersperse))]

use crate::quarto::BoardState;
use crate::quarto::{Piece, Quarto, QuartoError};
use sqlx::sqlite::SqliteQueryResult;

use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::convert::TryFrom;
use std::env;
use std::error::Error;

use log::{debug, error, info, warn};

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
        force: bool,
    },
    NewGame,
    Move {
        uuid: String,
        x: usize,
        y: usize,
        piece: String,
    },
    Quarto {
        uuid: String,
        x: usize,
        y: usize,
    },
}

async fn init_sqlite(db_url: &str) -> Result<SqliteQueryResult, SqlxError> {
    Sqlite::create_database(db_url).await?;

    let db: Pool<Sqlite> = SqlitePool::connect(db_url).await.unwrap();
    sqlx::query(
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
    .await
}

use sqlx::Error as SqlxError;

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
            info!("Insert record: {:?}", result);
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
            .fetch_one(db)
            .await
            .ok()?;
            if let (Some(bs), Some(np)) = (&result.board_state, &result.next_piece) {
                let np = Piece::try_from(np.to_string()).ok()?;
                let mut q = Quarto::try_from(bs).ok()?;
                if !q.pick_piece(&np) {
                    return None;
                }
                return Some(q);
            }
            None
        }
        #[cfg(feature = "init")]
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Cli::parse();
    let db_url = env::var("DATABASE_URL").expect("DATABASEURL should be set");
    info!("{:?}", &args);

    let result: Result<(), Box<dyn Error>> = match args.command {
        Command::Init { force } => {
            if !Sqlite::database_exists(&db_url).await.unwrap_or(false) || force {
                let result = init_sqlite(&db_url).await?;
            }
            Ok(())
        }
        Command::NewGame => {
            let db: Pool<Sqlite> = SqlitePool::connect(&db_url).await.unwrap();
            let uuid = Uuid::new_v4().to_string();
            let mut new_game = Quarto::new();
            // We are sure BSCF is valid Piece.
            let first_piece: Piece = Piece::try_from("BSCF".to_string()).unwrap();
            let _result = new_game.insert_new_game(&db, &uuid, &first_piece).await;
            println!("{}", uuid);
            Ok(())
        }
        Command::Move { uuid, x, y, piece } => {
            if !((0..4).contains(&x) && (0..4).contains(&y)) {
                error!("invalid coordinate: ({}, {})", &x, &y);
                return Err(QuartoError::OutOfRange)?;
            }
            if let None = piece.clone().into() {
                error!("invalid piece: {}", &piece);
                return Err(QuartoError::InvalidPieceError)?;
            }
            let db: Pool<Sqlite> = SqlitePool::connect(&db_url).await.unwrap();
            let np = Piece::try_from(piece.clone())?;
            if let Some(mut quarto) = Quarto::search_game_by_uuid(&db, &uuid).await {
                info!("{:?}", quarto);
                quarto.move_piece(x, y);
                quarto.pick_piece(&np);
                return Ok(());
            } else {
                error!("unknown uuid: {}", &uuid);
                return Err(QuartoError::AnyOther)?;
            }
        }
        Command::Quarto { .. } => Ok(()),
    };
    Ok(())
}
