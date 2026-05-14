use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Mutex;

pub struct Embedder {
    model: Mutex<TextEmbedding>,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        // Warn the user proactively before potentially starting a large download
        eprintln!("🧠 [gnx-rs] Initializing BGE-M3 Embedding Model...");
        eprintln!("   (If this is the first time, it will download ~1.2GB of model weights. Please wait...)");

        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGEM3)
                .with_show_download_progress(true),
        )?;

        Ok(Self {
            model: Mutex::new(model),
        })
    }

    pub fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut model = self.model.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        model.embed(texts, None).map_err(Into::into)
    }
}
