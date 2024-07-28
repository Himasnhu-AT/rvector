use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CliConfig {
    #[structopt(long, default_value = "./data")]
    pub storage_path: String,

    #[structopt(long, default_value = "8080")]
    pub api_port: u16,

    #[structopt(long, default_value = "info")]
    pub log_level: String,

    #[structopt(long, default_value = "./models/embedding_model.bin")]
    pub embedding_model: String,
}

impl CliConfig {
    pub fn from_args() -> Self {
        StructOpt::from_args() // This calls the `from_args` method from the `StructOpt` trait
    }
}
