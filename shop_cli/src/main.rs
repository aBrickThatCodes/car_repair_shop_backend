use shop_backend::*;

use anyhow::Result;

#[async_std::main]
async fn main() -> Result<()> {
    if let Err(e) = dotenvy::dotenv_override() {
        if !e.not_found() {
            bail!(e)
        }
    }
    Ok(())
}
