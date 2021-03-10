#[macro_export]
macro_rules! unwrap_or_return {
    ($x:expr, $y:expr) => {
        match $x {
            Some(x) => x,
            None => {
                return $y;
            }
        }
    };
    ($x:expr) => {
        unwrap_or_return!($x, ())
    };
}

#[macro_export]
macro_rules! error {
    ($($x:expr),*) => {
        log::error!("{} {}", LOGGING_MODULE, format!($($x),*));
    }
}

#[macro_export]
macro_rules! debug {
    ($($x:expr),*) => {
        log::debug!("{} {}", LOGGING_MODULE, format!($($x),*));
    }
}

#[macro_export]
macro_rules! info {
    ($($x:expr),*) => {
        log::info!("{} {}", LOGGING_MODULE, format!($($x),*));
    }
}

#[macro_export]
macro_rules! warn {
    ($($x:expr),*) => {
        log::warn!("{} {}", LOGGING_MODULE, format!($($x),*));
    }
}
