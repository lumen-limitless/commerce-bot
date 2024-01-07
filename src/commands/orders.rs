use crate::schema::HandlerResult;
use format as f;
use sqlx::SqlitePool;
use teloxide::prelude::*;

pub async fn view_orders(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    tracing::info!("processing /orders command in chat {}", msg.chat.id);

    bot.delete_message(msg.chat.id, msg.id).await?;

    let user_id = match msg.from() {
        Some(user) => user.id.to_string().parse::<i64>()?,
        None => {
            bot.send_message(msg.chat.id, "Failed to get user ID.")
                .await?;
            return Ok(());
        }
    };

    let orders = sqlx::query!(
        "SELECT * FROM orders WHERE user_id = ? ORDER BY id DESC",
        user_id
    )
    .fetch_all(&pool)
    .await?;

    if orders.is_empty() {
        bot.send_message(msg.chat.id, "You have no orders.").await?;
        return Ok(());
    }

    let orders = orders
        .iter()
        .map(|order| format!("#{}", order.id,))
        .collect::<Vec<_>>()
        .join("\n\n");

    bot.send_message(msg.chat.id, f!("Your orders:\n\n{}", orders))
        .await?;

    Ok(())
}
