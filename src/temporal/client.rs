pub struct TemporalClient;

impl TemporalClient {
    pub async fn connect(_address: &str) -> anyhow::Result<Self> {
        tracing::info!("Temporal integration placeholder — server at localhost:7233");
        Ok(TemporalClient)
    }
}
