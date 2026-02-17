use configs::app_config::AppConfig;
use configs::cli_config::CliConfig;

use rvector_core::check_working;

fn main() {
    println!("Hello, world!");

    let cli_config = CliConfig::from_args();
    println!("{:?}", cli_config);

    let app_config = AppConfig::default();
    println!("{:?}", app_config);

    check_working();
}
