#[cfg(feature = "fluvio-imports")]
pub mod fluvio {
    pub use fluvio::{FluvioError, PartitionConsumer, RecordKey, TopicProducer};
}

pub mod opt;

pub fn git_hash_version() -> &'static str {
    env!("GIT_HASH")
}
