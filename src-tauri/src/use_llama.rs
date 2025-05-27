use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize, Debug)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct LlamaOuterResponse {
    response: String, // This is actually a JSON string
    done: bool,
}

#[derive(Debug, Deserialize)]
pub struct File {
    className: String,
    pub filePath: String,
    pub fileContent: String,
}

#[derive(Debug, Deserialize)]
pub struct ParsedResponse {
    pub files: Vec<File>,
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
Your task is to use the input natural language query, along with the provided code file context, to generate new C++ class files in JSON format.

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
		}}
	]
}}

### Rules
- Each `className` must appear in its own file.
- Absolutely DO NOT put more than one class in the same file.
- The `filePath` must be unique for each file.
- The `filePath` should follow this pattern: `src/ClassName.cpp`.
- Make sure to include the parent class files.
- Do NOT reuse any file paths from the context.
- Only respond with VALID JSON, and nothing else.

Your response will be parsed and written directly to disk, so format correctness is critical.
"#,
		context,
        user_query
    )
}

fn remove_trailing_commas(json: &str) -> String {
    // Remove trailing commas before } or ]
    let re = regex::Regex::new(r",\s*([\]}])").unwrap();
    re.replace_all(json, "$1").to_string()
}

/// Sends a prompt to a local Ollama model and returns the full response.

pub async fn query_ollama(
    prompt: &str,
    context_files: &Vec<&str>,
) -> Result<Vec<File>, Box<dyn Error>> {
    let client = Client::new();

    let context = build_structured_files_context(context_files);
    let formatted_prompt = build_structured_file_gen_prompt(prompt, &context);

    let body = OllamaRequest {
        model: "codellama:7b",
        prompt: &formatted_prompt,
        stream: false,
    };

    println!("Sending request to Ollama with body: {:?}", body);

    let res = client
        .post("http://192.168.91.214:2000/api/generate")
        .json(&body)
        .send()
        .await?;

    println!("llama Response: {:?}", res);

    if !res.status().is_success() {
        return Err(format!("HTTP error: {}", res.status()).into());
    }

    let llama_raw_response = res.text().await?;

    // Step 1: Deserialize outer JSON
    let outer: LlamaOuterResponse = match serde_json::from_str(&llama_raw_response) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to deserialize outer response: {}", e);
            eprintln!("Raw response: {}", llama_raw_response);
            return Err(Box::new(e));
        }
    };

    // Step 2: Extract and clean the JSON portion from the inner response string
    let json_start = outer.response.find('{');
    if json_start.is_none() {
        eprintln!("No JSON object found in inner response");
        eprintln!("Inner response string: {}", outer.response);
        return Err("No JSON object found in inner response".into());
    }
    let json_str = &outer.response[json_start.unwrap()..];

    // Remove trailing commas before parsing
    let cleaned_json = remove_trailing_commas(json_str);

    let inner: ParsedResponse = match serde_json::from_str(&cleaned_json) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Failed to deserialize cleaned inner response: {}", e);
            eprintln!("Cleaned inner response string: {}", cleaned_json);
            return Err(Box::new(e));
        }
    };

    println!("{:#?}", inner);

    Ok(inner.files)
}

