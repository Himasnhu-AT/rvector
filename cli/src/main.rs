use configs::app_config::{self, AppConfig};
use configs::cli_config::CliConfig;

use embeddings::embed;

fn main() {
    println!("Hello, world!");
    embed();

    let cli_config = CliConfig::from_args();
    println!("{:?}", cli_config);

    let app_config = AppConfig::default();
    println!("{:?}", app_config);
}
