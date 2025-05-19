// src/bin/server.rs
use darkdb::{api, db::Database};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "darkdb-server", about = "DarkDB server")]
struct Opt {
    #[structopt(short, long, default_value = "data")]
    data_dir: PathBuf,

    #[structopt(short, long, default_value = "127.0.0.1")]
    host: String,

    #[structopt(short, long, default_value = "8080")]
    port: u16,

    #[structopt(long)]
    username: Option<String>,

    #[structopt(long)]
    password_hash: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let opt = Opt::from_args();

    // Initialize database
    let db = Database::load(&opt.data_dir)?;
    db.start_ttl_cleaner(60); // Clean every 60 seconds

    // Set up authentication
    let mut users = HashMap::new();
    if let (Some(username), Some(password_hash)) = (opt.username, opt.password_hash) {
        users.insert(username, password_hash);
    }

    // let auth_config = api::AuthConfig { users };
    // In your main.rs or server.rs:
    let auth_config = AuthConfig {
        users: {
            let mut map = HashMap::new();
            map.insert(
                "admin".to_string(),
                bcrypt::hash("your_password", 12).unwrap(),
            );
            map
        },
    };

    start_server(db, "0.0.0.0", 8080, auth_config).await?;

    // Start server
    // api::start_server(db, &opt.host, opt.port, auth_config).await?;

    Ok(())
}

// // src/bin/server.rs
// use darkdb::{api, db::Database};
// use std::{collections::HashMap, net::SocketAddr, path::PathBuf};
// use structopt::StructOpt;

// #[derive(Debug, StructOpt)]
// #[structopt(name = "darkdb-server", about = "DarkDB server")]
// struct Opt {
//     #[structopt(short, long, default_value = "data")]
//     data_dir: PathBuf,

//     #[structopt(short, long, default_value = "127.0.0.1")]
//     host: String,

//     #[structopt(short, long, default_value = "8080")]
//     port: u16,

//     #[structopt(long)]
//     username: Option<String>,

//     #[structopt(long)]
//     password_hash: Option<String>,
// }

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     tracing_subscriber::fmt()
//         .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//         .init();

//     let opt = Opt::from_args();

//     // Initialize database
//     let db = Database::load(&opt.data_dir)?;
//     db.start_ttl_cleaner(60); // Clean every 60 seconds

//     // Set up authentication
//     let mut users = HashMap::new();
//     if let (Some(username), Some(password_hash)) = (opt.username, opt.password_hash) {
//         users.insert(username, password_hash);
//     }

//     let auth_config = api::AuthConfig { users };

//     // Start server
//     api::start_server(db, &opt.host, opt.port, auth_config).await?;

//     Ok(())
// }
