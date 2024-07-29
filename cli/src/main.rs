use configs::app_config::{self, AppConfig};
use configs::cli_config::CliConfig;
use configs::types::{Storage, VectorDocument};
use rvector_core::check_working;
use storage::storage::StoreData;

fn main() {
    println!("Hello, world!");

    let cli_config = CliConfig::from_args();
    println!("{:?}", cli_config);

    let app_config = AppConfig::default();
    println!("{:?}", app_config);

    check_working();
}
