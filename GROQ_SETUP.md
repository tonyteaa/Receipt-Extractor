# Groq Setup Guide - Free & Fast Cloud AI!

The Receipt Extractor now uses **Groq** - a free, super-fast cloud AI service!

## ✅ Why Groq?

- **100% Free** - No credit card required
- **Super Fast** - 10-100x faster than local models
- **High Quality** - Uses Llama 3.2 Vision 90B (very powerful)
- **Generous Limits** - 14,400 requests/day on free tier
- **No Installation** - Just need an API key

## 🚀 Setup (Takes 2 Minutes!)

### Step 1: Create a Groq Account

1. Go to: https://console.groq.com/
2. Click **"Sign Up"** (you can use Google/GitHub login)
3. No credit card required!

### Step 2: Get Your API Key

1. Once logged in, go to: https://console.groq.com/keys
2. Click **"Create API Key"**
3. Give it a name (e.g., "Receipt Extractor")
4. Click **"Submit"**
5. **Copy the API key** (starts with `gsk_...`)

### Step 3: Use the API Key in the App

1. Run the app:
   ```powershell
   .\target\release\receipt_extractor.exe
   ```

2. **Paste your Groq API key** in the API Key field
3. Click **"Save"**
4. Start processing receipts!

## 🎯 What You Get

- **Model**: Llama 3.2 Vision 90B (one of the best vision models available)
- **Speed**: ~2-5 seconds per receipt (super fast!)
- **Free Tier Limits**:
  - 14,400 requests per day
  - 6,000 requests per minute
  - More than enough for personal use!

## 📊 Cloud AI Benefits

Using cloud AI providers (Groq + OpenAI):

| Feature | Cloud AI |
|---------|----------|
| Speed | 2-5 seconds per receipt |
| RAM Usage | 0 (runs in cloud) |
| Quality | Excellent (90B+ models) |
| Reliability | Automatic fallback |
| Setup | License-based |

## 🔧 Improvements in This Version

### 1. **Date Format Fixed**
- Now returns dates in **DD/MM/YYYY** format (e.g., `15/01/2024`)
- No more timestamps or timezone info

### 2. **Better Amount Detection**
- Improved instructions to find amounts in totals section
- Always includes currency symbol (e.g., `$45.99`)

### 3. **Better Item Detection**
- Stronger prompts to find product names
- Checks order details, item lists, and product descriptions

## 🆓 Free Tier Details

Groq's free tier is very generous:

- **Daily Limit**: 14,400 requests
- **Per Minute**: 6,000 requests
- **Tokens per Minute**: 30,000

For receipt processing, you can easily process **hundreds of receipts per day** for free!

## 🔄 AI Provider Strategy

The app uses a smart fallback strategy:

1. **Primary**: Groq API (fast, generous free tier)
2. **Fallback**: OpenAI API (if Groq rate limited)
3. **Automatic**: Switches between providers seamlessly

API keys are provided by the backend server after license validation.

## 🆘 Troubleshooting

### "Invalid API key" error
- Make sure you copied the full API key (starts with `gsk_`)
- Check for extra spaces before/after the key
- Generate a new key at https://console.groq.com/keys

### "Rate limit exceeded"
- You've hit the free tier limit (14,400/day)
- Wait until the next day, or upgrade to paid tier
- Check your usage at https://console.groq.com/

### Slow responses
- Groq is usually very fast (2-5 seconds)
- If slow, check your internet connection
- Check Groq status: https://status.groq.com/

## 🎉 You're All Set!

1. Get your API key from https://console.groq.com/keys
2. Paste it in the app
3. Process receipts super fast!

Enjoy unlimited, fast, high-quality receipt extraction! 🚀

