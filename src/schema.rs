use sqlx::SqlitePool;
use teloxide::{
    dispatching::{
        dialogue::{self, GetChatId, InMemStorage},
        UpdateHandler,
    },
    prelude::*,
    utils::command::BotCommands,
};

use crate::commands::{
    add::{
        add_product, receive_product_description, receive_product_image, receive_product_name,
        receive_product_price,
    },
    cancel::cancel,
    cart::{
        edit_cart_item_quantity_callback, place_order_callback,
        receive_edit_cart_item_quantity_amount, receive_edit_cart_item_quantity_id,
        receive_remove_cart_item_id, remove_cart_item_callback, view_cart,
    },
    help::help,
    inventory::{add_to_cart_callback, inventory, view_product_callback},
    orders::view_orders,
    remove::{receive_product_id, remove_product},
    shop::shop,
    start::start,
};

pub type AppDialogue = Dialogue<State, InMemStorage<State>>;

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,

    // Add product
    ReceiveProductName,
    ReceiveProductDescription {
        product_name: String,
    },
    ReceiveProductPrice {
        product_name: String,
        product_description: String,
    },
    ReceiveProductImage {
        product_name: String,
        product_description: String,
        product_price: i64,
    },

    // Remove product
    ReceiveProductId,

    // Cart
    ReceiveRemoveCartItemId,
    ReceiveEditCartItemQuantityId,
    ReceiveEditCartItemQuantityAmount {
        cart_item_id: i64,
    },
}

/// These commands are supported:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Show help text.")]
    Help,

    #[command(description = "Create a new account in the store.")]
    Start,

    #[command(description = "Cancel the current dialogue.")]
    Cancel,

    #[command(description = "View the store's inventory.")]
    Inventory,

    #[command(description = "Add a new product.")]
    Add,

    #[command(description = "Remove a product.")]
    Remove,

    #[command(description = "View your cart.")]
    Cart,

    #[command(description = "View your orders.")]
    Orders,

    #[command(description = "View the shop web app.")]
    Shop,
}

pub fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Help].endpoint(help))
                .branch(case![Command::Start].endpoint(start)),
        )
        .branch(case![Command::Cancel].endpoint(cancel))
        .branch(case![Command::Inventory].endpoint(inventory))
        .branch(case![Command::Add].endpoint(add_product))
        .branch(case![Command::Remove].endpoint(remove_product))
        .branch(case!(Command::Cart).endpoint(view_cart))
        .branch(case!(Command::Orders).endpoint(view_orders))
        .branch(case!(Command::Shop).endpoint(shop));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::ReceiveProductId].endpoint(receive_product_id))
        .branch(case![State::ReceiveProductName].endpoint(receive_product_name))
        .branch(
            case![State::ReceiveProductDescription { product_name }]
                .endpoint(receive_product_description),
        )
        .branch(
            case![State::ReceiveProductPrice {
                product_name,
                product_description
            }]
            .endpoint(receive_product_price),
        )
        .branch(
            case![State::ReceiveProductImage {
                product_name,
                product_description,
                product_price
            }]
            .endpoint(receive_product_image),
        )
        .branch(case![State::ReceiveRemoveCartItemId].endpoint(receive_remove_cart_item_id))
        .branch(
            case![State::ReceiveEditCartItemQuantityId]
                .endpoint(receive_edit_cart_item_quantity_id),
        )
        .branch(
            case![State::ReceiveEditCartItemQuantityAmount { cart_item_id }]
                .endpoint(receive_edit_cart_item_quantity_amount),
        )
        .branch(dptree::endpoint(invalid_state));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(Update::filter_callback_query().endpoint(callback_query_handler))
}

async fn callback_query_handler(
    bot: Bot,
    dialogue: AppDialogue,
    q: CallbackQuery,
    pool: SqlitePool,
) -> HandlerResult {
    tracing::debug!("Callback query: {:#?}", q);
    if let Some(data) = &q.data {
        tracing::info!("Handling callback query data: {}", data);

        match data.split_whitespace().collect::<Vec<&str>>().as_slice() {
            ["view_product", product_id] => {
                view_product_callback(bot, q.clone(), pool, product_id.parse::<i64>()?).await
            }

            ["add_to_cart", product_id] => {
                add_to_cart_callback(bot, q.clone(), product_id.parse::<i64>()?, pool).await
            }

            ["remove_cart_item"] => remove_cart_item_callback(bot, q.clone(), dialogue).await,

            ["edit_cart_item_quantity"] => {
                edit_cart_item_quantity_callback(bot, q.clone(), dialogue).await
            }

            ["place_order"] => place_order_callback(bot, q.clone(), pool).await,

            ["back"] => back_callback(bot, q.clone()).await,

            _ => Ok(()),
        }
    } else {
        tracing::warn!("Callback query data is empty");
        Ok(())
    }
}

async fn invalid_state(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Unable to handle the message. Type /help to see the usage.",
    )
    .await?;
    Ok(())
}

async fn back_callback(bot: Bot, q: CallbackQuery) -> HandlerResult {
    let chat_id = match q.chat_id() {
        Some(chat_id) => chat_id,
        None => return Ok(()),
    };

    let message_id = match q.message {
        Some(message) => message.id,
        None => return Ok(()),
    };

    bot.delete_message(chat_id, message_id).await?;

    bot.answer_callback_query(q.id).await?;

    Ok(())
}
