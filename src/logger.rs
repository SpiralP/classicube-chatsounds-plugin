use std::sync::Once;

use tracing::info;
use tracing_subscriber::filter::EnvFilter;

pub fn initialize(debug: bool, other_crates: bool) {
    static ONCE: Once = Once::new();
    ONCE.call_once(move || {
        let level = if debug { "debug" } else { "info" };
        let my_crate_name = env!("CARGO_PKG_NAME").replace('-', "_");

        let default_directive = if other_crates {
            level.parse().unwrap()
        } else {
            format!("{my_crate_name}={level}").parse().unwrap()
        };
        let filter = EnvFilter::builder()
            .with_default_directive(default_directive)
            .from_env_lossy();

        if let Err(e) = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_ansi(true)
            .without_time()
            .try_init()
        {
            eprintln!("failed to init tracing subscriber: {e}");
        }

        info!(
            "{} v{} init",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
    });
}
