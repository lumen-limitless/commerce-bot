use teloxide::{prelude::*, utils::command::BotCommands};

use crate::schema::{Command, HandlerResult};

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    tracing::info!("processing /help command in chat {}", msg.chat.id);

    bot.delete_message(msg.chat.id, msg.id).await?;

    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}
