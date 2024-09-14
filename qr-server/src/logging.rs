use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

use crate::config::Config;

pub fn init_logger(config: &Config) {
    let Config { env, .. } = config;

    log_panics::init();

    let colors = ColoredLevelConfig::new()
        .debug(Color::White)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    let log_level = match env {
        crate::config::Env::Dev => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };

    let mut dispatch = fern::Dispatch::new().level(log_level);
    if log_level == LevelFilter::Debug {
        dispatch = dispatch
            .level_for("rocket", log::LevelFilter::Warn)
            // rocket_cors has a lot of unhelpful info messages that spam the log on every request
            // https://github.com/ActivityWatch/activitywatch/issues/975
            .level_for("rocket_cors", log::LevelFilter::Warn)
            .level_for("_", log::LevelFilter::Warn) // Rocket requests
            .level_for("launch_", log::LevelFilter::Warn); // Rocket config info
    }
    // Formatting
    dispatch
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}]: {}",
                chrono::Local::now().format("%y-%m-%d %H:%M:%S%.3f"),
                colors.color(record.level()),
                record.target(),
                message,
            ))
        })
        // stdout
        .chain(fern::Dispatch::new().chain(std::io::stdout()))
        // TODO: chain to log file
        .apply()
        .expect("Failed to init logger");
    log::debug!("Success to init logger: config={:?}", config);
}
