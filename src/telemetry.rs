use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::MakeWriter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry,
};
/// Compose multiple layers into a `tracing`'s subscriber.
///
/// # Implementation Notes
///
/// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber<'a>(
    name: String,
    env_filter: String,
    _sink: impl MakeWriter<'a> + Send + Sync + 'static,
) -> impl Subscriber + Send + Sync {
    //Env filter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    // Formatting layer
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);
    // The `with` method is provided by `SubscriberExt` is an Extension trait for Subsciber exposed by `tracing_subscriber`
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all `log`'s events to our subsriber

    // `set_global_default` can be used by applications to specify what subscriber should be used to process spans.
    LogTracer::init().expect("Failed to set the logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
