# User Management Guide

## Overview

The admin tool now includes a **User Management** tab that allows you to search for customers and view all their associated information including licenses and device activations.

## Features

### 1. Device Fingerprinting (Already Implemented)

Every time a user activates a license, the system automatically tracks:
- **Device Fingerprint**: SHA-256 hash of hostname + username (unique per computer)
- **Device Name**: Human-readable computer name
- **Activation Time**: When the license was first activated on this device
- **Last Seen**: Last time the license was validated
- **IP Address**: IP address of the device (if available)
- **Status**: Active or Inactive

### 2. Activation Limits

- Each license can be activated on **2 devices** by default (configurable in `backend/.env`)
- Attempting to activate on a 3rd device will be rejected
- Admin can deactivate devices to free up slots

### 3. User Management Tab

#### Search Users
- Search by email or name (partial match)
- Shows number of licenses per user
- Click on a user to view details

#### View User Details
- **Email and Name**: Customer information
- **Licenses**: All licenses owned by this user
  - License key
  - Tier (PRO or BUDGET)
  - Creation date
  - Assignment date
- **Device Activations**: All devices using this user's licenses
  - Device name
  - Device fingerprint (first 16 characters shown)
  - Activation date
  - Last seen date
  - Active/Inactive status
  - Deactivate button (for active devices)

## How to Use

### Admin Tool

1. **Open Admin Tool**
   ```powershell
   cd admin_tool
   cargo run --release
   ```

2. **Go to "👥 User Management" tab**

3. **Search for a user**
   - Enter email or name in search box
   - Press Enter or click "Search"

4. **View user details**
   - Click on a user from search results
   - See all their licenses and devices

5. **Deactivate a device** (if needed)
   - Click "🚫 Deactivate Device" button
   - This frees up an activation slot

### API Endpoints (for programmatic access)

The backend also provides REST API endpoints for querying:

```bash
# Search by customer email
GET /api/admin/search/customer?email=user@example.com

# Get all activations for a license
GET /api/admin/license/PRO-XXXX-XXXX-XXXX/activations

# Search by device fingerprint
GET /api/admin/search/device?fingerprint=abc123...

# Universal search (searches everything)
GET /api/admin/search/all?q=searchterm
```

### Testing the API

Run the test script:
```powershell
cd backend
.\test-search.ps1
```

## Database Schema

### license_keys table
- Stores license information
- Links to customer via `customer_email` and `customer_name`

### license_activations table
- Stores device activations
- Links to license via `license_key` (foreign key)
- Unique constraint on `(license_key, device_fingerprint)` - prevents duplicate activations
- Tracks `is_active` status

## Security Features

1. **Device Fingerprinting**: Prevents license sharing across different computers
2. **Activation Limits**: Limits number of devices per license
3. **Deactivation**: Admin can remotely deactivate devices
4. **Audit Trail**: All activations are logged with timestamps and IP addresses

## Common Scenarios

### User Lost Access to Old Computer
1. Search for user in User Management tab
2. Find the old device activation
3. Click "Deactivate Device"
4. User can now activate on new computer

### Check if User is Sharing License
1. Search for user
2. Check device activations
3. Look for:
   - Multiple devices with different fingerprints
   - Devices with different names
   - Suspicious last_seen patterns

### Find All Users with PRO Licenses
1. Go to "Manage Licenses" tab
2. Filter by "PRO" tier
3. Filter by "Assigned" status
4. See all assigned PRO licenses with customer emails

## Next Steps

- **Deploy backend** to production server
- **Update API URL** in Rust app to production endpoint
- **Set up monitoring** for suspicious activation patterns
- **Create customer portal** for users to manage their own devices

