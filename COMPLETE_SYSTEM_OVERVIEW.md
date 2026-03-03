# Receipt Extractor - Complete System Overview 🎉

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    COMPLETE SYSTEM                          │
└─────────────────────────────────────────────────────────────┘

┌──────────────────┐         ┌──────────────────┐
│   Admin Tool     │────────▶│   MySQL DB       │
│   (Rust GUI)     │         │  (License Keys)  │
└──────────────────┘         └────────┬─────────┘
                                      │
                                      │
┌──────────────────┐         ┌───────▼──────────┐
│   User App       │────────▶│  Backend API     │
│   (Rust GUI)     │         │  (Node.js)       │
└──────────────────┘         └──────────────────┘
         │
         │
         ▼
┌──────────────────┐
│   AI Services    │
│  Groq / OpenAI   │
│  / Ollama        │
└──────────────────┘
```

---

## Components

### 1. User Application (Rust Desktop App)

**Location:** `./` (root directory)

**Purpose:** Main application for extracting receipt data using AI

**Features:**
- License activation with backend validation
- Device fingerprinting (max 2 devices per license)
- Pro tier with cloud AI access
- Multi-page PDF processing (up to 4 pages)
- AI provider fallback (Groq → OpenAI)
- Automatic fallback on rate limits
- CSV export with custom location
- Configurable extraction fields

**Tech Stack:**
- Rust + egui/eframe
- reqwest (HTTP client)
- AES-256-GCM encryption
- SHA-256 hashing
- whoami (device fingerprinting)

**Key Files:**
- `src/main.rs` - Entry point
- `src/app.rs` - Main UI and logic
- `src/license.rs` - License validation with backend
- `src/ai_extractor.rs` - AI API integration
- `src/document_processor.rs` - PDF/image processing

---

### 2. Backend API (Node.js + Express)

**Location:** `./backend/`

**Purpose:** License validation, activation tracking, API key delivery

**Features:**
- License validation endpoint
- Device activation tracking
- API key delivery for Pro users
- Admin endpoints for license management
- Shopify webhook integration
- Rate limiting (100 req/15min)
- Security headers (helmet)

**Tech Stack:**
- Node.js + Express
- MySQL (mysql2)
- dotenv (environment variables)
- cors, helmet, express-rate-limit

**Key Files:**
- `server.js` - Main server
- `routes/license.js` - License validation
- `routes/admin.js` - Admin endpoints
- `routes/shopify.js` - Shopify integration
- `database/schema.sql` - Database schema
- `scripts/generate-keys.js` - Key generation script

**Endpoints:**
- `GET /health` - Health check
- `POST /api/license/validate` - Validate license
- `POST /api/shopify/order-created` - Shopify webhook
- `GET /api/admin/stats` - Admin statistics
- `GET /api/admin/licenses` - List all licenses
- `POST /api/admin/license/revoke` - Revoke license

---

### 3. Admin Tool (Rust GUI)

**Location:** `./admin_tool/`

**Purpose:** Manage licenses, generate keys, assign to customers

**Features:**
- Dashboard with statistics
- Generate Pro and Budget license keys
- Assign licenses to customers
- Revoke licenses and remove activations
- Filter licenses by tier
- Direct MySQL integration
- Database connection settings

**Tech Stack:**
- Rust + egui/eframe
- mysql crate (direct DB access)
- rand (key generation)

**Key Files:**
- `src/main.rs` - Entry point
- `src/app.rs` - Main UI
- `src/database.rs` - MySQL operations
- `src/keygen.rs` - License key generation

**Tabs:**
1. Dashboard - Statistics and recent licenses
2. Generate Keys - Create new license keys
3. Manage Licenses - Assign and revoke licenses
4. Settings - Database configuration

---

### 4. MySQL Database

**Database:** `receipt_extractor`

**Tables:**
1. `license_keys` - All license keys
2. `license_activations` - Device activations
3. `api_logs` - API usage logs
4. `shopify_orders` - Shopify order tracking
5. `admin_users` - Admin authentication

**Views:**
1. `license_summary` - License overview
2. `activation_statistics` - Activation stats

---

## Business Model

### Pro Tier (Active)
- Groq + OpenAI (cloud AI processing)
- Fast (2-5 seconds per receipt)
- Automatic fallback on rate limits
- API keys provided by backend
- Configurable extraction fields
- Multi-page PDF support

### Budget Tier (Legacy)
- Legacy code, not actively used
- Removed from current version

---

## License Key Format

**Pro:** `PRO-XXXX-XXXX-XXXX` (18 characters)  
**Budget:** `BUDGET-XXXX-XXXX-XXXX` (21 characters)

**Validation:**
- Cryptographically secure random generation
- Unique in database
- Tier-specific prefix
- Alphanumeric segments

---

## Activation Flow

```
1. User enters license key in app
   ↓
2. App sends validation request to backend
   - License key
   - Device fingerprint (SHA-256 hash)
   - Device name (hostname)
   ↓
3. Backend validates:
   - Key exists in database
   - Not revoked
   - Under activation limit (2 devices)
   ↓
4. Backend saves activation and returns:
   - Tier (pro or budget)
   - API keys (if Pro tier)
   ↓
5. App saves encrypted license locally
   - AES-256-GCM encryption
   - Machine-specific key derivation
   ↓
6. App unlocks features based on tier
```

---

## Security Features

✅ **Encrypted License Storage** - AES-256-GCM  
✅ **Device Fingerprinting** - SHA-256 hash  
✅ **Server-Side Validation** - All licenses checked  
✅ **Activation Limits** - Max 2 devices  
✅ **API Key Protection** - Never in app code  
✅ **Rate Limiting** - 100 requests per 15 minutes  
✅ **CORS Protection** - Configured origins  
✅ **Security Headers** - Helmet middleware  
✅ **HMAC Verification** - Shopify webhooks  

---

## Testing

### Test License Keys

**Pro Tier:**
- `PRO-JIMK-NJ7G-AZRO`
- `PRO-DFGI-07SX-XT2B`
- `PRO-4QSN-M129-TEQZ`

**Budget Tier:**
- `BUDGET-4RGW-8MCE-LM46`
- `BUDGET-M1NL-NPP0-EQXU`
- `BUDGET-E9Y9-K2JG-66A1`

### Test Script

```powershell
cd backend
.\test-api.ps1
```

---

## Deployment Checklist

### Backend
- [ ] Update API URL in production
- [ ] Set environment variables
- [ ] Configure MySQL connection
- [ ] Set up SSL/TLS
- [ ] Configure firewall
- [ ] Set up monitoring
- [ ] Configure backups

### User App
- [ ] Update API URL to production
- [ ] Build release binary
- [ ] Code sign executable
- [ ] Create installer
- [ ] Test on clean machine
- [ ] Prepare documentation

### Admin Tool
- [ ] Configure production database
- [ ] Build release binary
- [ ] Restrict access
- [ ] Document procedures

---

## Next Steps

1. ✅ **Rust App Updated** - Backend integration complete
2. ✅ **Admin Tool Created** - GUI for license management
3. ⏳ **Deploy Backend** - Move to production server
4. ⏳ **Shopify Integration** - Auto-send keys on purchase
5. ⏳ **Build Installers** - Create distributable packages
6. ⏳ **Marketing Website** - Landing page with purchase
7. ⏳ **Documentation** - User guides and tutorials

---

## Support

For issues or questions:
1. Check troubleshooting sections in READMEs
2. Review backend logs
3. Check MySQL database state
4. Verify API connectivity

