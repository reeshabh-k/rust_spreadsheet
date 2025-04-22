use reqwest::Client;

use serde_json::json;
#[derive(Clone)]
pub struct CohereChat {
    client: Client,
    api_key: String,
}

impl CohereChat {
    pub fn new(api_key: &str) -> Self {
        CohereChat {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn ask_cohere(&self, prompt: &str) -> String {
        // self.conversation.push(format!("User: {}", prompt));
        // let full_prompt = self.conversation.join("\n");
        // println!("Full prompt: {}", full_prompt);  // Print the full prompt
        let response = self.client.post("https://api.cohere.ai/generate")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "prompt": prompt,
                "max_tokens": 200,
                "stop_sequences": ["\n"],
                "temperature": 0.5,
            })).send()
            .await
            .unwrap();
        let json : serde_json::Value = response.json().await.unwrap();
        if let Some(text) = json["text"].as_str() {
            // self.conversation.push(format!("Cohere: {}", text));
            println!("Cohere: {}", text);  // Print the response text
            text.to_string()
        } else {
            "Failed to extract response text".to_string()
        }
    }
}

// #[tokio::main]
// async fn main() {
//     let cohere_api_key = "tyQev2QzfLWJpuhi041QeENIqhuI1rK1caEELTmi";
//     let mut cohere_chat = CohereChat::new(cohere_api_key);

//     loop {
//         let mut input = String::new();
//         println!("You: ");
//         std::io::stdin().read_line(&mut input).unwrap();
//         let input = input.trim();
//         if input.is_empty() {
//             break;
//         }
//         cohere_chat.ask_cohere(input).await;
//     }
//     println!("Goodbye!");
// }
