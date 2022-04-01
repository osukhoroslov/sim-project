use atty::Stream;
use colored::{Color, ColoredString, Colorize};

pub fn get_colored(s: &str, color: Color) -> ColoredString {
    if atty::is(Stream::Stderr) {
        s.color(color)
    } else {
        s.normal()
    }
}

#[macro_export]
macro_rules! log_info {
    ($ctx:expr, $msg:expr) => (
        log::info!(
            target: $ctx.name(),
            "[{:.3} {}  {}] {}",
            $ctx.time(), $crate::log::get_colored("INFO", $crate::colored::Color::Green), $ctx.name(), $msg
        )
    );
    ($ctx:expr, $format:expr, $($arg:tt)+) => (
        log::info!(
            target: $ctx.name(),
            concat!("[{:.3} {}  {}] ", $format),
            $ctx.time(), $crate::log::get_colored("INFO", $crate::colored::Color::Green), $ctx.name(), $($arg)+
        )
    );
}

#[macro_export]
macro_rules! log_debug {
    ($ctx:expr, $msg:expr) => (
        log::debug!(
            target: $ctx.name(),
            "[{:.3} {} {}] {}",
            $ctx.time(), $crate::log::get_colored("DEBUG", $crate::colored::Color::Blue), $ctx.name(), $msg
        )
    );
    ($ctx:expr, $format:expr, $($arg:tt)+) => (
        log::debug!(
            target: $ctx.name(),
            concat!("[{:.3} {} {}] ", $format),
            $ctx.time(), $crate::log::get_colored("DEBUG", $crate::colored::Color::Blue), $ctx.name(), $($arg)+
        )
    );
}

#[macro_export]
macro_rules! log_trace {
    ($ctx:expr, $msg:expr) => (
        log::trace!(
            target: $ctx.name(),
            "[{:.3} {} {}] {}",
            $ctx.time(), $crate::log::get_colored("TRACE", $crate::colored::Color::Cyan), $ctx.name(), $msg
        )
    );
    ($ctx:expr, $format:expr, $($arg:tt)+) => (
        log::trace!(
            target: $ctx.name(),
            concat!("[{:.3} {} {}] ", $format),
            $ctx.time(), $crate::log::get_colored("TRACE", $crate::colored::Color::Cyan), $ctx.name(), $($arg)+
        )
    );
}

#[macro_export]
macro_rules! log_error {
    ($ctx:expr, $msg:expr) => (
        log::error!(
            target: $ctx.name(),
            "[{:.3} {} {}] {}",
            $ctx.time(), $crate::log::get_colored("ERROR", $crate::colored::Color::Red), $ctx.name(), $msg
        )
    );
    ($ctx:expr, $format:expr, $($arg:tt)+) => (
        log::error!(
            target: $ctx.name(),
            concat!("[{:.3} {} {}] ", $format),
            $ctx.time(), $crate::log::get_colored("ERROR", $crate::colored::Color::Red), $ctx.name(), $($arg)+
        )
    );
}

#[macro_export]
macro_rules! log_warn {
    ($ctx:expr, $msg:expr) => (
        log::warn!(
            target: $ctx.name(),
            "[{:.3} {}  {}] {}",
            $ctx.time(), $crate::log::get_colored("WARN", $crate::colored::Color::Yellow), $ctx.name(), $msg
        )
    );
    ($ctx:expr, $format:expr, $($arg:tt)+) => (
        log::warn!(
            target: $ctx.name(),
            concat!("[{:.3} {}  {}] ", $format),
            $ctx.time(), $crate::log::get_colored("WARN", $crate::colored::Color::Yellow), $ctx.name(), $($arg)+
        )
    );
}
