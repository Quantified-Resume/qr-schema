use std::{
    fmt::{Display, Error},
    fs::File,
};

use clap::{arg, crate_version, Parser};
use homedir::my_home;
use serde::{Deserialize, Serialize};

use crate::err::none;

#[derive(Parser, Deserialize, Debug)]
#[clap(version = crate_version!(), author = "Sheep Zhang")]
#[serde(rename_all = "camelCase")]
pub struct Arg {
    /// The port to listen
    #[arg(short, long)]
    port: Option<u16>,
    /// The path of database
    #[arg(short, long)]
    db_path: Option<String>,
}

#[derive(Serialize)]
pub struct Config {
    pub port: u16,
    pub db_path: String,
}

impl Config {
    /// Parse configuration from all the inputs
    pub fn parse() -> Self {
        let mut config = Config::default();
        // [home_dir]/.qr/config.yaml
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
            Ok(json_str) => {
                write!(f, "{}", json_str)?;
                Ok(())
            }
            Err(_e) => Err(Error::default()),
        }
    }
}

impl Config {
    fn default() -> Self {
        // TODO: use App Data of different OS
        let db_path = String::from(".quantified_resume.db");
        Config {
            port: 12233,
            db_path,
        }
    }
}

fn parse_home_arg() -> Option<Arg> {
    let mut dir = match my_home() {
        Ok(opt) => match opt {
            Some(v) => v,
            None => return None,
        },
        Err(e) => return none(e, "Failed to get home dir path"),
    };
    dir.push(".qr");
    dir.push("config.json");
    if !dir.exists() {
        return None;
    }
    let file = match File::open(dir) {
        Ok(v) => v,
        Err(e) => return none(e, "Failed to open config file"),
    };

    match serde_json::from_reader::<File, Arg>(file) {
        Ok(v) => {
            println!("{:?}", v);
            Some(v)
        }
        Err(e) => none(e, "Failed to prase yaml config file"),
    }
}
/// Merge arg into config
fn merge(config: &mut Config, arg: Arg) {
    let Arg { port, db_path } = arg;
    if_present(port, |v| config.port = v);
    if_present(db_path, |v| config.db_path = v);
}

fn if_present<T, F>(opt: Option<T>, f: F)
where
    F: FnOnce(T) -> (),
{
    if opt.is_some() {
        f(opt.unwrap());
    }
}
