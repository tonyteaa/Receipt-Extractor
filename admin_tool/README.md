# Receipt Extractor - Admin Tool 🔑

A GUI application for managing license keys, generating new licenses, and assigning them to customers.

## Features

✅ **Dashboard** - View statistics and recent licenses  
✅ **Generate Keys** - Create Pro and Budget license keys  
✅ **Manage Licenses** - Assign licenses to customers, revoke licenses  
✅ **Settings** - Configure MySQL database connection  
✅ **Direct MySQL Integration** - All changes saved to database immediately  

---

## Installation

### Prerequisites

- Rust (latest stable)
- MySQL server running
- Backend API database initialized

### Build

```powershell
cd admin_tool
cargo build --release
```

### Run

```powershell
cargo run --release
```

---

## Usage

### 1. Dashboard Tab

View statistics:
- Total licenses
- Used licenses
- Pro vs Budget breakdown
- Recent license activity

### 2. Generate Keys Tab

1. Select tier (Pro or Budget)
2. Enter number of keys to generate
3. Click "Generate Keys"
4. Keys are automatically saved to database
5. Copy keys to clipboard

### 3. Manage Licenses Tab

**Assign License:**
1. Enter license key
2. Enter customer email
3. Enter customer name
4. Click "Assign License"

**Revoke License:**
1. Find license in the list
2. Click "Revoke" button
3. Removes all device activations
4. Resets license to available state

**Filter Licenses:**
- All - Show all licenses
- Pro - Show only Pro tier
- Budget - Show only Budget tier

### 4. Settings Tab

Configure database connection:
- Host (default: localhost)
- Port (default: 3306)
- User (default: Antoine)
- Password
- Database name (default: receipt_extractor)

Click "Reconnect" to apply changes.

---

## Database Schema

The admin tool works with these tables:

### license_keys
- `id` - Primary key
- `license_key` - Unique license key
- `tier` - "pro" or "budget"
- `is_used` - Boolean flag
- `customer_email` - Customer email (nullable)
- `customer_name` - Customer name (nullable)
- `created_at` - Timestamp
- `activated_at` - Timestamp (nullable)

### license_activations
- `id` - Primary key
- `license_key` - Foreign key
- `device_fingerprint` - SHA-256 hash
- `device_name` - Device hostname
- `activated_at` - Timestamp

---

## Workflow

### Generating and Assigning Licenses

1. **Generate Keys**
   - Go to "Generate Keys" tab
   - Select tier and count
   - Click "Generate Keys"
   - Keys are saved to database

2. **Assign to Customer**
   - Go to "Manage Licenses" tab
   - Copy a generated key
   - Enter customer details
   - Click "Assign License"

3. **Customer Activates**
   - Customer enters key in Rust app
   - App validates with backend API
   - Device fingerprint saved
   - License marked as used

4. **Revoke if Needed**
   - Go to "Manage Licenses" tab
   - Find the license
   - Click "Revoke"
   - Customer can re-activate on new devices

---

## Security

🔒 **Database Password** - Stored in memory only, not saved to disk  
🔒 **Direct Connection** - No API layer, connects directly to MySQL  
🔒 **Admin Only** - This tool should only be used by administrators  

---

## Troubleshooting

### "Database connection failed"
- Check MySQL is running
- Verify credentials in Settings tab
- Check firewall settings
- Ensure database exists

### "Failed to insert key"
- Check database permissions
- Verify table schema is correct
- Check for duplicate keys (very rare)

### "Failed to assign license"
- Verify license key exists
- Check license is not already used
- Verify customer email format

---

## Future Enhancements

- [ ] Export licenses to CSV
- [ ] Import licenses from CSV
- [ ] Email license keys to customers
- [ ] View activation history
- [ ] Bulk operations
- [ ] Search and filter improvements
- [ ] License expiration dates
- [ ] Usage analytics

---

## License

This admin tool is part of the Receipt Extractor project.
For internal use only.

