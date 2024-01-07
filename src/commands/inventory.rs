use crate::schema::HandlerResult;
use crate::utils::format_price;
use format as f;
use itertools::Itertools;
use sqlx::SqlitePool;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

pub async fn inventory(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    tracing::info!("processing /inventory command in chat {}", msg.chat.id);

    bot.delete_message(msg.chat.id, msg.id).await?;

    let products = sqlx::query!("SELECT * FROM products")
        .fetch_all(&pool)
        .await?;

    if products.is_empty() {
        bot.send_message(msg.chat.id, "The store is empty.").await?;
        return Ok(());
    }

    let products = products.into_iter().map(|product| {
        InlineKeyboardButton::callback(product.name, f!("view_product {}", product.id))
    });

    let products = products
        .chunks(2)
        .into_iter()
        .map(|chunk| chunk.collect::<Vec<_>>())
        .collect::<Vec<_>>();

    bot.send_message(msg.chat.id, "Select a product to view more information:")
        .reply_markup(InlineKeyboardMarkup::new(products))
        .await?;

    Ok(())
}

pub async fn view_product_callback(
    bot: Bot,
    q: CallbackQuery,
    pool: SqlitePool,
    product_id: i64,
) -> HandlerResult {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => {
            bot.answer_callback_query(q.id)
                .text("Failed to get chat ID.")
                .await?;

            return Ok(());
        }
    };

    let product = sqlx::query!("SELECT * FROM products WHERE id = ?", product_id)
        .fetch_one(&pool)
        .await?;

    let (name, description, price, image) = (
        product.name,
        product.description,
        product.price,
        product.image,
    );

    bot.send_message(
            chat_id,
            f!("Name: {name}\n\nID: {product_id}\n\nDescription: {description}\n\nPrice: {}\n\nImage: {image}", format_price(price)),
        )
        .reply_markup(InlineKeyboardMarkup::new([
            vec![InlineKeyboardButton::callback(
                "Add to cart",
                f!("add_to_cart {product_id}"),
            ),
            InlineKeyboardButton::callback(
                "Back",
                "back",
            )
         ],
        ]))
        .await?;

    bot.answer_callback_query(q.id).await?;

    Ok(())
}

pub async fn add_to_cart_callback(
    bot: Bot,
    q: CallbackQuery,
    product_id: i64,
    pool: SqlitePool,
) -> HandlerResult {
    let user_id = q.from.id.to_string().parse::<i64>()?;

    let cart = sqlx::query!("SELECT * FROM carts WHERE user_id = ?", user_id)
        .fetch_one(&pool)
        .await?;

    let cart_item = sqlx::query!(
        "SELECT * FROM cart_items WHERE cart_id = ? AND product_id = ?",
        cart.id,
        product_id
    );

    match cart_item.fetch_optional(&pool).await {
        Ok(cart_item) => match cart_item {
            Some(cart_item) => {
                sqlx::query!(
                    "UPDATE cart_items SET quantity = quantity + 1 WHERE id = ?",
                    cart_item.id
                )
                .execute(&pool)
                .await?;

                bot.answer_callback_query(q.id)
                    .text("Added another to your cart.")
                    .await?;
            }
            None => {
                sqlx::query!(
                    "INSERT INTO cart_items (cart_id, product_id, quantity) VALUES (?, ?, ?)",
                    cart.id,
                    product_id,
                    1
                )
                .execute(&pool)
                .await?;

                bot.answer_callback_query(q.id)
                    .text("Product added to cart.")
                    .await?;
            }
        },
        Err(_) => {
            bot.answer_callback_query(q.id)
                .text("Failed to add product to cart.")
                .await?;
        }
    }

    Ok(())
}
