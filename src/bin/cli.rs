use clap::{Parser, Subcommand};
use darkdb::db::{Database, DbError};
// use serde_json::{Value, json};
use serde_json::Value;
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "CLI for DarkDB", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new collection
    Create { name: String },
    /// Insert a document
    Insert {
        collection: String,
        json: String,
        #[arg(short, long)]
        ttl: Option<i64>,
    },
    /// Find a document
    Find { collection: String, id: String },
    /// List all documents in a collection
    List { collection: String },
    /// Update a document
    Update {
        collection: String,
        id: String,
        json: String,
    },
    /// Delete a document
    Delete { collection: String, id: String },
    /// Drop a collection
    Drop { name: String },
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

fn main() -> Result<(), DbError> {
    init_logging();
    let cli = Cli::parse();
    let db = Database::new("data")?;
    // db.start_ttl_cleaner(10); // Clean every 60 seconds

    match cli.command {
        Commands::Create { name } => {
            db.collection(&name)?;
            println!("Created collection: {}", name);
            db.create_collection(&name)?;
            println!("Created collection: {}", name);
        }
        Commands::Insert {
            collection,
            json,
            ttl,
        } => {
            // let mut docs = self.documents.write().map_err(|_| DbError::LockPoisoned)?;
            let col = db.collection(&collection)?;
            println!("Parsing JSON: {}", &json);
            let value: Value = serde_json::from_str(&json)?;
            println!("Inserting...");
            let doc = col.insert(value, ttl)?;
            println!("Inserted document with ID: {}", doc.id);
            // println!("Parsed JSON: {}",);
        }
        Commands::Find { collection, id } => {
            let col = db.collection(&collection)?;
            if let Some(doc) = col.find(&id)? {
                println!("{}", serde_json::to_string_pretty(&doc.data)?);
            } else {
                println!("Document not found");
            }
        }
        Commands::List { collection } => {
            let col = db.collection(&collection)?;
            let docs = col.find_all()?;
            println!("{}", serde_json::to_string_pretty(&docs)?);
        }
        Commands::Update {
            collection,
            id,
            json,
        } => {
            let col = db.collection(&collection)?;
            let value: Value = serde_json::from_str(&json)?;
            let doc = col.update(&id, value)?;
            println!("Updated document with ID: {}", doc.id);
        }
        Commands::Delete { collection, id } => {
            let col = db.collection(&collection)?;
            col.delete(&id)?;
            println!("Deleted document with ID: {}", id);
        }
        Commands::Drop { name } => {
            db.drop_collection(&name)?;
            println!("Dropped collection: {}", name);
        }
    }

    Ok(())
}
