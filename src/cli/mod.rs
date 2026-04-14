mod args;
mod commands;

pub use args::Args;

use crate::config::AppConfig;

pub async fn run(args: Args, config: AppConfig) -> anyhow::Result<()> {
    commands::dispatch(args, config).await
}

impl Args {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
}
