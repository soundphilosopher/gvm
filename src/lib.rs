pub mod cli;
pub mod config;
pub mod utils;

pub type Res<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("[{}] {}", "o".yellow().bold(), std::format_args!($($arg)*));
  })
}

#[macro_export]
macro_rules! success {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("\t[{}] {}", "✓".green().bold(), std::format_args!($($arg)*));
  })
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => ({
    use colored::Colorize;
    println!("\t[{}] {}", "!".red().bold(), std::format_args!($($arg)*));
    std::process::exit(1);
  })
}
