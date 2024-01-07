use crate::schema::{AppDialogue, HandlerResult};
use crate::{utils::assert_admin_id, State};
use sqlx::SqlitePool;
use teloxide::prelude::*;
use teloxide::types::ForceReply;

pub async fn remove_product(bot: Bot, msg: Message, dialogue: AppDialogue) -> HandlerResult {
    tracing::info!("processing /remove command in chat {}", msg.chat.id);

    bot.delete_message(msg.chat.id, msg.id).await?;

    let id = msg.from().unwrap().id.to_string().parse::<i64>()?;
    assert_admin_id(id)?;

    bot.send_message(msg.chat.id, "Please, send me the product id.")
        .reply_markup(ForceReply::default())
        .await?;

    dialogue.update(State::ReceiveProductId).await?;

    Ok(())
}

pub async fn receive_product_id(
    bot: Bot,
    dialogue: AppDialogue,
    msg: Message,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(product_id) => {
            let product_id = match product_id.parse::<i64>() {
                Ok(product_id) => product_id,
                Err(_) => {
                    bot.send_message(msg.chat.id, "Invalid product id.").await?;
                    return Ok(());
                }
            };

            if let Err(err) = sqlx::query!("DELETE FROM products WHERE id = ?", product_id)
                .execute(&pool)
                .await
            {
                tracing::error!("Error: {}", err);
                bot.send_message(msg.chat.id, "Invalid product id.").await?;
                return Ok(());
            };

            bot.send_message(msg.chat.id, "Product removed successfully.")
                .await?;

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me the product id.")
                .await?;
        }
    }

    Ok(())
}
