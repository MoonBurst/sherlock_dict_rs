
use serde::{Deserialize, Serialize};
use std::env;
use surf;
use tokio; 
#[derive(Debug, Serialize, Deserialize)]
struct DefinitionResponse {
    word: String,
    phonetic: Option<String>,
    phonetics: Vec<Phonetic>,
    meanings: Vec<Meaning>,
    source_urls: Option<Vec<String>>,
    origin: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Phonetic {
    text: Option<String>,
    audio: Option<String>,
    source_url: Option<String>,
    license: Option<License>,
}

#[derive(Debug, Serialize, Deserialize)]
struct License {
    name: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Meaning {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: String,
    definitions: Vec<Definition>,
    synonyms: Option<Vec<String>>,
    antonyms: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Definition {
    definition: String,
    example: Option<String>,
    synonyms: Option<Vec<String>>,
    antonyms: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiErrorResponse {
    title: String,
    message: String,
    resolution: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SherlockPipeResponse {
    title: String,
    content: String,
    next_content: String,
}



impl DefinitionResponse {
    fn format_content_for_sherlock(&self) -> String {
        let mut content_buffer = String::new();

        // Iterate through each meaning and format it
        content_buffer.push_str("<span font_desc=\"monospace\">\n");

        for meaning in &self.meanings {
            content_buffer.push_str(&format!("─── <b><i>{}</i></b> ───\n\n", meaning.part_of_speech));
            for (i, def) in meaning.definitions.iter().enumerate() {
                content_buffer.push_str(&format!(" {:>2}. {}\n", i + 1, def.definition));
                if let Some(example) = &def.example {
                    content_buffer.push_str(&format!("     Example: \"{}\"\n", example));
                }
                if let Some(synonyms) = &def.synonyms {
                    if !synonyms.is_empty() {
                        content_buffer.push_str(&format!("     Synonyms: {}\n", synonyms.join(", ")));
                    }
                }
                if let Some(antonyms) = &def.antonyms {
                    if !antonyms.is_empty() {
                        content_buffer.push_str(&format!("     Antonyms: {}\n", antonyms.join(", ")));
                    }
                }
                content_buffer.push_str("\n");
            }
        }
        content_buffer.push_str("────────────\n");
        content_buffer.push_str("</span>");

        content_buffer
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: No word provided. Usage: sherlock-dictionary <word_to_define>");
        std::process::exit(1);
    }

    let word_to_define = &args[1];
    let definition_url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        word_to_define
    );

    let mut response = surf::get(&definition_url).await?;
    let status = response.status(); 
    let body_text = response.body_string().await?; 


    if status.is_success() {
        // Attempt to parse the response as a vector of DefinitionResponse (successful case).
        match serde_json::from_str::<Vec<DefinitionResponse>>(&body_text) {
            Ok(definitions) => {
                if definitions.is_empty() {
                    eprintln!("No definition found for '{}'.", word_to_define);
                    // Output a simplified "No definition found" for Sherlock
                    let sherlock_error_response = SherlockPipeResponse {
                        title: "No definition found".to_string(),
                        content: String::new(), // Empty content for a concise message
                        next_content: String::new(),
                    };
                    println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
                } else {
                    // Consolidate all definitions into a single content string
                    let mut all_definitions_content = String::new();
                    for def_response in definitions {
                        all_definitions_content.push_str(&def_response.format_content_for_sherlock());
                    }

                    // Create a single SherlockPipeResponse with all content
                    let sherlock_response = SherlockPipeResponse {
                        title: format!(r#"Definition of "{}""#, word_to_define),
                        content: all_definitions_content.clone(),
                        next_content: all_definitions_content, // Populate if Sherlock supports pagination
                    };
                    println!("{}", serde_json::to_string(&sherlock_response).unwrap());
                }
            }
            Err(e) => {
                // If parsing as Vec<DefinitionResponse> failed, it might be an error object
                // even if the status was 200 OK (less common, but possible for "not found"
                // if the API returns a 200 with an error payload).
                match serde_json::from_str::<ApiErrorResponse>(&body_text) {
                    Ok(api_error) => {
                        // Check if the API error indicates "No Definitions Found"
                        if api_error.title == "No Definitions Found" {
                            eprintln!("No definition found for '{}'.", word_to_define);
                            let sherlock_error_response = SherlockPipeResponse {
                                title: "No definition found".to_string(),
                                content: String::new(), // Empty content for a concise message
                                next_content: String::new(),
                            };
                            println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
                        } else {
                            // For other API errors, output the detailed message
                            eprintln!("API Error: {}", api_error.title);
                            eprintln!("Message: {}", api_error.message);
                            eprintln!("Resolution: {}", api_error.resolution);
                            let sherlock_error_response = SherlockPipeResponse {
                                title: format!("API Error: {}", api_error.title),
                                content: format!("Message: {}\nResolution: {}", api_error.message, api_error.resolution),
                                next_content: String::new(),
                            };
                            println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
                        }
                    }
                    Err(_) => {
                        // If it's neither a definition array nor a known error object,
                        // print the raw body and the original parsing error for debugging.
                        eprintln!("Failed to parse API response for '{}'.", word_to_define);
                        eprintln!("Raw response body: {}", body_text);
                        eprintln!("Parsing error: {}", e);
                        // Output generic parsing error as JSON for Sherlock
                        let sherlock_error_response = SherlockPipeResponse {
                            title: format!("Parsing Error for '{}'", word_to_define),
                            content: format!("Failed to parse API response. Raw body: {}", body_text),
                            next_content: String::new(),
                        };
                        println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
                    }
                }
            }
        }
    } else {
        // Handle non-success HTTP status codes (e.g., 404 Not Found, 500 Internal Server Error).
        // In these cases, the body is often an error object.
        match serde_json::from_str::<ApiErrorResponse>(&body_text) {
            Ok(api_error) => {
                // Check if the API error indicates "No Definitions Found"
                if api_error.title == "No Definitions Found" {
                    eprintln!("No definition found for '{}'.", word_to_define);
                    let sherlock_error_response = SherlockPipeResponse {
                        title: "No definition found".to_string(),
                        content: String::new(), // Empty content for a concise message
                        next_content: String::new(),
                    };
                    println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
                } else {
                    // For other API errors, output the detailed message
                    eprintln!("API Error (Status {}): {}", status, api_error.title);
                    eprintln!("Message: {}", api_error.message);
                    eprintln!("Resolution: {}", api_error.resolution);
                    let sherlock_error_response = SherlockPipeResponse {
                        title: format!("API Error (Status {}): {}", status, api_error.title),
                        content: format!("Message: {}\nResolution: {}", api_error.message, api_error.resolution),
                        next_content: String::new(),
                    };
                    println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
                }
            }
            Err(e) => {
                // If the status is not successful, and we can't parse it into our
                // known error format, print a generic error with the raw body.
                eprintln!("Error fetching definition for '{}'.", word_to_define);
                eprintln!("HTTP Status: {}", status);
                eprintln!("Failed to parse error response: {}", e);
                eprintln!("Raw response body: {}", body_text);
                // Output generic HTTP error as JSON for Sherlock
                let sherlock_error_response = SherlockPipeResponse {
                    title: format!("HTTP Error (Status {}) for '{}'", status, word_to_define),
                    content: format!("Failed to parse error response. Raw body: {}", body_text),
                    next_content: String::new(),
                };
                println!("{}", serde_json::to_string(&sherlock_error_response).unwrap());
            }
        }
    }

    // Return Ok(()) to indicate successful execution.
    Ok(())
}
