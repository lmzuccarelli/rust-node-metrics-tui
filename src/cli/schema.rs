// module schema
use clap::Parser;

/// cli struct
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Node Metrics Service", long_about = None)]
#[command(
    help_template = "{author-with-newline} {about-section}Version: {version} \n {usage-heading} {usage} \n {all-args} {tab}"
)]
pub struct Cli {
    /// config file to use
    #[arg(short, long, value_name = "config")]
    pub config: String,
}
