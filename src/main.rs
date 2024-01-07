mod commands;
mod schema;
mod utils;

use schema::{schema, Command, State};
use sqlx::SqlitePool;
use std::env;
use teloxide::utils::command::BotCommands;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting bot");

    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let bot = Bot::from_env();

    match bot.set_my_commands(Command::bot_commands()).await {
        Err(err) => tracing::error!("Failed to set commands: {}", err),
        Ok(_) => tracing::info!("Commands set successfully"),
    };

    let pool = SqlitePool::connect(&database_url).await?;

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new(), pool])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
