use chrono::Local;
use env_logger::Builder;
use std::io::Write;
/// 初始化logger
/// 初始化日志系统
/// 1. 设置日志级别为Info
/// 2. 配置日志格式：时间戳 [日志级别] -> 日志内容
pub fn init_logger() {
    let mut builder = Builder::new();

    builder.filter(None, log::LevelFilter::Info);
    builder.format(|buf, record| {
        writeln!(
            buf,
            "{} [{}] -> {}",
            // Utc::now().with_timezone(&FixedOffset::east_opt(8 * 3600).expect("FixedOffset::east out of bounds")).format("%Y-%m-%d %H:%M:%S"),
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.args()
        )
    });
    builder.init();
}
