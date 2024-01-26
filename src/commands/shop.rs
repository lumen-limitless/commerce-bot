use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, WebAppInfo},
};
use url::Url;

use crate::schema::{AppDialogue, HandlerResult};

pub async fn shop(bot: Bot, dialogue: AppDialogue, msg: Message) -> HandlerResult {
    tracing::info!("processing /cancel command in chat {}", msg.chat.id);

    bot.delete_message(msg.chat.id, msg.id).await?;

    bot.send_message(msg.chat.id, "Cancelled the dialogue.")
        .await?;

    bot.send_message(msg.chat.id, "Visit the store here:")
        .reply_markup(InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::web_app(
                "Visit Store",
                WebAppInfo {
                    url: Url::parse("https://xenon-lumenlimitless.vercel.app/").unwrap(),
                },
            ),
        ]]))
        .await?;

    dialogue.exit().await?;
    Ok(())
}
