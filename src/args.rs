use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    /// Config file for the spa server
    #[arg(short, long, value_name = "FILE")]
    pub config: PathBuf,

    /// Log Requests
    #[arg(long, default_value_t = false)]
    pub log_requests: bool,

    /// Log Responses
    #[arg(long, default_value_t = false)]
    pub log_responses: bool,
}

impl Args {
    pub fn new() -> Self {
        Self::parse()
    }
}
