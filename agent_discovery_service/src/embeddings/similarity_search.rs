use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};


// Simulate vector embeddings as a simple Vec<f32>
pub type Embedding = Vec<f32>;

// Represents a registered agent or tool with its metadata and embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchableAgent {
    pub id: String,
    pub name: String,
    pub description: String,
    // This is the vector representation of the description
    pub embedding: Embedding,
}

// In-memory vector database for demonstration purposes.
// In a real implementation, this would be a client to a real vector DB.
#[derive(Debug, Clone, Default)]
pub struct VectorDB {
    agents: Vec<SearchableAgent>,
}

impl VectorDB {
    // A simple cosine similarity function
    fn cosine_similarity(a: &Embedding, b: &Embedding) -> f32 {
        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
        dot_product / (norm_a * norm_b)
    }

    // Add an agent to our in-memory DB. In a real scenario, this would be an "upsert" call.
    pub fn add_agent(&mut self, agent: SearchableAgent) {
        self.agents.push(agent);
    }

    // The core search function
    pub fn find_similar(&self, query_embedding: &Embedding, top_k: usize) -> Vec<&SearchableAgent> {
        let mut scored_agents: Vec<_> = self.agents.iter().map(|agent| {
            let similarity = Self::cosine_similarity(&agent.embedding, query_embedding);
            (similarity, agent)
        }).collect();

        // Sort by similarity score in descending order
        scored_agents.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Return the top_k results
        scored_agents.into_iter().take(top_k).map(|(_, agent)| agent).collect()
    }
}

// The main state for our discovery service
#[derive(Debug, Clone, Default)]
pub struct DiscoveryServiceState {
    pub vector_db: Arc<Mutex<VectorDB>>,
}

// Placeholder for a function that generates embeddings.
// In a real implementation, this would call an external service like Vertex AI API.
pub fn generate_embedding(text: &str) -> Embedding {
    // For demonstration, we'll generate a deterministic, non-random vector based on the text's length.
    // THIS IS A SIMULATION. A real model would be used here.
    let mut vec = vec![0.0; 128]; // Using a smaller dimension for the example
    vec[0] = text.len() as f32;
    vec
}