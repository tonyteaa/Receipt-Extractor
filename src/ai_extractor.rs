use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::debug_log::log as log_to_file;
use crate::document_processor::ExtractedItem;

/// Sanitize error messages to be more user-friendly
/// Removes technical jargon and provides clearer explanations
fn sanitize_error_message(error: &str) -> String {
    let error_lower = error.to_lowercase();

    // Rate limiting errors
    if error_lower.contains("resource_exhausted") || error_lower.contains("429") || error_lower.contains("rate limit") {
        return "Rate limit reached. The API is temporarily limiting requests. Please wait a moment and try again.".to_string();
    }

    // Quota/billing errors
    if error_lower.contains("quota") || error_lower.contains("billing") || error_lower.contains("insufficient_funds") {
        return "API quota exceeded or billing issue. Please check your API account and billing status.".to_string();
    }

    // Authentication errors
    if error_lower.contains("401") || error_lower.contains("unauthorized") || error_lower.contains("invalid_api_key") || error_lower.contains("invalid api key") {
        return "Invalid API key. Please check that your API key is correct and active.".to_string();
    }

    // Forbidden/permission errors
    if error_lower.contains("403") || error_lower.contains("forbidden") || error_lower.contains("permission") {
        return "Access denied. Your API key may not have the required permissions for this model.".to_string();
    }

    // Model not found
    if error_lower.contains("model not found") || error_lower.contains("model_not_found") {
        return "The selected AI model is not available. Try switching to a different model or provider.".to_string();
    }

    // Content safety / moderation
    if error_lower.contains("content_policy") || error_lower.contains("moderation") || error_lower.contains("safety") {
        return "Content was flagged by the AI's safety filter. The receipt may contain content the AI cannot process.".to_string();
    }

    // Timeout errors
    if error_lower.contains("timeout") || error_lower.contains("timed out") {
        return "Request timed out. The AI service is slow to respond. Please try again.".to_string();
    }

    // Connection errors
    if error_lower.contains("connection") || error_lower.contains("network") || error_lower.contains("dns") {
        return "Connection error. Please check your internet connection and try again.".to_string();
    }

    // Server errors
    if error_lower.contains("500") || error_lower.contains("502") || error_lower.contains("503") || error_lower.contains("internal server") {
        return "The AI service is experiencing issues. Please try again in a few minutes.".to_string();
    }

    // If no specific pattern matched, truncate and clean up the message
    let cleaned = error
        .replace("\\n", " ")
        .replace("\\\"", "\"")
        .chars()
        .take(200)
        .collect::<String>();

    if cleaned.len() < error.len() {
        format!("{}...", cleaned)
    } else {
        cleaned
    }
}

// ============================================================================
// OPENAI STRUCTS (KEPT FOR REFERENCE - CURRENTLY COMMENTED OUT)
// ============================================================================
/*
#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: MessageContent,
}

#[derive(Serialize)]
#[serde(untagged)]
enum MessageContent {
    Text(String),
    MultiPart(Vec<ContentPart>),
}

#[derive(Serialize)]
struct ContentPart {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<ImageUrl>,
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}
*/



// ============================================================================
// GROQ STRUCTS (ACTIVE - FREE & FAST!)
// ============================================================================

#[derive(Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct GroqMessage {
    role: String,
    content: GroqMessageContent,
}

#[derive(Serialize)]
#[serde(untagged)]
enum GroqMessageContent {
    Text(String),
    MultiPart(Vec<GroqContentPart>),
}

#[derive(Serialize)]
struct GroqContentPart {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<GroqImageUrl>,
}

#[derive(Serialize)]
struct GroqImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct GroqResponse {
    choices: Vec<GroqChoice>,
}

#[derive(Deserialize)]
struct GroqChoice {
    message: GroqResponseMessage,
}

#[derive(Deserialize)]
struct GroqResponseMessage {
    content: String,
}

// ============================================================================
// CLAUDE STRUCTS
// ============================================================================

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: Vec<ClaudeContentPart>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ClaudeContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ClaudeImageSource },
}

#[derive(Serialize)]
struct ClaudeImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeResponseContent>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClaudeResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
}

// ============================================================================
// GEMINI STRUCTS
// ============================================================================

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiGenerationConfig,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    InlineData { inline_data: GeminiInlineData },
}

#[derive(Serialize)]
struct GeminiInlineData {
    mime_type: String,
    data: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}

pub struct AIExtractor {
    api_key: String,
    client: Client,
    use_latest_models: bool,
    /// Custom extraction prompts for specific fields (field_name -> prompt)
    /// Used for AI-enhanced custom fields
    custom_prompts: std::collections::HashMap<String, String>,
    /// Local LLM base URL (e.g. "http://localhost:11434"). Set via with_local_llm().
    local_llm_url: Option<String>,
    /// Local LLM model name (e.g. "llava", "llama3.2-vision"). Set via with_local_llm().
    local_llm_model: Option<String>,
}

