use shop_backend::*;

use anyhow::Result;

#[async_std::main]
async fn main() -> Result<()> {
    let _ = ShopBackend::init().await?;
    Ok(())
}
