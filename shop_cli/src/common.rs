use anyhow::Result;
use dialoguer::console::Term;

pub fn wait_for_continue(term: &Term) -> Result<()> {
    term.write_line("Press any key to continue")?;
    term.read_key()?;
    Ok(())
}
