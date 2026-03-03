# Quick Start Guide 🚀

## Complete System Setup

### Prerequisites

✅ **Installed:**
- Rust (latest stable)
- Node.js (v18+)
- MySQL (8.0+)
- PowerShell (Windows)

---

## Step 1: Start MySQL Database

Make sure MySQL is running with the database initialized:

```sql
Database: receipt_extractor
User: Antoine
Password: 3e3.]Koh=+meXwXo
```

---

## Step 2: Start Backend API

```powershell
cd backend
node server.js
```

**Expected output:**
```
MySQL database connected successfully

╔════════════════════════════════════════════╗
║  Receipt Extractor License API Server     ║
╚════════════════════════════════════════════╝

🚀 Server running on port 3000
📍 Environment: development
🔗 Health check: http://127.0.0.1:3000/health
```

**Test the API:**
```powershell
.\test-api.ps1
```

---

## Step 3: Run User Application

In a new terminal:

```powershell
cd ..
cargo run --release
```

**First Launch:**
1. App will show license activation screen
2. Enter a test license key:
   - Pro: `PRO-JIMK-NJ7G-AZRO`
   - Budget: `BUDGET-4RGW-8MCE-LM46`
3. Click "Activate License"
4. App validates with backend and saves encrypted license

**Subsequent Launches:**
- App loads cached license
- Validates with backend on startup
- Unlocks features based on tier

---

## Step 4: Run Admin Tool (Optional)

In a new terminal:

```powershell
cd admin_tool
cargo run --release
```

**Features:**
- Dashboard: View license statistics
- Generate Keys: Create new Pro/Budget licenses
- Manage Licenses: Assign to customers, revoke licenses
- Settings: Configure database connection

---

## Testing the Complete Flow

### 1. Generate License Keys (Admin Tool)

1. Open Admin Tool
2. Go to "Generate Keys" tab
3. Select tier (Pro or Budget)
4. Enter count (e.g., 5)
5. Click "Generate Keys"
6. Copy a generated key

### 2. Assign License to Customer (Admin Tool)

1. Go to "Manage Licenses" tab
2. Paste the license key
3. Enter customer email and name
4. Click "Assign License"

### 3. Activate License (User App)

1. Open User App
2. Enter the license key
3. Click "Activate License"
4. App validates with backend
5. License saved locally (encrypted)

### 4. Process Receipts (User App)

**Pro Tier:**
1. Select files (PDF/images)
2. Choose AI provider (Groq or OpenAI)
3. Click "Start Processing"
4. Fast processing (2-5 seconds per receipt)
5. Save CSV to custom location

**Budget Tier:**
1. Select files (PDF/images)
2. Only Ollama available
3. Click "Start Processing"
4. Slower processing (20-60 seconds per receipt)
5. Save CSV to custom location

### 5. Revoke License (Admin Tool)

1. Go to "Manage Licenses" tab
2. Find the license in the list
3. Click "Revoke" button
4. All device activations removed
5. Customer can re-activate on new devices

---

## Test License Keys

### Pro Tier (Fast Cloud Processing)
```
PRO-JIMK-NJ7G-AZRO
PRO-DFGI-07SX-XT2B
PRO-4QSN-M129-TEQZ
PRO-RVQY-RVQY-RVQY
PRO-RVQY-RVQY-RVQZ
PRO-RVQY-RVQY-RVR0
PRO-RVQY-RVQY-RVR1
PRO-RVQY-RVQY-RVR2
PRO-RVQY-RVQY-RVR3
PRO-RVQY-RVQY-RVR4
```

### Budget Tier (Local Processing)
```
BUDGET-4RGW-8MCE-LM46
BUDGET-M1NL-NPP0-EQXU
BUDGET-E9Y9-K2JG-66A1
BUDGET-RVQY-RVQY-RVQY
BUDGET-RVQY-RVQY-RVQZ
BUDGET-RVQY-RVQY-RVR0
BUDGET-RVQY-RVQY-RVR1
BUDGET-RVQY-RVQY-RVR2
BUDGET-RVQY-RVQY-RVR3
BUDGET-RVQY-RVQY-RVR4
```

---

## Troubleshooting

### Backend won't start
- Check MySQL is running
- Verify credentials in `.env`
- Check port 3000 is not in use: `netstat -ano | findstr :3000`

### User app can't connect
- Make sure backend is running
- Check API URL in `src/license.rs` (should be `http://127.0.0.1:3000`)
- Verify firewall isn't blocking port 3000

### License activation fails
- Check backend logs for errors
- Verify license key exists in database
- Check device activation limit (max 2 devices)

### Admin tool can't connect to database
- Verify MySQL credentials in Settings tab
- Check MySQL is running
- Verify database `receipt_extractor` exists

---

## File Locations

### User App
- **License file:** `%APPDATA%/receipt_extractor/license.enc`
- **Encrypted with:** AES-256-GCM
- **Key derivation:** Machine-specific (hostname + username)

### Backend
- **Database:** MySQL `receipt_extractor`
- **Environment:** `backend/.env`
- **Logs:** Console output

### Admin Tool
- **No local storage** - All data in MySQL
- **Direct database access** - No API layer

---

## Next Steps

1. ✅ **System is working!** All components integrated
2. 🔄 **Test the complete flow** with the steps above
3. 📝 **Generate more licenses** as needed
4. 🚀 **Deploy backend** to production server
5. 🌐 **Update API URL** in Rust app for production
6. 📦 **Build release binaries** for distribution
7. 🛒 **Set up Shopify** for automated license delivery

---

## Support

For issues:
1. Check backend logs
2. Check MySQL database state
3. Verify API connectivity
4. Review troubleshooting sections

Enjoy your Receipt Extractor system! 🎉

