use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    done: Option<bool>,
}

fn build_structured_prompt(user_query: &str) -> String {
    format!(
        r#"
You are a helpful AI assistant for a code generation app.

Your task is to parse natural language queries into structured JSON instructions for generating cpp files.

### Input
{}

### Output Format (JSON)
{{
  "className": "ClassName",
  "filePath": "filePath for this class",
  "fileContent": "full fill content",
}}

Respond ONLY with valid JSON and nothing else. class Name should contain the name of the class that is generated, filePath should contain the path where the class is saved, and fileContent should contain the full content of the file and nothing more.
        "#,
        user_query
    )
}

/// Sends a prompt to a local Ollama model and returns the full response.
pub fn query_ollama(prompt: &str) -> Result<String, Box<dyn Error>> {
	let client = Client::new();

  let formatted_prompt = build_structured_prompt(prompt);


	let body = OllamaRequest {
			model: "gemma3:1b",
			prompt: &formatted_prompt,
			stream: false,
	};

	let res = client
			.post("http://192.168.229.214:2000/api/generate")
			.json(&body)
			.send()?;


	if !res.status().is_success() {
		return Err(format!("HTTP error: {}", res.status()).into());
	}
	println!("Response status: {}", res.status());

	// Parse the JSON response
	let parsed: OllamaResponse = res.json()?;
	println!("Response text: {}", parsed.response);

	Ok(parsed.response)
}
