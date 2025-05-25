use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

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

fn build_structured_files_context(
	context_files: &Vec<&str>,
) -> String {

	#[derive(Serialize)]
	struct FileContext<'a> {
		filePath: &'a str,
		fileContent: String,
	}

	let mut files = Vec::new();

	for file in context_files {
		let file_path = file.clone();
		let content = fs::read_to_string(file).unwrap_or_else(|_| "".to_string());
		files.push(FileContext {
			filePath: file_path,
			fileContent: content,
		});
	}

	serde_json::to_string(&files).unwrap_or_else(|_| "[]".to_string())
}

fn build_structured_file_gen_prompt(
	user_query: &str,
	context: &str,
) -> String {
    format!(
        r#"
You are a helpful AI assistant for code file generation.
Your task is to use natural language queries along with context of files that are already present in project to generate structured JSON for generating cpp files.

### Context
{}
### Input
{}

### Output Format (JSON)
{{
	"files": [
		{{
			"className": "Class1",
  		"filePath": "filePath for Class1",
  		"fileContent": "full fill text for this class file"
		}},
	]
}}

Respond ONLY with valid JSON and nothing else.
Do not re write any class that is already given in the context, only write new class files content.
        "#,
				context,
        user_query
    )
}

/// Sends a prompt to a local Ollama model and returns the full response.

pub async fn query_ollama(
    prompt: &str,
    context_files: &Vec<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();

    let context = build_structured_files_context(context_files);
    let formatted_prompt = build_structured_file_gen_prompt(prompt, &context);

    let body = OllamaRequest {
        model: "gemma3:1b",
        prompt: &formatted_prompt,
        stream: false,
    };

    let res = client
        .post("http://192.168.229.214:2000/api/generate")
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("HTTP error: {}", res.status()).into());
    }

    let parsed: OllamaResponse = res.json().await?;
    Ok(parsed.response)
}