impl AIExtractor {
    pub fn new_with_options(api_key: String, use_latest_models: bool) -> Self {
        // Create HTTP client with timeout to prevent hanging on slow connections
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // 2 minute timeout for large PDFs
            .connect_timeout(std::time::Duration::from_secs(30)) // 30s connection timeout
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            api_key,
            client,
            use_latest_models,
            custom_prompts: std::collections::HashMap::new(),
            local_llm_url: None,
            local_llm_model: None,
        }
    }

    /// Configure this extractor to use a local LLM (Ollama, LM Studio, Jan, etc.)
    /// The url is the base URL (e.g. "http://localhost:11434") and model is the model name (e.g. "llava").
    pub fn with_local_llm(mut self, url: String, model: String) -> Self {
        self.local_llm_url = Some(url);
        self.local_llm_model = Some(model);
        self
    }

    /// Set custom extraction prompts for specific fields
    pub fn set_custom_prompts(&mut self, prompts: std::collections::HashMap<String, String>) {
        self.custom_prompts = prompts;
    }

    // Detect which AI provider based on API key prefix or local LLM configuration
    fn is_local_llm(&self) -> bool {
        self.local_llm_url.is_some()
    }

    fn is_groq(&self) -> bool {
        !self.is_local_llm() && self.api_key.starts_with("gsk_")
    }

    fn is_openai(&self) -> bool {
        !self.is_local_llm() && self.api_key.starts_with("sk-") && !self.is_claude() && !self.is_openrouter()
    }

    fn is_claude(&self) -> bool {
        !self.is_local_llm() && self.api_key.starts_with("sk-ant-")
    }

    fn is_gemini(&self) -> bool {
        !self.is_local_llm() && self.api_key.starts_with("AIza")
    }

    fn is_openrouter(&self) -> bool {
        !self.is_local_llm() && self.api_key.starts_with("sk-or-")
    }

    fn get_api_endpoint(&self) -> String {
        if self.is_local_llm() {
            // Local LLMs (Ollama, LM Studio, Jan) expose an OpenAI-compatible endpoint
            let base = self.local_llm_url.as_deref().unwrap_or("http://localhost:11434");
            format!("{}/v1/chat/completions", base.trim_end_matches('/'))
        } else if self.is_groq() {
            "https://api.groq.com/openai/v1/chat/completions".to_string()
        } else if self.is_openai() {
            "https://api.openai.com/v1/chat/completions".to_string()
        } else if self.is_claude() {
            "https://api.anthropic.com/v1/messages".to_string()
        } else if self.is_gemini() {
            // Gemini endpoint is dynamic based on model, will be constructed in send method
            "https://generativelanguage.googleapis.com/v1beta/models".to_string()
        } else if self.is_openrouter() {
            "https://openrouter.ai/api/v1/chat/completions".to_string()
        } else {
            "https://api.groq.com/openai/v1/chat/completions".to_string() // Default to Groq
        }
    }

    fn get_model_name(&self) -> String {
        if self.is_local_llm() {
            // Use whatever model the user configured (default to llava for vision)
            self.local_llm_model.clone().unwrap_or_else(|| "llava".to_string())
        } else if self.is_groq() {
            if self.use_latest_models {
                "llama-3.3-70b-versatile".to_string()  // Latest: Faster, newer model
            } else {
                "meta-llama/llama-4-scout-17b-16e-instruct".to_string()  // Stable: Vision-capable model
            }
        } else if self.is_openai() {
            if self.use_latest_models {
                "gpt-4o".to_string()  // Latest: Better quality, more expensive
            } else {
                "gpt-4o-mini".to_string()  // Stable: Cost-effective, good quality
            }
        } else if self.is_claude() {
            if self.use_latest_models {
                "claude-3-5-sonnet-latest".to_string()  // Latest: Newest version
            } else {
                "claude-3-5-sonnet-20241022".to_string()  // Stable: Specific version from Oct 2024
            }
        } else if self.is_gemini() {
            if self.use_latest_models {
                "gemini-flash-latest".to_string()  // Latest: Newest version - may change
            } else {
                "gemini-2.5-flash".to_string()  // Stable: Specific version from Jan 2026
            }
        } else if self.is_openrouter() {
            if self.use_latest_models {
                "openai/gpt-4o".to_string()  // Latest: Premium model via OpenRouter
            } else {
                "meta-llama/llama-3.2-11b-vision-instruct".to_string()  // Stable: Free/cheap vision model
            }
        } else {
            "meta-llama/llama-4-scout-17b-16e-instruct".to_string() // Default to Groq stable
        }
    }

    // ========================================================================
    // OPENAI METHODS (COMMENTED OUT - KEPT FOR REFERENCE)
    // ========================================================================
    /*
    pub async fn extract_from_text(
        &self,
        text: &str,
        fields: &[String],
    ) -> Result<HashMap<String, String>> {
        let prompt = self.create_extraction_prompt(fields);
        let full_prompt = format!("{}\n\nDocument text:\n{}", prompt, text);

        log_to_file("Sending request to OpenAI API...");
        let msg = format!("Fields to extract: {:?}", fields);
        log_to_file(&msg);

        let request = OpenAIRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::Text(full_prompt),
            }],
            temperature: 0.1,
        };

        let response = self.send_request(request).await?;
        let msg = format!("Received response from OpenAI: {}", response);
        log_to_file(&msg);
        self.parse_response(&response, fields)
    }

    pub async fn extract_from_image(
        &self,
        base64_image: &str,
        fields: &[String],
    ) -> Result<HashMap<String, String>> {
        let prompt = self.create_extraction_prompt(fields);
        log_to_file(&format!("Vision API prompt: {}", prompt));
        let image_url = format!("data:image/png;base64,{}", base64_image);

        let request = OpenAIRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::MultiPart(vec![
                    ContentPart {
                        content_type: "text".to_string(),
                        text: Some(prompt),
                        image_url: None,
                    },
                    ContentPart {
                        content_type: "image_url".to_string(),
                        text: None,
                        image_url: Some(ImageUrl { url: image_url }),
                    },
                ]),
            }],
            temperature: 0.1,
        };

        let response = self.send_request(request).await?;
        self.parse_response(&response, fields)
    }
    */



    // ========================================================================
    // GROQ/OPENAI METHODS (ACTIVE - SUPPORTS BOTH!)
    // ========================================================================

    pub async fn extract_from_text(
        &self,
        text: &str,
        fields: &[String],
    ) -> Result<HashMap<String, String>> {
        let prompt = self.create_extraction_prompt(fields);
        let full_prompt = format!("{}\n\nDocument text:\n{}", prompt, text);

        // Route to the correct provider based on API key
        let response = if self.is_claude() {
            log_to_file("Sending text extraction request to Claude API...");
            log_to_file(&format!("Fields to extract: {:?}", fields));

            let request = ClaudeRequest {
                model: self.get_model_name().to_string(),
                max_tokens: 4096,  // Increased to prevent JSON truncation for long item lists
                messages: vec![ClaudeMessage {
                    role: "user".to_string(),
                    content: vec![ClaudeContentPart::Text { text: full_prompt }],
                }],
            };

            let response = self.client
                .post(self.get_api_endpoint())
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let status = response.status();
            log_to_file(&format!("Claude API response status: {}", status));

            if !status.is_success() {
                let error_text = response.text().await?;
                log_to_file(&format!("Claude API error: {}", error_text));
                anyhow::bail!("{}", sanitize_error_message(&error_text));
            }

            let api_response: ClaudeResponse = response.json().await?;
            if let Some(ClaudeResponseContent::Text { text }) = api_response.content.first() {
                text.clone()
            } else {
                anyhow::bail!("No response from Claude. The AI didn't return any text.")
            }
        } else if self.is_gemini() {
            log_to_file("Sending text extraction request to Gemini API...");
            log_to_file(&format!("Fields to extract: {:?}", fields));

            let request = GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![GeminiPart::Text { text: full_prompt }],
                }],
                generation_config: GeminiGenerationConfig {
                    temperature: 0.1,
                    max_output_tokens: 4096,  // Increased to prevent JSON truncation for long item lists
                },
            };

            let model_name = self.get_model_name();
            let endpoint = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model_name, self.api_key
            );

            let response = self.client
                .post(&endpoint)
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let status = response.status();
            log_to_file(&format!("Gemini API response status: {}", status));

            if !status.is_success() {
                let error_text = response.text().await?;
                log_to_file(&format!("Gemini API error: {}", error_text));
                anyhow::bail!("{}", sanitize_error_message(&error_text));
            }

            let api_response: GeminiResponse = response.json().await?;
            if let Some(candidate) = api_response.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    part.text.clone()
                } else {
                    anyhow::bail!("No response from Gemini. The AI didn't return any text.")
                }
            } else {
                anyhow::bail!("No response from Gemini. The AI didn't return any text.")
            }
        } else if self.is_openrouter() || self.is_groq() || self.is_openai() || self.is_local_llm() {
            // OpenRouter, Groq, OpenAI, and Local LLMs all use OpenAI-compatible API
            let provider = if self.is_local_llm() { "Local LLM" } else if self.is_groq() { "Groq" } else if self.is_openai() { "OpenAI" } else { "OpenRouter" };
            log_to_file(&format!("Sending text extraction request to {} API...", provider));
            log_to_file(&format!("Fields to extract: {:?}", fields));

            let request = GroqRequest {
                model: self.get_model_name().to_string(),
                messages: vec![GroqMessage {
                    role: "user".to_string(),
                    content: GroqMessageContent::Text(full_prompt),
                }],
                temperature: 0.1,
                max_tokens: 4096,  // Increased to prevent JSON truncation for long item lists
            };

            let response = self.send_groq_request(request).await?;
            log_to_file(&format!("Received response from {}: {}", provider, response));
            response
        } else {
            anyhow::bail!("Unsupported AI provider")
        };

        self.parse_response(&response, fields)
    }

    pub async fn extract_from_image(
        &self,
        base64_image: &str,
        fields: &[String],
    ) -> Result<HashMap<String, String>> {
        // Just call extract_from_images with a single image
        self.extract_from_images(&[base64_image.to_string()], fields).await
    }

    pub async fn extract_from_images(
        &self,
        base64_images: &[String],
        fields: &[String],
    ) -> Result<HashMap<String, String>> {
        let prompt = self.create_extraction_prompt(fields);

        // Debug: Log which provider is being detected
        log_to_file(&format!("API Key prefix: {}", &self.api_key.chars().take(10).collect::<String>()));
        log_to_file(&format!("is_gemini: {}, is_claude: {}, is_groq: {}, is_openai: {}, is_openrouter: {}",
            self.is_gemini(), self.is_claude(), self.is_groq(), self.is_openai(), self.is_openrouter()));

        // Route to appropriate provider
        let response = if self.is_claude() {
            log_to_file("Claude Vision API prompt (multi-page)");
            self.send_claude_request(prompt, base64_images).await?
        } else if self.is_gemini() {
            log_to_file("Gemini Vision API prompt (multi-page)");
            self.send_gemini_request(prompt, base64_images).await?
        } else if self.is_openrouter() || self.is_groq() || self.is_openai() || self.is_local_llm() {
            // OpenRouter, Groq, OpenAI, and Local LLMs all use OpenAI-compatible API
            let provider = if self.is_local_llm() { "Local LLM" } else if self.is_groq() { "Groq" } else if self.is_openai() { "OpenAI" } else { "OpenRouter" };
            log_to_file(&format!("{} Vision API prompt (multi-page): {}", provider, prompt));

            // Build content parts: first the text prompt, then all images
            let mut content_parts = vec![
                GroqContentPart {
                    content_type: "text".to_string(),
                    text: Some(prompt.clone()),
                    image_url: None,
                }
            ];

            // Add all images (support both PNG and JPEG)
            for (idx, base64_image) in base64_images.iter().enumerate() {
                // Auto-detect format based on base64 header or default to JPEG for multi-page
                let image_url = if base64_images.len() > 1 {
                    format!("data:image/jpeg;base64,{}", base64_image)
                } else {
                    format!("data:image/png;base64,{}", base64_image)
                };
                content_parts.push(GroqContentPart {
                    content_type: "image_url".to_string(),
                    text: None,
                    image_url: Some(GroqImageUrl { url: image_url }),
                });
                log_to_file(&format!("Added page {} to vision request", idx + 1));
            }

            let request = GroqRequest {
                model: self.get_model_name().to_string(),
                messages: vec![GroqMessage {
                    role: "user".to_string(),
                    content: GroqMessageContent::MultiPart(content_parts),
                }],
                temperature: 0.1,
                max_tokens: 4096,  // Increased to prevent JSON truncation for long item lists
            };

            self.send_groq_request(request).await?
        } else {
            anyhow::bail!("Unsupported AI provider")
        };

        log_to_file(&format!("Received response: {}", response));
        self.parse_response(&response, fields)
    }

    fn create_extraction_prompt(&self, fields: &[String]) -> String {
        // Use model-specific prompts for better results
        if self.is_gemini() {
            self.create_gemini_prompt(fields)
        } else if self.is_claude() {
            self.create_claude_prompt(fields)
        } else {
            // Groq, OpenAI, OpenRouter use the detailed prompt
            self.create_detailed_prompt(fields)
        }
    }

    // Gemini-optimized prompt: Clear examples and concise instructions
    fn create_gemini_prompt(&self, fields: &[String]) -> String {
        let fields_json = fields.iter()
            .map(|f| format!("  \"{}\": \"\"", f))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            "You are extracting data from a receipt image. Extract these fields: {}\n\n\
            INSTRUCTIONS:\n\
            1. Look carefully at the entire receipt\n\
            2. Extract the exact values you see\n\
            3. For dates: use DD/MM/YYYY format (e.g., '15/01/2024' for January 15, 2024)\n\
            4. For amounts: return only numbers without $ or currency symbols (e.g., '45.99' not '$45.99')\n\
            5. For tax: if you see multiple taxes (GST + PST), add them together\n\
            6. For items: extract the full product name/description\n\
            7. If a field is not found, use empty string \"\"\n\n\
            EXAMPLE OUTPUT FORMAT:\n\
            {{\n\
            {}\n\
            }}\n\n\
            Now extract the data from this receipt and return ONLY the JSON object:",
            fields.join(", "),
            fields_json
        )
    }

    // Claude-optimized prompt: Structured thinking with examples
    fn create_claude_prompt(&self, fields: &[String]) -> String {
        let fields_json = fields.iter()
            .map(|f| format!("  \"{}\": \"\"", f))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            "Extract the following fields from this receipt: {}\n\n\
            APPROACH:\n\
            1. Scan the entire receipt carefully\n\
            2. Identify each requested field\n\
            3. Extract the exact value as shown\n\n\
            KEY RULES:\n\
            - Dates: DD/MM/YYYY format (e.g., '05/01/2024' for Jan 5)\n\
            - Amounts: Numbers only, no symbols (e.g., '45.99' not '$45.99')\n\
            - Tax: Sum all tax lines (GST + PST + HST, etc.)\n\
            - Items: Full product name with brand/model if visible\n\
            - Empty fields: Use \"\" if not found\n\n\
            DOCUMENT TYPES:\n\
            - Regular receipt: Extract as shown\n\
            - Refund: Use negative amounts (e.g., '-45.99') and prefix items with 'REFUND:'\n\
            - e-Transfer: Use recipient name as vendor, 'Interac e-Transfer' for items\n\n\
            Return ONLY a JSON object:\n\
            {{\n\
            {}\n\
            }}",
            fields.join(", "),
            fields_json
        )
    }

    // Detailed prompt for Groq/OpenAI/OpenRouter (they handle complexity well)
    fn create_detailed_prompt(&self, fields: &[String]) -> String {
        let fields_list = fields.join(", ");

        // Build field-specific instructions
        let mut field_instructions = String::new();
        let mut has_item_field = false;
        let mut has_date_field = false;
        let mut has_amount_field = false;

        // Add document type detection instruction at the beginning
        field_instructions.push_str(
            "\n\nDOCUMENT TYPES:\n\
            - Regular receipt/invoice (may include discounts)\n\
            - REFUND receipt (money returned - use negative amounts like '-45.99' and prefix items with 'REFUND:')\n\
            - Interac e-Transfer (use recipient name as vendor, write 'Interac e-Transfer' for items)\n\
            - Email containing a receipt (focus on the receipt portion, not email conversation)\n"
        );

        for field in fields {
            let field_lower = field.to_lowercase();
            let mut field_handled = false;

            // FIRST: Check if we have a custom AI-generated prompt for this field
            if let Some(custom_prompt) = self.custom_prompts.get(field) {
                field_instructions.push_str(&format!("\n- '{}': {}", field, custom_prompt));
                continue;  // Skip all keyword-based checks
            }

            // Date field instructions
            if field_lower.contains("date") {
                has_date_field = true;
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Extract transaction/order date in DD/MM/YYYY format (e.g., '05/01/2024' for Jan 5, 2024). NOT the PDF creation date.",
                    field
                ));
            }

            // Vendor/Store field instructions
            if (field_lower.contains("vendor") || field_lower.contains("store")) &&
               !field_lower.contains("seller") && !field_lower.contains("sold by") {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Main marketplace/platform name (e.g., 'Amazon', 'Walmart'). For e-Transfers, use recipient name.",
                    field
                ));
            }

            // Seller field instructions
            if field_lower.contains("seller") || field_lower.contains("sold by") ||
               (field_lower.contains("actual") && field_lower.contains("vendor")) {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Actual seller name (look for 'Sold by' on marketplaces). For e-Transfers, use recipient name.",
                    field
                ));
            }

            // Payment method field instructions
            if field_lower.contains("payment") {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Payment method (e.g., 'Visa ••••1234'). For e-Transfers, use 'Interac e-Transfer'.",
                    field
                ));
            }

            // Amount field instructions (Total, Tax, Subtotal, etc.)
            if field_lower.contains("amount") || field_lower.contains("total") ||
               field_lower.contains("tax") || field_lower.contains("subtotal") {
                has_amount_field = true;
                field_handled = true;

                // Specific instructions for different amount types
                if field_lower.contains("subtotal") || field_lower.contains("before") {
                    field_instructions.push_str(&format!(
                        "\n- '{}': Amount BEFORE tax (usually smaller than total). Return number only (e.g., '100.00').",
                        field
                    ));
                } else if field_lower.contains("tax") {
                    field_instructions.push_str(&format!(
                        "\n- '{}': Find ALL tax lines (GST, PST, HST, etc.) and ADD them together. Return total as number only.",
                        field
                    ));
                } else if field_lower.contains("total") {
                    field_instructions.push_str(&format!(
                        "\n- '{}': FINAL total paid (usually the largest amount). Return number only (e.g., '113.50').",
                        field
                    ));
                } else {
                    field_instructions.push_str(&format!(
                        "\n- '{}': Extract amount as number only (no $ or currency symbols).",
                        field
                    ));
                }
            }

            // Number/Count field instructions (MUST come BEFORE generic item check)
            if (field_lower.contains("number") || field_lower.contains("count") ||
                field_lower.contains("quantity") || field_lower.contains("qty")) &&
               (field_lower.contains("item") || field_lower.contains("product")) {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Return ONLY a number (count of items). Example: '3' not 'Item 1, Item 2, Item 3'.",
                    field
                ));
            }
            // Item field instructions (full description)
            else if field_lower.contains("item") || field_lower.contains("content") ||
               field_lower.contains("object") || field_lower.contains("product") ||
               field_lower.contains("purchase") || field_lower.contains("material") ||
               field_lower.contains("tool") {
                has_item_field = true;
                field_handled = true;

                // Check if this is a summary field
                if field_lower.contains("summary") || field_lower.contains("short") ||
                   field_lower.contains("brief") || field_lower.contains("condensed") {
                    field_instructions.push_str(&format!(
                        "\n- '{}': Short item summary (under 5 words). For e-Transfers: 'Interac e-Transfer'. For refunds: prefix with 'REFUND:'.",
                        field
                    ));
                } else {
                    field_instructions.push_str(&format!(
                        "\n- '{}': Full product name/description with brand, model, size. Multiple items separated by commas. For e-Transfers: 'Interac e-Transfer to [Name]'. For refunds: prefix with 'REFUND:'.",
                        field
                    ));
                }
            }

            // Transaction type fields (purchase, refund, return, etc.)
            if !field_handled && (field_lower.contains("refund") || field_lower.contains("return") ||
               field_lower.contains("type") || field_lower.contains("transaction")) {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Determine the transaction type by looking for keywords like 'REFUND', 'RETURN', 'CREDIT', 'PURCHASE', or negative amounts. Answer concisely (e.g., 'Purchase', 'Refund', 'Return').",
                    field
                ));
            }

            // Category/classification fields
            if !field_handled && (field_lower.contains("category") || field_lower.contains("class") ||
               field_lower.contains("department") || field_lower.contains("section")) {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Categorize the purchase based on the items (e.g., 'Electronics', 'Groceries', 'Office Supplies', 'Clothing').",
                    field
                ));
            }

            // Note/comment/description fields
            if !field_handled && (field_lower.contains("note") || field_lower.contains("comment") ||
               field_lower.contains("memo") || field_lower.contains("remark") ||
               field_lower.contains("description") || field_lower.contains("detail")) {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Extract any notes, comments, or additional details visible on the receipt.",
                    field
                ));
            }

            // Reference/ID fields (invoice, order, receipt number)
            if !field_handled && (field_lower.contains("invoice") || field_lower.contains("order") ||
               field_lower.contains("receipt") || field_lower.contains("reference") ||
               field_lower.contains("confirmation") || field_lower.contains("tracking") ||
               (field_lower.contains("number") && !field_lower.contains("item"))) {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Look for order numbers, invoice numbers, receipt numbers, or confirmation codes on the document.",
                    field
                ));
            }

            // Currency field
            if !field_handled && field_lower.contains("currency") {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Identify the currency used (e.g., 'CAD', 'USD', 'EUR'). Look for currency symbols or codes.",
                    field
                ));
            }

            // Location/address fields
            if !field_handled && (field_lower.contains("address") || field_lower.contains("location") ||
               field_lower.contains("city") || field_lower.contains("province") ||
               field_lower.contains("state") || field_lower.contains("country") ||
               field_lower.contains("postal") || field_lower.contains("zip")) {
                field_handled = true;
                field_instructions.push_str(&format!(
                    "\n- '{}': Extract the relevant address or location information from the receipt.",
                    field
                ));
            }

            // FALLBACK: Custom/unknown fields - provide generic but helpful instruction
            if !field_handled {
                // Check if field name is a question (ends with ? or starts with question words)
                let is_question = field.ends_with('?') ||
                    field_lower.starts_with("is ") ||
                    field_lower.starts_with("are ") ||
                    field_lower.starts_with("was ") ||
                    field_lower.starts_with("were ") ||
                    field_lower.starts_with("do ") ||
                    field_lower.starts_with("does ") ||
                    field_lower.starts_with("did ") ||
                    field_lower.starts_with("has ") ||
                    field_lower.starts_with("have ") ||
                    field_lower.starts_with("can ") ||
                    field_lower.starts_with("will ") ||
                    field_lower.starts_with("what ") ||
                    field_lower.starts_with("which ") ||
                    field_lower.starts_with("how ");

                if is_question {
                    field_instructions.push_str(&format!(
                        "\n- '{}': Answer this question based on what you can see in the receipt. Provide a clear, concise answer.",
                        field
                    ));
                } else {
                    field_instructions.push_str(&format!(
                        "\n- '{}': Look for any information related to '{}' in the receipt and extract it. If not found, return empty string.",
                        field, field
                    ));
                }
            }
        }

        // Add concise emphasis sections
        let item_emphasis = if has_item_field {
            "\n\nFor items: Look in order details, item list, or product description sections. Extract actual product names, not generic terms."
        } else {
            ""
        };

        let date_emphasis = if has_date_field {
            "\n\nFor dates: MUST use DD/MM/YYYY format (e.g., '05/01/2024' for Jan 5). Extract transaction date, NOT PDF creation date."
        } else {
            ""
        };

        let amount_emphasis = if has_amount_field {
            "\n\nFor amounts:\n\
            - Return ONLY numbers (e.g., '45.99' not '$45.99')\n\
            - Total = final amount paid (largest number)\n\
            - Subtotal = amount before tax (smaller than total)\n\
            - Tax = SUM of ALL tax lines (GST + PST + HST, etc.)\n\
            - For refunds: use negative numbers (e.g., '-45.99')"
        } else {
            ""
        };

        format!(
            "Extract the following fields from this receipt: {}\n\
            {}{}{}{}\n\n\
            RULES:\n\
            1. Read values directly from the receipt - don't calculate\n\
            2. Return ONLY a JSON object with the extracted values\n\
            3. Use empty string \"\" if a field is not found\n\n\
            EXAMPLE (if receipt shows GST: $5.00 and PST: $8.00):\n\
            {{\n\
              \"Tax Amount\": \"13.00\",\n\
              \"Total Amount\": \"113.00\",\n\
              \"Date\": \"15/01/2024\"\n\
            }}",
            fields_list,
            field_instructions,
            item_emphasis,
            date_emphasis,
            amount_emphasis
        )
    }

    // ========================================================================
    // OPENAI SEND REQUEST (COMMENTED OUT - KEPT FOR REFERENCE)
    // ========================================================================
    /*
    async fn send_request(&self, request: OpenAIRequest) -> Result<String> {
        log_to_file("Making API request to OpenAI...");
        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self._api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let msg = format!("API response status: {}", status);
        log_to_file(&msg);

        if !status.is_success() {
            let error_text = response.text().await?;
            let msg = format!("OpenAI API error: {}", error_text);
            log_to_file(&msg);
            anyhow::bail!("{}", sanitize_error_message(&error_text));
        }

        let api_response: OpenAIResponse = response.json().await?;
        Ok(api_response.choices[0].message.content.clone())
    }
    */



    // ========================================================================
    // GROQ/OPENAI SEND REQUEST (ACTIVE - SUPPORTS BOTH!)
    // ========================================================================

    async fn send_groq_request(&self, request: GroqRequest) -> Result<String> {
        let provider = if self.is_local_llm() { "Local LLM" } else if self.is_groq() { "Groq" } else { "OpenAI" };
        let endpoint = self.get_api_endpoint();

        // Retry logic for rate limits
        let max_retries = 3;
        let mut retry_count = 0;

        loop {
            log_to_file(&format!("Making API request to {}...", provider));

            // Serialize the request for potential retry
            let request_json = serde_json::to_value(&request)?;

            let response = self
                .client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request_json)
                .send()
                .await?;

            let status = response.status();
            let msg = format!("{} API response status: {}", provider, status);
            log_to_file(&msg);

            if status.is_success() {
                let api_response: GroqResponse = response.json().await?;
                return Ok(api_response.choices[0].message.content.clone());
            }

            let error_text = response.text().await?;
            log_to_file(&format!("{} API error: {}", provider, error_text));

            // Check for rate limit (429) and retry with exponential backoff
            if status.as_u16() == 429 && retry_count < max_retries {
                retry_count += 1;
                let wait_secs = retry_count as u64 * 5; // 5s, 10s, 15s (more aggressive backoff)
                log_to_file(&format!("Rate limited, waiting {}s before retry {}/{}...", wait_secs, retry_count, max_retries));
                tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
                continue;
            }

            anyhow::bail!("{}", sanitize_error_message(&error_text));
        }
    }

    // ========================================================================
    // CLAUDE SEND REQUEST
    // ========================================================================

    async fn send_claude_request(&self, prompt: String, base64_images: &[String]) -> Result<String> {
        log_to_file("Making API request to Claude...");

        let mut content_parts = vec![ClaudeContentPart::Text { text: prompt }];

        // Add images
        for base64_image in base64_images {
            content_parts.push(ClaudeContentPart::Image {
                source: ClaudeImageSource {
                    source_type: "base64".to_string(),
                    media_type: "image/png".to_string(),
                    data: base64_image.clone(),
                },
            });
        }

        let request = ClaudeRequest {
            model: self.get_model_name().to_string(),
            max_tokens: 4096,  // Increased to prevent JSON truncation for long item lists
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: content_parts,
            }],
        };

        let response = self
            .client
            .post(self.get_api_endpoint())
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        log_to_file(&format!("Claude API response status: {}", status));

        if !status.is_success() {
            let error_text = response.text().await?;
            log_to_file(&format!("Claude API error: {}", error_text));
            anyhow::bail!("{}", sanitize_error_message(&error_text));
        }

        let api_response: ClaudeResponse = response.json().await?;
        if let Some(ClaudeResponseContent::Text { text }) = api_response.content.first() {
            Ok(text.clone())
        } else {
            anyhow::bail!("No response from Claude. The AI didn't return any text.")
        }
    }

    // ========================================================================
    // GEMINI SEND REQUEST
    // ========================================================================

    async fn send_gemini_request(&self, prompt: String, base64_images: &[String]) -> Result<String> {
        log_to_file("Making API request to Gemini...");

        let mut parts = vec![GeminiPart::Text { text: prompt }];

        // Add images
        for base64_image in base64_images {
            parts.push(GeminiPart::InlineData {
                inline_data: GeminiInlineData {
                    mime_type: "image/png".to_string(),
                    data: base64_image.clone(),
                },
            });
        }

        let request = GeminiRequest {
            contents: vec![GeminiContent { parts }],
            generation_config: GeminiGenerationConfig {
                temperature: 0.1,
                max_output_tokens: 4096,  // Increased to prevent JSON truncation for long item lists
            },
        };

        // Gemini uses a different URL structure
        let model_name = self.get_model_name();
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model_name, self.api_key
        );

        let response = self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        log_to_file(&format!("Gemini API response status: {}", status));

        if !status.is_success() {
            let error_text = response.text().await?;
            log_to_file(&format!("Gemini API error: {}", error_text));
            anyhow::bail!("{}", sanitize_error_message(&error_text));
        }

        let api_response: GeminiResponse = response.json().await?;
        if let Some(candidate) = api_response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }
        anyhow::bail!("No response from Gemini. The AI didn't return any text.")
    }

    fn validate_extraction(&self, result: &mut HashMap<String, String>) {
        // Check if the AI might have calculated the total instead of reading it
        let subtotal = result.get("amount befor taxe")
            .and_then(|s| s.replace(",", "").parse::<f64>().ok());
        let tax = result.get("Tax Amount")
            .and_then(|s| s.replace(",", "").parse::<f64>().ok());
        let total = result.get("Total Amount")
            .and_then(|s| s.replace(",", "").parse::<f64>().ok());

        if let (Some(sub), Some(tax_amt), Some(tot)) = (subtotal, tax, total) {
            let calculated_total = sub + tax_amt;
            let diff = (calculated_total - tot).abs();

            // If total exactly equals subtotal + tax, warn but DON'T auto-fix
            // (The AI might have read the values correctly!)
            if diff < 0.02 {  // Allow 2 cent rounding difference
                let warning = format!(
                    "⚠️ WARNING: Total ({}) = Subtotal ({}) + Tax ({}) - AI may have CALCULATED instead of READING!\n\
                     This could be correct, or the AI might have calculated. Manual review recommended.",
                    tot, sub, tax_amt
                );
                log_to_file(&warning);
                eprintln!("{}", warning);
            }

            // Check if tax seems too low (might be missing PST or GST)
            // For Canadian receipts, combined GST+PST is usually 12-15%
            let tax_percentage = (tax_amt / sub) * 100.0;
            if tax_percentage < 8.0 && tax_percentage > 0.0 {
                let warning = format!(
                    "⚠️ WARNING: Tax ({}) is only {:.1}% of Subtotal ({}) - This seems LOW! \
                    AI likely missed a second tax (PST or GST). Expected ~12-15% for GST+PST.\n\
                    ⚠️ MANUAL REVIEW REQUIRED - Check the receipt for multiple tax lines!",
                    tax_amt, tax_percentage, sub
                );
                log_to_file(&warning);
                eprintln!("{}", warning);
            }
        }
    }

    fn parse_response(
        &self,
        response: &str,
        fields: &[String],
    ) -> Result<HashMap<String, String>> {
        // Log the full AI response for debugging
        log_to_file("========== FULL AI RESPONSE ==========");
        log_to_file(response);
        log_to_file("======================================");

        // Try multiple strategies to extract JSON from the response

        // Strategy 1: Look for JSON inside markdown code blocks (```json ... ```)
        let json_str = if let Some(json_block) = self.extract_json_from_markdown(response) {
            json_block
        } else {
            // Strategy 2: Find the LAST complete JSON object (most likely to be the final answer)
            self.extract_last_json_object(response).unwrap_or_else(|| response.to_string())
        };

        // Safely truncate at character boundary, not byte boundary
        let truncated = json_str.chars().take(200).collect::<String>();
        log_to_file(&format!("Attempting to parse JSON: {}", truncated));

        match serde_json::from_str::<HashMap<String, serde_json::Value>>(&json_str) {
            Ok(parsed) => {
                let mut result = HashMap::new();
                for field in fields {
                    let value = parsed
                        .get(field)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    result.insert(field.clone(), value.clone());

                    // Log each extracted field
                    log_to_file(&format!("  Extracted '{}': '{}'", field, value));
                }

                // Validation and auto-fix: Check if AI calculated instead of reading
                self.validate_extraction(&mut result);

                log_to_file(&format!("Successfully parsed {} fields", result.len()));
                Ok(result)
            }
            Err(e) => {
                eprintln!("Failed to parse AI response as JSON: {}", e);
                eprintln!("Response was: {}", response);
                log_to_file(&format!("JSON parse error: {}", e));

                // Determine the specific error reason
                let error_reason = if response.contains("```json") && !json_str.ends_with('}') {
                    "JSON truncated - response was cut off mid-stream".to_string()
                } else if response.is_empty() {
                    "Empty response from AI".to_string()
                } else if response.contains("rate") || response.contains("429") || response.contains("quota") {
                    "Rate limit exceeded".to_string()
                } else {
                    format!("JSON parse error: {}", e)
                };

                log_to_file(&format!("Extraction error reason: {}", error_reason));

                // Return empty values for all fields, plus a special error field
                let mut result: HashMap<String, String> = fields.iter().map(|f| (f.clone(), String::new())).collect();
                result.insert("__extraction_error__".to_string(), error_reason);
                Ok(result)
            }
        }
    }

    /// Extract JSON from markdown code blocks (```json ... ``` or ``` ... ```)
    fn extract_json_from_markdown(&self, response: &str) -> Option<String> {
        // Look for the LAST ```json block (most likely to be the final answer)
        let mut last_json = None;

        // Try to find ```json blocks first
        for part in response.split("```json") {
            if let Some(end_idx) = part.find("```") {
                let json_content = part[..end_idx].trim();
                if json_content.starts_with('{') && json_content.ends_with('}') {
                    last_json = Some(json_content.to_string());
                }
            }
        }

        // If no ```json blocks, try plain ``` blocks
        if last_json.is_none() {
            for part in response.split("```") {
                let trimmed = part.trim();
                if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    last_json = Some(trimmed.to_string());
                }
            }
        }

        last_json
    }

    /// Extract the last complete JSON object from the response
    fn extract_last_json_object(&self, response: &str) -> Option<String> {
        // Find all { positions and their matching }
        let mut brace_depth = 0;
        let mut json_start: Option<usize> = None;
        let mut last_complete_json: Option<String> = None;

        for (i, c) in response.char_indices() {
            match c {
                '{' => {
                    if brace_depth == 0 {
                        json_start = Some(i);
                    }
                    brace_depth += 1;
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        if let Some(start) = json_start {
                            let potential_json = &response[start..=i];
                            // Validate it's actually parseable JSON
                            if serde_json::from_str::<serde_json::Value>(potential_json).is_ok() {
                                last_complete_json = Some(potential_json.to_string());
                            }
                        }
                        json_start = None;
                    }
                }
                _ => {}
            }
        }

        last_complete_json
    }

    // ========================================================================
    // API KEY TESTING
    // ========================================================================

    /// Test the API key by sending a simple test request
    pub async fn test_api_key(&self) -> Result<String> {
        log_to_file("Testing API key with a simple request...");

        // Create a simple test prompt
        let test_prompt = "Respond with exactly: 'API key is working!'";

        // Route to appropriate provider for testing
        let response = if self.is_claude() {
            // For Claude, we need to send a text-only request
            let request = ClaudeRequest {
                model: self.get_model_name().to_string(),
                max_tokens: 50,
                messages: vec![ClaudeMessage {
                    role: "user".to_string(),
                    content: vec![ClaudeContentPart::Text { text: test_prompt.to_string() }],
                }],
            };

            let response = self
                .client
                .post(self.get_api_endpoint())
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await?;
                anyhow::bail!("{}", sanitize_error_message(&error_text));
            }

            let api_response: ClaudeResponse = response.json().await?;
            if let Some(ClaudeResponseContent::Text { text }) = api_response.content.first() {
                text.clone()
            } else {
                anyhow::bail!("No response from Claude. The API test didn't return any text.")
            }
        } else if self.is_gemini() {
            // For Gemini, send a text-only request
            let request = GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![GeminiPart::Text { text: test_prompt.to_string() }],
                }],
                generation_config: GeminiGenerationConfig {
                    temperature: 0.1,
                    max_output_tokens: 50,
                },
            };

            let model_name = self.get_model_name();
            let endpoint = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model_name, self.api_key
            );

            let response = self
                .client
                .post(&endpoint)
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().await?;
                anyhow::bail!("{}", sanitize_error_message(&error_text));
            }

            let api_response: GeminiResponse = response.json().await?;
            if let Some(candidate) = api_response.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    part.text.clone()
                } else {
                    anyhow::bail!("No response from Gemini. The API test didn't return any text.")
                }
            } else {
                anyhow::bail!("No response from Gemini. The API test didn't return any text.")
            }
        } else if self.is_openrouter() || self.is_groq() || self.is_openai() || self.is_local_llm() {
            // For OpenAI-compatible APIs (Groq, OpenAI, OpenRouter, Local LLMs)
            let request = GroqRequest {
                model: self.get_model_name().to_string(),
                messages: vec![GroqMessage {
                    role: "user".to_string(),
                    content: GroqMessageContent::Text(test_prompt.to_string()),
                }],
                temperature: 0.1,
                max_tokens: 50,
            };

            self.send_groq_request(request).await?
        } else {
            anyhow::bail!("Unsupported AI provider")
        };

        log_to_file(&format!("API test response: {}", response));
        Ok(response)
    }

    // ========================================================================
    // ITEM EXTRACTION METHODS (for detailed extraction mode)
    // ========================================================================

    /// Create a prompt specifically for extracting individual items from receipts
    fn create_item_extraction_prompt(&self, item_fields: &[String]) -> String {
        // Build field-specific instructions
        let mut field_instructions = String::new();
        for field in item_fields {
            let field_lower = field.to_lowercase();

            // FIRST: Check if we have a custom AI-generated prompt for this field
            if let Some(custom_prompt) = self.custom_prompts.get(field) {
                field_instructions.push_str(&format!("\n- '{}': {}", field, custom_prompt));
                continue;  // Skip all keyword-based checks
            }

            if field_lower.contains("name") || field_lower.contains("description") || field_lower.contains("item") && !field_lower.contains("price") && !field_lower.contains("total") && !field_lower.contains("qty") && !field_lower.contains("quantity") {
                field_instructions.push_str(&format!(
                    "\n- '{}': The TEXT description of the product (e.g., 'Milk 2L', 'USB Cable', 'Paper Towels'). \
                    This is NEVER a number! It's the product name/description that tells you WHAT was purchased.",
                    field
                ));
            } else if field_lower.contains("quantity") || field_lower.contains("qty") {
                field_instructions.push_str(&format!(
                    "\n- '{}': How many units were purchased (e.g., '1', '2', '3'). Usually a small whole number. Default to '1' if not shown.",
                    field
                ));
            } else if field_lower.contains("unit") && field_lower.contains("price") {
                field_instructions.push_str(&format!(
                    "\n- '{}': The price PER SINGLE UNIT before multiplying by quantity (e.g., '4.99', '12.50'). Numbers only, no $ symbol.",
                    field
                ));
            } else if field_lower.contains("total") || field_lower.contains("amount") || field_lower.contains("price") {
                field_instructions.push_str(&format!(
                    "\n- '{}': The total price for this line item (quantity × unit price). Numbers only, no $ symbol (e.g., '9.98', '25.00').",
                    field
                ));
            } else {
                field_instructions.push_str(&format!(
                    "\n- '{}': Extract this field if visible on the receipt.",
                    field
                ));
            }
        }

        // Create example with realistic values
        let example_item = item_fields.iter()
            .map(|f| {
                let field_lower = f.to_lowercase();
                let example_value = if field_lower.contains("name") || field_lower.contains("description") || (field_lower.contains("item") && !field_lower.contains("price") && !field_lower.contains("total")) {
                    "Paper Towels 6-Pack"
                } else if field_lower.contains("quantity") || field_lower.contains("qty") {
                    "2"
                } else if field_lower.contains("unit") && field_lower.contains("price") {
                    "5.99"
                } else if field_lower.contains("total") || field_lower.contains("amount") || field_lower.contains("price") {
                    "11.98"
                } else {
                    ""
                };
                format!("\"{}\": \"{}\"", f, example_value)
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "Extract ALL individual line items from this receipt. Return a JSON object with an \"items\" array.\n\n\
            For each item, extract these fields: {}\n\n\
            FIELD DEFINITIONS:{}\n\n\
            CRITICAL RULES:\n\
            1. 'Item Name' is ALWAYS TEXT (product description), NEVER a number!\n\
            2. Prices/amounts are ALWAYS numbers with decimals (e.g., '4.99')\n\
            3. Read the receipt carefully - items are usually listed with description on the left, prices on the right\n\
            4. Skip subtotals, taxes, shipping, and total lines - only include actual purchased products\n\
            5. For amounts: return only numbers without $ or currency symbols\n\
            6. If quantity is not shown, assume '1'\n\
            7. If a field is not visible for an item, use empty string \"\"\n\n\
            COMMON MISTAKE TO AVOID:\n\
            ❌ WRONG: {{ \"Item Name\": \"3.99\", \"Unit Price\": \"Milk\" }} - You swapped name and price!\n\
            ✅ CORRECT: {{ \"Item Name\": \"Milk 2L\", \"Unit Price\": \"3.99\" }}\n\n\
            EXAMPLE OUTPUT:\n\
            {{\n  \"items\": [\n    {{ {} }},\n    {{ \"Item Name\": \"Bread Whole Wheat\", \"Quantity\": \"1\", \"Unit Price\": \"3.49\", \"Line Total\": \"3.49\" }}\n  ]\n}}\n\n\
            Now extract ALL items and return ONLY the JSON object:",
            item_fields.join(", "),
            field_instructions,
            example_item
        )
    }

    /// Parse response containing an array of items
    fn parse_items_response(
        &self,
        response: &str,
        item_fields: &[String],
    ) -> Result<Vec<ExtractedItem>> {
        log_to_file("========== ITEM EXTRACTION RESPONSE ==========");
        log_to_file(response);
        log_to_file("===============================================");

        // Try to extract JSON from the response
        let json_str = if let Some(json_block) = self.extract_json_from_markdown(response) {
            json_block
        } else {
            self.extract_last_json_object(response).unwrap_or_else(|| response.to_string())
        };

        // Try to parse as { "items": [...] } format first
        #[derive(Deserialize)]
        struct ItemsWrapper {
            items: Vec<HashMap<String, serde_json::Value>>,
        }

        if let Ok(wrapper) = serde_json::from_str::<ItemsWrapper>(&json_str) {
            let items: Vec<ExtractedItem> = wrapper.items.into_iter().map(|item_map| {
                let mut fields = HashMap::new();
                for field in item_fields {
                    let value = item_map
                        .get(field)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    fields.insert(field.clone(), value);
                }
                ExtractedItem { fields }
            }).collect();

            log_to_file(&format!("Successfully parsed {} items", items.len()));
            return Ok(items);
        }

        // Try to parse as a direct array
        if let Ok(items_array) = serde_json::from_str::<Vec<HashMap<String, serde_json::Value>>>(&json_str) {
            let items: Vec<ExtractedItem> = items_array.into_iter().map(|item_map| {
                let mut fields = HashMap::new();
                for field in item_fields {
                    let value = item_map
                        .get(field)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    fields.insert(field.clone(), value);
                }
                ExtractedItem { fields }
            }).collect();

            log_to_file(&format!("Successfully parsed {} items from array", items.len()));
            return Ok(items);
        }

        log_to_file("Failed to parse items response");
        Ok(Vec::new())
    }

    /// Extract individual items from text
    pub async fn extract_items_from_text(
        &self,
        text: &str,
        item_fields: &[String],
    ) -> Result<Vec<ExtractedItem>> {
        let prompt = self.create_item_extraction_prompt(item_fields);
        let full_prompt = format!("{}\n\nDocument text:\n{}", prompt, text);

        let response = if self.is_claude() {
            log_to_file("Sending item extraction request to Claude API...");

            let request = ClaudeRequest {
                model: self.get_model_name().to_string(),
                max_tokens: 4096,
                messages: vec![ClaudeMessage {
                    role: "user".to_string(),
                    content: vec![ClaudeContentPart::Text { text: full_prompt }],
                }],
            };

            let response = self.client
                .post(self.get_api_endpoint())
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let api_response: ClaudeResponse = response.json().await?;
            if let Some(ClaudeResponseContent::Text { text }) = api_response.content.first() {
                text.clone()
            } else {
                String::new()
            }

        } else if self.is_gemini() {
            log_to_file("Sending item extraction request to Gemini API...");

            let request = GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![GeminiPart::Text { text: full_prompt }],
                }],
                generation_config: GeminiGenerationConfig {
                    temperature: 0.1,
                    max_output_tokens: 4096,
                },
            };

            let model_name = self.get_model_name();
            let endpoint = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model_name, self.api_key
            );

            let response = self.client
                .post(&endpoint)
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let api_response: GeminiResponse = response.json().await?;
            if let Some(candidate) = api_response.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    part.text.clone()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }

        } else {
            // Groq, OpenAI, OpenRouter
            log_to_file("Sending item extraction request to Groq/OpenAI API...");

            let request = GroqRequest {
                model: self.get_model_name().to_string(),
                messages: vec![GroqMessage {
                    role: "user".to_string(),
                    content: GroqMessageContent::Text(full_prompt),
                }],
                temperature: 0.1,
                max_tokens: 4096,
            };

            self.send_groq_request(request).await?
        };

        self.parse_items_response(&response, item_fields)
    }

    /// Extract individual items from images
    pub async fn extract_items_from_images(
        &self,
        base64_images: &[String],
        item_fields: &[String],
    ) -> Result<Vec<ExtractedItem>> {
        let prompt = self.create_item_extraction_prompt(item_fields);
        log_to_file(&format!("Item extraction vision prompt: {}", prompt));

        let response = if self.is_claude() {
            log_to_file("Sending item extraction to Claude Vision API...");

            let mut content_parts: Vec<ClaudeContentPart> = Vec::new();
            content_parts.push(ClaudeContentPart::Text { text: prompt });

            for base64_image in base64_images {
                content_parts.push(ClaudeContentPart::Image {
                    source: ClaudeImageSource {
                        media_type: "image/png".to_string(),
                        data: base64_image.clone(),
                        source_type: "base64".to_string(),
                    },
                });
            }

            let request = ClaudeRequest {
                model: self.get_model_name().to_string(),
                max_tokens: 4096,
                messages: vec![ClaudeMessage {
                    role: "user".to_string(),
                    content: content_parts,
                }],
            };

            let response = self.client
                .post(self.get_api_endpoint())
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let api_response: ClaudeResponse = response.json().await?;
            if let Some(ClaudeResponseContent::Text { text }) = api_response.content.first() {
                text.clone()
            } else {
                String::new()
            }

        } else if self.is_gemini() {
            log_to_file("Sending item extraction to Gemini Vision API...");

            let mut parts: Vec<GeminiPart> = Vec::new();
            parts.push(GeminiPart::Text { text: prompt });

            for base64_image in base64_images {
                parts.push(GeminiPart::InlineData {
                    inline_data: GeminiInlineData {
                        mime_type: "image/png".to_string(),
                        data: base64_image.clone(),
                    },
                });
            }

            let request = GeminiRequest {
                contents: vec![GeminiContent { parts }],
                generation_config: GeminiGenerationConfig {
                    temperature: 0.1,
                    max_output_tokens: 4096,
                },
            };

            let model_name = self.get_model_name();
            let endpoint = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model_name, self.api_key
            );

            let response = self.client
                .post(&endpoint)
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let api_response: GeminiResponse = response.json().await?;
            if let Some(candidate) = api_response.candidates.first() {
                if let Some(part) = candidate.content.parts.first() {
                    part.text.clone()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }

        } else {
            // Groq, OpenAI, OpenRouter
            log_to_file("Sending item extraction to Groq/OpenAI Vision API...");

            let mut content_parts: Vec<GroqContentPart> = Vec::new();
            content_parts.push(GroqContentPart {
                content_type: "text".to_string(),
                text: Some(prompt),
                image_url: None,
            });

            for base64_image in base64_images {
                let image_url = format!("data:image/png;base64,{}", base64_image);
                content_parts.push(GroqContentPart {
                    content_type: "image_url".to_string(),
                    text: None,
                    image_url: Some(GroqImageUrl { url: image_url }),
                });
            }

            let request = GroqRequest {
                model: self.get_model_name().to_string(),
                messages: vec![GroqMessage {
                    role: "user".to_string(),
                    content: GroqMessageContent::MultiPart(content_parts),
                }],
                temperature: 0.1,
                max_tokens: 4096,
            };

            self.send_groq_request(request).await?
        };

        self.parse_items_response(&response, item_fields)
    }

    /// Generate an optimized extraction prompt for a custom field
    /// This uses AI to create a clear, concise instruction for extracting the field
    pub async fn enhance_field_prompt(&self, field_name: &str) -> Result<String> {
        let enhancement_prompt = format!(
            r#"You are helping create an extraction instruction for a receipt data extraction system.

The user wants to extract a field called: "{}"

Generate a clear, concise instruction (1-2 sentences) that tells an AI exactly:
1. What to look for in the receipt
2. How to format the answer
3. What to return if not found (use empty string)

Return ONLY the instruction text, nothing else. No quotes, no explanation.

Examples:
Field: "Is this a refund?"
Instruction: Determine if this is a purchase or refund by looking for keywords like REFUND, RETURN, CREDIT, or negative amounts. Answer with exactly "Purchase" or "Refund".

Field: "Category"
Instruction: Categorize this purchase based on the items and store (e.g., Groceries, Electronics, Clothing, Restaurant, Office Supplies). Return a single category name.

Field: "Discount amount"
Instruction: Find any discount, coupon, or savings amount shown on the receipt. Return the amount with currency symbol or empty string if none.

Now generate the instruction for: "{}""#,
            field_name, field_name
        );

        // Use text-based extraction (no image needed)
        // Route to appropriate provider
        let response = if self.is_claude() {
            // Claude uses its own request format
            let request = serde_json::json!({
                "model": if self.use_latest_models { "claude-sonnet-4-20250514" } else { "claude-3-5-haiku-20241022" },
                "max_tokens": 500,
                "messages": [{
                    "role": "user",
                    "content": enhancement_prompt
                }]
            });

            let response = self.client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await?;

            let response_json: serde_json::Value = response.json().await?;
            response_json["content"][0]["text"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("No response from Claude"))?
        } else if self.is_gemini() {
            let request = GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![GeminiPart::Text { text: enhancement_prompt.clone() }],
                }],
                generation_config: GeminiGenerationConfig {
                    temperature: 0.3,
                    max_output_tokens: 500
                },
            };

            let model = if self.use_latest_models { "gemini-2.0-flash" } else { "gemini-2.5-flash" };
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, self.api_key
            );

            let response = self.client
                .post(&url)
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await?;

            let response_json: serde_json::Value = response.json().await?;
            response_json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("No response from Gemini"))?
        } else if self.is_openrouter() || self.is_groq() || self.is_openai() || self.is_local_llm() {
            // OpenRouter, Groq, OpenAI, and Local LLMs all use OpenAI-compatible API
            let request = GroqRequest {
                model: self.get_model_name().to_string(),
                messages: vec![GroqMessage {
                    role: "user".to_string(),
                    content: GroqMessageContent::Text(enhancement_prompt),
                }],
                temperature: 0.3,
                max_tokens: 500,
            };
            self.send_groq_request(request).await?
        } else {
            return Err(anyhow::anyhow!("Unknown API provider"));
        };

        // Clean up the response - remove any quotes or extra whitespace
        let cleaned = response.trim().trim_matches('"').trim().to_string();
        Ok(cleaned)
    }
}

