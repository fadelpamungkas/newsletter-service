use tracing_subscriber::{fmt::MakeWriter, prelude::__tracing_subscriber_SubscriberExt};

pub fn init_subscriber<T: tracing::Subscriber + Send + Sync>(subscriber: T) {
    // Redirect all log's events to our subscriber
    tracing_log::LogTracer::init().expect("Failed to set logger");
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl tracing::Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(env_filter));
    let formatting_layer = tracing_bunyan_formatter::BunyanFormattingLayer::new(name, sink);

    tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(tracing_bunyan_formatter::JsonStorageLayer)
        .with(formatting_layer)
}
