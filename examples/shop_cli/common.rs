use anyhow::Result;
use dialoguer::{console::Term, Input};

pub fn wait_for_continue(term: &Term) -> Result<()> {
    term.write_line("Press any key to continue")?;
    term.read_key()?;
    Ok(())
}

pub fn input(term: &Term, prompt: &str) -> Result<String> {
    Ok(Input::new().with_prompt(prompt).interact_text_on(term)?)
}

pub fn format_err(e: &dyn std::error::Error) -> String {
    format!("{e}")
}
