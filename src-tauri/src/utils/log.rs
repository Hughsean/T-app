use super::config::Config;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger() {
    let log_file: std::fs::File = Config::get_instance().logger.clone().into();
    let level: tracing::level_filters::LevelFilter = Config::get_instance().logger.clone().into();

    // 配置输出到文件的 fmt 层
    let file_fmt = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::new(
            time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        ))
        .with_ansi(false)
        .with_writer(std::sync::Mutex::new(log_file))
        .with_thread_names(true);

    // 配置输出到控制台的 fmt 层
    let console_fmt = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::LocalTime::new(
            time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
        ))
        .with_ansi(true);

    tracing_subscriber::registry()
        .with(level)
        .with(file_fmt)
        .with(console_fmt)
        .init();
    info!(">>>>>日志初始化完成<<<<<");

    info!("工作目录: {}", std::env::current_dir().unwrap().display());
}
