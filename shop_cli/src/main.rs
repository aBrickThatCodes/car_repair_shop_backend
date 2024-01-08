use anyhow::Result;
use shop_backend::ShopBackend;

#[async_std::main]
async fn main() -> Result<()> {
    let _ = ShopBackend::init().await;
    Ok(())
}
