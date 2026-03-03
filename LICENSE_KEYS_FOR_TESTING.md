# License Key System

## 🔑 Generate New License Keys

Use the built-in key generator:

```powershell
# Generate 1 Budget key
cargo run --bin keygen budget

# Generate 5 Pro keys
cargo run --bin keygen pro 5

# Generate 10 Budget keys
cargo run --bin keygen budget 10
```

**Output Example:**
```
🔑 Generating 5 PRO License Keys:

─────────────────────────────────────
1. PRO-LVBZ-HHB3-1DCS
2. PRO-1B6M-KJLI-0QMI
3. PRO-GX6F-YXM9-9CPO
4. PRO-IP91-3OCA-8YKA
5. PRO-7P2K-CZD1-CQ3A
─────────────────────────────────────
```

---

## 📋 Test License Keys

### Budget Tier License Keys
Format: `BUDGET-XXXX-XXXX-XXXX` (minimum 20 characters)

**Pre-generated Test Keys:**
- `BUDGET-T8I5-MEAM-MRGR`
- `BUDGET-Y9H5-TV2F-GC26`
- `BUDGET-YUKK-MAXP-KAGB`

### Pro Tier License Keys
Format: `PRO-XXXX-XXXX-XXXX` (minimum 20 characters)

**Pre-generated Test Keys:**
- `PRO-LVBZ-HHB3-1DCS`
- `PRO-1B6M-KJLI-0QMI`
- `PRO-GX6F-YXM9-9CPO`

## How It Works

1. **Budget Tier** (`BUDGET-...`):
   - Uses Ollama only (local processing)
   - Slower (20-60 seconds per receipt)
   - Unlimited processing
   - No cloud API costs

2. **Pro Tier** (`PRO-...`):
   - Uses Groq + OpenAI (cloud processing)
   - Fast (2-5 seconds per receipt)
   - Automatic fallback to Ollama if rate limits hit
   - Cloud APIs provided by you

## Current Implementation

**Note:** This is a simple offline validation for testing. For production, you'll need:

1. **Server-side validation**: Verify license keys against your database
2. **Shopify integration**: Generate unique keys on purchase
3. **API key delivery**: Securely deliver cloud API keys to Pro users
4. **License activation limits**: Prevent key sharing (e.g., max 2 devices)

## Next Steps for Production

### Phase 1: Backend Setup
- Create simple API endpoint: `POST /api/validate-license`
- Store valid license keys in database
- Return tier + API keys for Pro users

### Phase 2: Shopify Integration
- Create Shopify webhook for new orders
- Generate unique license keys
- Email keys to customers
- Store in database

### Phase 3: Enhanced Security
- Add device fingerprinting
- Limit activations per key
- Add license expiration (optional)
- Implement license revocation

## Example Backend Response

```json
{
  "valid": true,
  "tier": "pro",
  "api_keys": {
    "groq": "gsk_...",
    "openai": "sk-..."
  }
}
```

For Budget tier:
```json
{
  "valid": true,
  "tier": "budget"
}
```

