# 🔑 License Key Generator

## Quick Start

Generate license keys for your customers:

```powershell
# Generate Budget tier keys
cargo run --bin keygen budget 10

# Generate Pro tier keys
cargo run --bin keygen pro 10
```

---

## 📖 Usage Guide

### Command Syntax
```
cargo run --bin keygen <tier> [count]
```

**Arguments:**
- `<tier>` - Required: `budget` or `pro`
- `[count]` - Optional: Number of keys (1-100, default: 1)

### Examples

**Single Key:**
```powershell
cargo run --bin keygen pro
```

**Multiple Keys:**
```powershell
cargo run --bin keygen budget 25
cargo run --bin keygen pro 50
```

**For Shopify Integration:**
```powershell
# Generate 100 Pro keys for launch
cargo run --bin keygen pro 100 > pro_keys.txt
```

---

## 🏷️ Key Format

### Budget Tier
- **Format:** `BUDGET-XXXX-XXXX-XXXX`
- **Length:** 20+ characters
- **Example:** `BUDGET-T8I5-MEAM-MRGR`

### Pro Tier
- **Format:** `PRO-XXXX-XXXX-XXXX`
- **Length:** 20+ characters
- **Example:** `PRO-LVBZ-HHB3-1DCS`

**Character Set:** A-Z, 0-9 (uppercase only)

---

## 🔐 Security Features

1. **Cryptographically Secure**: Uses `rand` crate with secure RNG
2. **Unique Keys**: Each key is randomly generated
3. **Format Validation**: App validates format before accepting
4. **Tier Identification**: Prefix determines tier automatically

---

## 🛒 Shopify Integration Workflow

### Option 1: Pre-generate Keys

1. **Generate keys in bulk:**
   ```powershell
   cargo run --bin keygen pro 1000 > pro_keys.txt
   cargo run --bin keygen budget 1000 > budget_keys.txt
   ```

2. **Import to database:**
   - Store keys with `used: false` status
   - Mark as `used: true` when assigned to customer

3. **Shopify webhook:**
   - On new order → fetch unused key from database
   - Email key to customer
   - Mark key as used

### Option 2: Generate On-Demand

1. **Shopify webhook triggers your backend**
2. **Backend calls keygen or generates key programmatically**
3. **Store in database + email to customer**

---

## 💻 Programmatic Generation (Backend)

If you want to generate keys from your backend (Node.js, Python, etc.), here's the algorithm:

### JavaScript Example
```javascript
function generateLicenseKey(tier) {
  const prefix = tier.toUpperCase();
  const segments = [
    generateSegment(4),
    generateSegment(4),
    generateSegment(4)
  ];
  return `${prefix}-${segments.join('-')}`;
}

function generateSegment(length) {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

// Usage
const proKey = generateLicenseKey('PRO');
const budgetKey = generateLicenseKey('BUDGET');
```

### Python Example
```python
import random
import string

def generate_license_key(tier):
    prefix = tier.upper()
    segments = [generate_segment(4) for _ in range(3)]
    return f"{prefix}-{'-'.join(segments)}"

def generate_segment(length):
    chars = string.ascii_uppercase + string.digits
    return ''.join(random.choice(chars) for _ in range(length))

# Usage
pro_key = generate_license_key('PRO')
budget_key = generate_license_key('BUDGET')
```

---

## 📊 Database Schema

Recommended schema for storing keys:

```sql
CREATE TABLE license_keys (
    id SERIAL PRIMARY KEY,
    key VARCHAR(50) UNIQUE NOT NULL,
    tier VARCHAR(10) NOT NULL,  -- 'BUDGET' or 'PRO'
    created_at TIMESTAMP DEFAULT NOW(),
    used BOOLEAN DEFAULT FALSE,
    customer_email VARCHAR(255),
    order_id VARCHAR(100),
    activated_at TIMESTAMP,
    device_fingerprint VARCHAR(255)
);

CREATE INDEX idx_unused_keys ON license_keys(tier, used) WHERE used = FALSE;
```

---

## 🚀 Production Checklist

- [ ] Generate initial key pool (1000+ keys per tier)
- [ ] Set up database with schema above
- [ ] Create backend API endpoint for key validation
- [ ] Set up Shopify webhook for new orders
- [ ] Test key generation and validation flow
- [ ] Set up email delivery system
- [ ] Monitor key usage and generate more as needed

---

## 🔧 Troubleshooting

**Error: "Tier must be 'budget' or 'pro'"**
- Check spelling: must be lowercase `budget` or `pro`

**Error: "Count must be between 1 and 100"**
- Generate in batches if you need more than 100 keys

**Keys not working in app:**
- Ensure format is correct: `TIER-XXXX-XXXX-XXXX`
- Minimum 20 characters total
- Uppercase only

---

## 📝 Notes

- Keys are **not** stored anywhere by the generator
- You must save the output yourself (copy or redirect to file)
- Each run generates completely new random keys
- No validation against existing keys (implement in your backend)

