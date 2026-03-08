use anyhow::{Context, Result};
use clap::Args;
use ulid::Ulid;

use crate::{actions, display};

#[derive(Debug, Args, Clone)]
pub struct DeleteArgs {
    id: Ulid,
}

impl DeleteArgs {
    pub async fn exec(&self, base_url: &str, token: &str, json: bool) -> Result<()> {
        actions::delete_publication(base_url, token, &self.id)
            .await
            .context("delete publication")?;
        if json {
            println!("{{\"id\": \"{}\"}}", self.id);
        } else {
            display::print_delete_success(&self.id);
        }
        Ok(())
    }
}
