use std::path::Path;

use serde::{Deserialize, Serialize};

pub fn format_price(price: i64) -> String {
    format!("${:.2}", price / 100)
}

pub fn assert_admin_id(id: i64) -> eyre::Result<()> {
    let admin_id = std::env::var("ADMIN_ID")?.parse::<i64>()?;
    if id != admin_id {
        Err(eyre::eyre!("Only admin can perform this action"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    open: String,
    close: String,
    payment_methods: Vec<String>,
    fulfillment_methods: Vec<String>,
}

pub fn parse_config() -> eyre::Result<Config> {
    let config = confy::load_path(Path::new("Config.toml"))?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_admin_id() {
        dotenvy::dotenv().ok();
        let admin_id = std::env::var("ADMIN_ID").unwrap().parse::<i64>().unwrap();

        assert!(assert_admin_id(admin_id).is_ok());

        assert!(assert_admin_id(admin_id + 1).is_err());
    }
}
