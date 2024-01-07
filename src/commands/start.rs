use crate::schema::HandlerResult;
use sqlx::SqlitePool;
use teloxide::prelude::*;

pub async fn start(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    tracing::info!("processing /start command in chat {}", msg.chat.id);

    let from = match msg.from() {
        Some(from) => from,
        None => {
            bot.send_message(msg.chat.id, "Unable to get your information.")
                .await?;
            return Ok(());
        }
    };

    let id = from.id.to_string().parse::<i64>()?;

    sqlx::query!(
        "INSERT OR IGNORE INTO users (id, username, first_name, last_name) VALUES (?, ?, ?, ?)",
        id,
        from.username,
        from.first_name,
        from.last_name
    )
    .execute(&pool)
    .await?;

    sqlx::query!("INSERT OR IGNORE INTO carts (user_id) VALUES (?)", id)
        .execute(&pool)
        .await?;

    bot.send_message(msg.chat.id, "Welcome to the store!")
        .await?;

    Ok(())
}
