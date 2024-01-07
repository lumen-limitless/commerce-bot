use teloxide::prelude::*;

use crate::schema::{AppDialogue, HandlerResult};

pub async fn cancel(bot: Bot, dialogue: AppDialogue, msg: Message) -> HandlerResult {
    tracing::info!("processing /cancel command in chat {}", msg.chat.id);

    bot.delete_message(msg.chat.id, msg.id).await?;

    bot.send_message(msg.chat.id, "Cancelled the dialogue.")
        .await?;

    dialogue.exit().await?;
    Ok(())
}
