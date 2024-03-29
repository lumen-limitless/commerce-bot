use crate::{
    schema::{AppDialogue, HandlerResult},
    utils::assert_admin_id,
    State,
};
use format as f;
use sqlx::SqlitePool;
use teloxide::{prelude::*, types::ForceReply};

pub async fn add_product(bot: Bot, msg: Message, dialogue: AppDialogue) -> HandlerResult {
    tracing::info!("processing /add command in chat {}", msg.chat.id);

    bot.delete_message(msg.chat.id, msg.id).await?;

    let id = msg.from().unwrap().id.to_string().parse::<i64>()?;
    assert_admin_id(id)?;

    bot.send_message(msg.chat.id, "Please, send me the product name.")
        .reply_markup(ForceReply::default())
        .await?;

    dialogue.update(State::ReceiveProductName).await?;

    Ok(())
}

pub async fn receive_product_name(bot: Bot, dialogue: AppDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(product_name) => {
            dialogue
                .update(State::ReceiveProductDescription { product_name })
                .await?;

            bot.send_message(msg.chat.id, "Please, send me the product description.")
                .reply_markup(ForceReply::default())
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me the product name.")
                .reply_markup(ForceReply::default())
                .await?;
        }
    }

    Ok(())
}

pub async fn receive_product_description(
    bot: Bot,
    dialogue: AppDialogue,
    product_name: String,
    msg: Message,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(product_description) => {
            dialogue
                .update(State::ReceiveProductPrice {
                    product_name,
                    product_description,
                })
                .await?;

            bot.send_message(msg.chat.id, "Please, send me the product price in cents.")
                .reply_markup(ForceReply::default())
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me the product description.")
                .reply_markup(ForceReply::default())
                .await?;
        }
    }

    Ok(())
}

pub async fn receive_product_price(
    bot: Bot,
    dialogue: AppDialogue,
    (product_name, product_description): (String, String),
    msg: Message,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(product_price) => {
            let product_price = match product_price.parse::<i64>() {
                Ok(price) => price,
                Err(_) => {
                    bot.send_message(msg.chat.id, "Invalid price. Try again.")
                        .await?;
                    return Ok(());
                }
            };

            dialogue
                .update(State::ReceiveProductImage {
                    product_name,
                    product_description,
                    product_price,
                })
                .await?;

            bot.send_message(msg.chat.id, "Please, send me the product image.")
                .reply_markup(ForceReply::default())
                .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me the product price.")
                .reply_markup(ForceReply::default())
                .await?;
        }
    }

    Ok(())
}

pub async fn receive_product_image(
    bot: Bot,
    (product_name, product_description, product_price): (String, String, i64),
    msg: Message,
    dialogue: AppDialogue,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(product_image) => {
            sqlx::query!(
                "INSERT INTO products (name, description, price, image) VALUES (?, ?, ?, ?)",
                product_name,
                product_description,
                product_price,
                product_image
            )
            .execute(&pool)
            .await?;

            bot.send_message(
                msg.chat.id,
                f!("Product {product_name} added successfully."),
            )
            .await?;

            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me the product image.")
                .await?;
        }
    }

    Ok(())
}
