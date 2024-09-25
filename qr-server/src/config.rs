use crate::err::{none, or_none};
use clap::{arg, crate_version, Parser};
use homedir::my_home;
use qr_util::if_present;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Error},
    fs::File,
};

#[derive(Parser, Deserialize, Debug)]
#[clap(version = crate_version!(), author = "Sheep Zhang")]
#[serde(rename_all = "camelCase")]
pub struct Arg {
    /// Environment, optional: dev, stage, prod. Default is prod
    env: Option<String>,
    /// The port to listen
    #[arg(short, long)]
    port: Option<u16>,
    /// The path of database
    #[arg(short, long)]
    db_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Env {
    Dev,
    Stage,
    Prod,
}

impl Env {
    fn from(str_opt: Option<String>) -> Option<Self> {
        str_opt.and_then(|v| match v.to_lowercase().as_str() {
            "dev" | "development" => Some(Env::Dev),
            "stage" | "staging" | "test" | "testing" => Some(Env::Stage),
            "prod" | "production" => Some(Env::Prod),
            _ => None,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub env: Env,
    pub port: u16,
    pub db_path: String,
}

impl Config {
    /// Parse configuration from all the inputs
    pub fn parse() -> Self {
        let mut config = Config::default();
        let home_arg = parse_home_arg();
        if_present(home_arg, |a| merge(&mut config, a));
        // Cli Argument
        let arg = Arg::parse();
        merge(&mut config, arg);
        config
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(&self) {
            Ok(json_str) => write!(f, "{}", json_str),
            Err(_e) => Err(Error::default()),
        }
    }
}

impl Config {
    fn default() -> Self {
        // TODO: use App Data of different OS
        let db_path = String::from(".quantified_resume.db");
        Config {
            env: Env::Dev,
            port: 12233,
            db_path,
        }
    }
}

/// Parse config from [home_dir]/.qr/config.yaml
fn parse_home_arg() -> Option<Arg> {
    let mut dir = match my_home() {
        Ok(opt) => opt?,
        Err(e) => return none(e, "Failed to get home dir path"),
    };
    dir.push(".qr");
    dir.push("config.json");
    if !dir.exists() {
        return None;
    }
    let file = or_none(File::open(dir), "Failed to open config file")?;
    or_none(
        serde_json::from_reader::<File, Arg>(file),
        "Failed to prase yaml config file",
    )
}

/// Merge arg into config
fn merge(config: &mut Config, arg: Arg) {
    let Arg { port, db_path, env } = arg;
    if_present(port, |v| config.port = v);
    if_present(db_path, |v| config.db_path = v);
    if_present(Env::from(env), |v| config.env = v);
}
