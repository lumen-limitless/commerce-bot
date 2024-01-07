use format as f;
use sqlx::SqlitePool;
use teloxide::{
    dispatching::dialogue::GetChatId,
    prelude::*,
    types::{ForceReply, InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::{
    schema::{AppDialogue, HandlerResult},
    utils::format_price,
    State,
};

pub async fn view_cart(bot: Bot, msg: Message, pool: SqlitePool) -> HandlerResult {
    tracing::info!("processing /cart command in chat {}", msg.chat.id);

    bot.delete_message(msg.chat.id, msg.id).await?;

    let id = msg.from().unwrap().id.to_string().parse::<i64>()?;

    let cart = sqlx::query!("SELECT * FROM carts WHERE user_id = ?", id)
        .fetch_one(&pool)
        .await?;

    let cart_items = sqlx::query!("SELECT * FROM cart_items WHERE cart_id = ?", cart.id)
        .fetch_all(&pool)
        .await?;

    if cart_items.is_empty() {
        bot.send_message(msg.chat.id, "Your cart is empty.").await?;
        return Ok(());
    }

    let cart_products = sqlx::query!(
        "SELECT * FROM products WHERE id IN (SELECT product_id FROM cart_items WHERE cart_id = ?)",
        cart.id
    )
    .fetch_all(&pool)
    .await?;

    bot.send_message(
        msg.chat.id,
        f!(
            "Your cart({}):\n\n#ID - name - quantity - price\n\n--------------------------\n\n{}\n\n--------------------------\n\nTotal: {}",
            cart_items.len(),
            cart_products
                .iter()
                .map(|product| {
                    let cart_item = cart_items
                        .iter()
                        .find(|cart_item| cart_item.product_id == product.id.unwrap())
                        .unwrap();
                    f!(
                        "#{} - {} - x{} - {}",
                        cart_item.id,
                        product.name,
                        cart_item.quantity,
                        format_price(product.price * cart_item.quantity )
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
            format_price(cart_products.iter().fold(0, |acc, product| {
                let quantity = cart_items
                    .iter()
                    .find(|cart_item| cart_item.product_id == product.id.unwrap())
                    .unwrap()
                    .quantity;
                acc + product.price * quantity
            })),
        ),
    )
    .reply_markup(InlineKeyboardMarkup::new([
        vec![InlineKeyboardButton::callback("Place Order", "place_order")],
        vec![
            InlineKeyboardButton::callback("Remove Item", "remove_cart_item"),
            InlineKeyboardButton::callback("Edit Quantity", "edit_cart_item_quantity"),
        ],
    ]))
    .await?;

    Ok(())
}

pub async fn place_order_callback(bot: Bot, q: CallbackQuery, pool: SqlitePool) -> HandlerResult {
    let user_id = q.from.id.to_string().parse::<i64>()?;

    let cart_items = sqlx::query!(
        "SELECT * FROM cart_items WHERE cart_id = (SELECT id FROM carts WHERE user_id = ?)",
        user_id
    )
    .fetch_all(&pool)
    .await?;

    if cart_items.is_empty() {
        bot.send_message(q.chat_id().unwrap(), "Your cart is empty.")
            .await?;
        return Ok(());
    }

    let order = sqlx::query!(
        "INSERT INTO orders (user_id) VALUES (?) RETURNING *",
        user_id
    )
    .fetch_one(&pool)
    .await?;

    for cart_item in cart_items {
        sqlx::query!(
            "INSERT INTO order_items (order_id, product_id, quantity) VALUES (?, ?, ?)",
            order.id,
            cart_item.product_id,
            cart_item.quantity
        )
        .execute(&pool)
        .await?;

        sqlx::query!("DELETE FROM cart_items WHERE id = ?", cart_item.id)
            .execute(&pool)
            .await?;
    }

    bot.delete_message(q.chat_id().unwrap(), q.clone().message.unwrap().id)
        .await?;

    bot.send_message(
        q.chat_id().unwrap(),
        "Order placed successfully. use /orders to view your orders.",
    )
    .await?;

    Ok(())
}

pub async fn remove_cart_item_callback(
    bot: Bot,
    q: CallbackQuery,
    dialogue: AppDialogue,
) -> HandlerResult {
    dialogue.update(State::ReceiveRemoveCartItemId).await?;

    bot.send_message(
        q.chat_id().unwrap(),
        "Please, send me the cart item ID (#).",
    )
    .reply_markup(ForceReply::default())
    .await?;
    Ok(())
}

pub async fn receive_remove_cart_item_id(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
    dialogue: AppDialogue,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(cart_item_id) => {
            tracing::info!("cart_item_id: {}", cart_item_id);

            let cart_item_id = match cart_item_id.parse::<i64>() {
                Ok(cart_item_id) => cart_item_id,
                Err(_) => {
                    bot.send_message(msg.chat.id, "Invalid cart item id.")
                        .await?;
                    return Ok(());
                }
            };

            match sqlx::query!("DELETE FROM cart_items WHERE id = ?", cart_item_id)
                .execute(&pool)
                .await
            {
                Err(_) => {
                    bot.send_message(msg.chat.id, "Invalid cart item id.")
                        .await?;
                    return Ok(());
                }
                Ok(_) => {
                    bot.send_message(msg.chat.id, "Cart item removed successfully.")
                        .await?;

                    dialogue.exit().await?;
                }
            };
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me the cart item id.")
                .await?;
        }
    }
    Ok(())
}

pub async fn edit_cart_item_quantity_callback(
    bot: Bot,
    q: CallbackQuery,
    dialogue: AppDialogue,
) -> HandlerResult {
    dialogue
        .update(State::ReceiveEditCartItemQuantityId)
        .await?;

    bot.send_message(
        q.chat_id().unwrap(),
        "Please, send me the cart item ID (#).",
    )
    .reply_markup(ForceReply::default())
    .await?;
    Ok(())
}

pub async fn receive_edit_cart_item_quantity_id(
    bot: Bot,
    msg: Message,
    dialogue: AppDialogue,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(cart_item_id) => {
            tracing::info!("cart_item_id: {}", cart_item_id);

            let cart_item_id = match cart_item_id.parse::<i64>() {
                Ok(cart_item_id) => cart_item_id,
                Err(_) => {
                    bot.send_message(msg.chat.id, "Invalid cart item id.")
                        .await?;
                    return Ok(());
                }
            };

            dialogue
                .update(State::ReceiveEditCartItemQuantityAmount { cart_item_id })
                .await?;

            bot.send_message(
                msg.chat.id,
                format!(
                    "Please, send me the new quantity for cart item #{}.",
                    cart_item_id
                ),
            )
            .reply_markup(ForceReply::default())
            .await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me the cart item id.")
                .await?;
        }
    }
    Ok(())
}

pub async fn receive_edit_cart_item_quantity_amount(
    bot: Bot,
    msg: Message,
    cart_item_id: i64,
    dialogue: AppDialogue,
    pool: SqlitePool,
) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(quantity) => {
            tracing::info!("quantity: {}", quantity);

            let quantity = match quantity.parse::<i64>() {
                Ok(quantity) => quantity,
                Err(_) => {
                    bot.send_message(msg.chat.id, "Invalid quantity.").await?;
                    return Ok(());
                }
            };

            match sqlx::query!(
                "UPDATE cart_items SET quantity = ? WHERE id = ?",
                quantity,
                cart_item_id
            )
            .execute(&pool)
            .await
            {
                Err(_) => {
                    bot.send_message(msg.chat.id, "Invalid cart item id.")
                        .await?;
                    return Ok(());
                }
                Ok(_) => {
                    bot.send_message(msg.chat.id, "Cart item updated successfully.")
                        .await?;

                    dialogue.exit().await?;
                }
            };
        }
        None => {
            bot.send_message(msg.chat.id, "Please, send me the quantity.")
                .await?;
        }
    }
    Ok(())
}
