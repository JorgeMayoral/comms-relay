use anyhow::{Context, Result};
use clap::Args;

use crate::{actions, display};

#[derive(Debug, Args, Clone)]
pub struct ListArgs {
    /// Number of publications per page
    #[arg(long, default_value = "100")]
    per_page: i64,
    /// Number of the publications page to load
    #[arg(long, default_value = "1")]
    page: i64,
}

impl ListArgs {
    pub async fn exec(&self, base_url: &str, json: bool) -> Result<()> {
        let data = actions::get_all_publications(base_url, self.page, self.per_page)
            .await
            .context("list publications")?;
        if json {
            let out =
                serde_json::to_string_pretty(&data).context("serialize publications as JSON")?;
            println!("{out}");
        } else {
            display::print_publications(&data.publications());
            display::print_pagination_footer(data.page(), data.total_pages(), data.total_results());
        }
        Ok(())
    }
}
