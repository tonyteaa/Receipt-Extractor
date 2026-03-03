# Changelog

## Version 3.0 - License System & Backend Integration (Current)

### 🚀 Major Changes

**Added License System with Backend API**

- **License Tiers**: Pro tier with cloud AI access
- **Backend API**: Node.js/Express server for license validation
- **Device Management**: Max 2 activations per license
- **AI Strategy**: Groq (primary) with OpenAI fallback
- **Benefits**:
  - ⚡ **Fast**: 2-5 seconds per receipt with Groq
  - 🔐 **Secure**: License validation via backend
  - 🔄 **Reliable**: Automatic fallback to OpenAI
  - 📊 **Flexible**: Any extraction field can be added

### 🎯 Improvements

**1. Date Format Standardization**
- **Problem**: Dates were returned in various formats with timestamps (e.g., "Jan 15, 2024 10:30 AM PST")
- **Solution**: Now enforces DD/MM/YYYY format (e.g., "15/01/2024")
- **Changes**:
  - Updated prompt instructions for date fields
  - Added emphasis section for date formatting
  - Updated example format in prompt

**2. Better Amount Detection**
- **Problem**: Amount fields were sometimes empty
- **Solution**: Enhanced prompts to look in multiple locations
- **Changes**:
  - Added specific instructions to check totals section, order summary, payment details
  - Added emphasis to look for labels like "Total", "Grand Total", "Order Total", etc.
  - Improved consistency in currency symbol inclusion

**3. Better Item Detection**
- **Problem**: Item names were sometimes not detected
- **Solution**: Stronger prompts with multiple search locations
- **Changes**:
  - Added instructions to check order details, item lists, product descriptions
  - Enhanced emphasis section for item extraction
  - Better handling of full product names vs summaries

### 📝 Code Changes

**File: `src/ai_extractor.rs`**

1. **Removed legacy Ollama code**
   - Cleaned up all Ollama structs, methods, and status checks
   - Simplified codebase to focus on cloud AI providers

2. **Groq + OpenAI Integration**
   - `GroqRequest`, `GroqMessage`, `GroqMessageContent`, etc.
   - Automatic fallback from Groq to OpenAI on rate limits
   - API keys provided by backend server

3. **Added Groq methods** (lines 286-352)
   - `extract_from_text()` - For text-based documents
   - `extract_from_image()` - For image-based PDFs and scanned receipts
   - Uses `llama-3.2-90b-vision-preview` model

4. **Added Groq send_request** (lines 565-594)
   - Endpoint: `https://api.groq.com/openai/v1/chat/completions`
   - Uses API key from user input
   - Error handling for rate limits and API errors

5. **Enhanced prompts** (lines 354-501)
   - Date: Added DD/MM/YYYY format requirement
   - Amount: Added instructions to check multiple locations
   - Item: Added instructions to check order details sections
   - Updated example format to show DD/MM/YYYY dates

### 📚 Documentation

**New Files:**
- `backend/README.md` - Backend API documentation
- `BACKEND_INTEGRATION.md` - Integration guide
- `LICENSE_KEYS_FOR_TESTING.md` - Testing guide

**Updated Files:**
- `README.md` - Updated to reflect license system
- `CHANGELOG.md` - This file

### 🔄 Migration Path

**From OpenAI → Groq → License System**

1. **Version 1.0**: Used OpenAI (hit rate limits)
2. **Version 2.0**: Switched to Groq (fast, free tier)
3. **Version 3.0**: Added license system with backend (current)

Legacy Ollama code has been removed to simplify the codebase.

### 🐛 Known Issues Fixed

1. ✅ **Date format inconsistency** - Now always DD/MM/YYYY
2. ✅ **Missing amounts** - Better detection with enhanced prompts
3. ✅ **Missing item names** - Stronger prompts to find products
4. ✅ **Slow processing** - 10-20x faster with Groq
5. ✅ **RAM issues** - No longer runs locally

### 🎯 Default Fields

The app comes with these 8 default fields:

1. **Date** - Purchase date in DD/MM/YYYY format
2. **Vendor/Store Name** - Main marketplace (e.g., "Amazon.com")
3. **Seller** - Actual seller (e.g., "Office Supplies Inc")
4. **Total Amount** - Total with currency symbol (e.g., "$45.99")
5. **Tax Amount** - Tax with currency symbol
6. **Payment Method** - Card type and last 4 digits (e.g., "Visa ••••1234")
7. **item** - Full product description
8. **item summary** - Short name (under 5 words)

### 🚀 Next Steps for Users

1. **Close the app** if it's running
2. **Rebuild**: `cargo build --release`
3. **Get Groq API key**: https://console.groq.com/keys
4. **Run the app**: `.\target\release\receipt_extractor.exe`
5. **Enter API key** and start processing!

### 📊 Performance Comparison

| Metric | OpenAI | Ollama (MiniCPM) | Groq |
|--------|--------|------------------|------|
| Speed | 3-5s | 20-60s | 2-5s |
| Cost | Paid (rate limits) | Free | Free |
| RAM | 0 (cloud) | 8GB+ | 0 (cloud) |
| Quality | Excellent | Good | Excellent |
| Setup | API key | Install + model | API key |
| Limits | 200K TPM | Unlimited | 14.4K req/day |

**Winner**: Groq! 🏆

