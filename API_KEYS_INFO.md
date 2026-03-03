# Hard-Coded API Keys

This app now has **both API keys hard-coded** for easy switching between providers!

## 🔑 Hard-Coded Keys

### Groq (Default)
- **API Key**: `your_groq_api_key_here`
- **Model**: Llama 4 Scout 17B (Vision)
- **Speed**: 2-5 seconds per receipt
- **Cost**: 100% Free (14,400 requests/day)
- **Best for**: Fast, free processing

### ChatGPT (OpenAI)
- **API Key**: `your_openai_api_key_here`
- **Model**: GPT-4o-mini
- **Speed**: 3-5 seconds per receipt
- **Cost**: Pay-as-you-go (very cheap)
- **Best for**: Highest quality, when you need the best

## 🎯 How to Use

1. **Run the app**:
   ```powershell
   .\target\release\receipt_extractor.exe
   ```

2. **Select AI Provider** from the dropdown:
   - **Groq (Free & Fast)** - Default, recommended for most users
   - **ChatGPT (OpenAI)** - For highest quality

3. **That's it!** No need to enter API keys manually - they're already hard-coded!

## 🔄 Switching Between Providers

Just use the dropdown menu in the app! The app automatically:
- ✅ Detects which provider based on API key prefix (`gsk_` = Groq, `sk-` = OpenAI)
- ✅ Uses the correct API endpoint
- ✅ Uses the correct model name
- ✅ Logs which provider is being used

## 📊 Comparison

| Feature | Groq | ChatGPT |
|---------|------|---------|
| Speed | ⚡ 2-5s | ⚡ 3-5s |
| Cost | 💰 Free | 💵 ~$0.001/receipt |
| Quality | 🎯 Excellent | 🎯 Best |
| Limits | 14.4K/day | Very high |
| Model | Llama 4 Scout 17B | GPT-4o-mini |

## 🔧 Technical Details

### Auto-Detection Logic

The app detects which provider to use based on the API key prefix:

```rust
fn is_groq(&self) -> bool {
    self.api_key.starts_with("gsk_")
}

fn is_openai(&self) -> bool {
    self.api_key.starts_with("sk-")
}
```

### API Endpoints

- **Groq**: `https://api.groq.com/openai/v1/chat/completions`
- **OpenAI**: `https://api.openai.com/v1/chat/completions`

### Models Used

- **Groq**: `meta-llama/llama-4-scout-17b-16e-instruct`
- **OpenAI**: `gpt-4o-mini`

## 🎉 Benefits

1. ✅ **No manual API key entry** - Just select from dropdown
2. ✅ **Easy switching** - Change providers with one click
3. ✅ **Both options available** - Use free Groq or premium OpenAI
4. ✅ **Automatic detection** - App knows which provider you selected
5. ✅ **Same features** - All extraction features work with both providers

## 🚀 Recommended Usage

- **For daily use**: Use **Groq** (free, fast, excellent quality)
- **For critical receipts**: Use **ChatGPT** (highest quality, small cost)
- **For batch processing**: Use **Groq** (no rate limits on free tier for reasonable use)

Enjoy! 🎯

