use shop_backend::ShopBackend;

use anyhow::Result;
use async_std::task::block_on;

fn main() -> Result<()> {
    let _ = block_on(ShopBackend::init());
    Ok(())
}
