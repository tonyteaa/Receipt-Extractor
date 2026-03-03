# Receipt Extractor Backend API

This document describes the backend API server for the Receipt Data Extractor application.

## Overview

The backend is a Node.js/Express API server that handles:
- **License Validation** - Validates and activates license keys
- **Usage Tracking** - Tracks monthly receipt processing limits
- **Shopify Integration** - Automatically processes orders and assigns licenses
- **Admin API** - Management endpoints for the Admin Tool

## Deployment

The backend is deployed on **Railway** and runs 24/7.

| Component | Details |
|-----------|---------|
| **API URL** | `https://clever-vision-production.up.railway.app` |
| **Database** | MySQL on Railway |
| **Runtime** | Node.js 22 |

## Database Tables

| Table | Description |
|-------|-------------|
| `license_keys` | All license keys with tier, customer info, subscription plan |
| `license_activations` | Device activations (fingerprint, name, last seen) |
| `usage_tracking` | Monthly usage per license (receipts processed) |
| `usage_log` | Detailed log of each usage increment |
| `subscription_plans` | Available plans (STARTER, PERSONAL, FAMILY, POWER, UNLIMITED) |
| `shopify_orders` | Processed Shopify orders |
| `admin_users` | Admin API keys |
| `api_logs` | API call logs |

## Subscription Plans

| Plan | Receipts/Month | Price/Month | Price/Year |
|------|----------------|-------------|------------|
| STARTER | 100 | $4.99 | $49.99 |
| PERSONAL | 500 | $9.99 | $99.99 |
| FAMILY | 1,500 | $19.99 | $199.99 |
| POWER | 5,000 | $39.99 | $399.99 |
| UNLIMITED | Unlimited | $99.99 | $999.99 |

---

## API Endpoints

### Health Check

```
GET /health
```

Returns server status. No authentication required.

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2024-12-17T10:00:00.000Z",
  "service": "Receipt Extractor License API"
}
```

---

### License Endpoints

#### Validate License

```
POST /api/license/validate
```

Validates a license key and optionally activates it on a device.

**Request Body:**
```json
{
  "license_key": "PRO-XXXX-XXXX-XXXX",
  "customer_email": "user@example.com",
  "device_fingerprint": "abc123...",
  "device_name": "MyPC"
}
```

**Success Response (200):**
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

**Error Responses:**
- `400` - Invalid format or missing email
- `403` - Email mismatch, revoked, or max devices reached
- `404` - License not found

#### Deactivate Device

```
POST /api/license/deactivate-device
```

Removes a device activation to free up a slot.

**Request Body:**
```json
{
  "license_key": "PRO-XXXX-XXXX-XXXX",
  "customer_email": "user@example.com",
  "device_fingerprint": "abc123..."
}
```

---

### Usage Endpoints

#### Get Usage

```
GET /api/usage?license_key=PRO-XXXX&customer_email=user@example.com
```

Returns current usage for the billing period.

**Response:**
```json
{
  "success": true,
  "plan_code": "PERSONAL",
  "plan_name": "Personal",
  "billing_cycle": "YEARLY",
  "receipts_used": 150,
  "receipts_limit": 500,
  "receipts_remaining": 350,
  "is_unlimited": false,
  "limit_reached": false,
  "days_until_reset": 14
}
```

#### Check Usage

```
POST /api/usage/check
```

Checks if processing N receipts is allowed (without incrementing).

**Request Body:**
```json
{
  "license_key": "PRO-XXXX-XXXX-XXXX",
  "customer_email": "user@example.com",
  "count": 5
}
```

**Response:**
```json
{
  "allowed": true,
  "receipts_used": 150,
  "receipts_limit": 500,
  "receipts_remaining": 350
}
```

#### Increment Usage

```
POST /api/usage/increment
```

Increments usage count after processing.

**Request Body:**
```json
{
  "license_key": "PRO-XXXX-XXXX-XXXX",
  "customer_email": "user@example.com",
  "count": 5,
  "device_fingerprint": "abc123..."
}
```

#### Get Plans

```
GET /api/usage/plans
```

Returns all available subscription plans.

---

### Shopify Webhook

```
POST /api/shopify/order-created
```

Webhook endpoint for Shopify. When a customer purchases:
1. Generates a new license key
2. Assigns it to the customer email
3. Sends license email to customer
4. Adds license to Shopify order notes

**Headers Required:**
- `X-Shopify-Hmac-Sha256` - Webhook signature

---

### Admin Endpoints

All admin endpoints require the header:
```
X-Admin-API-Key: your-admin-api-key
```

#### Get Statistics

```
GET /api/admin/stats
```

Returns dashboard statistics (license counts, recent orders, activations).

#### Get Licenses

```
GET /api/admin/licenses?tier=PRO&is_used=true&customer_email=user@example.com
```

Returns filtered list of licenses.

#### Get Single License

```
GET /api/admin/license/:key
```

Returns detailed license info including activations.

#### Revoke License

```
POST /api/admin/license/revoke
```

Revokes a license key.

**Request Body:**
```json
{
  "license_key": "PRO-XXXX-XXXX-XXXX"
}
```

#### Delete Activation

```
DELETE /api/admin/activation/:id
```

Removes a specific device activation.

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `PORT` | Server port (default: 3000) |
| `DB_HOST` | MySQL host |
| `DB_PORT` | MySQL port |
| `DB_USER` | MySQL username |
| `DB_PASSWORD` | MySQL password |
| `DB_NAME` | MySQL database name |
| `NODE_ENV` | Environment (production/development) |
| `GROQ_API_KEY` | Groq API key for AI processing |
| `OPENAI_API_KEY` | OpenAI API key (backup) |
| `ADMIN_API_KEY` | Admin panel authentication |
| `SHOPIFY_STORE_URL` | Shopify store domain |
| `SHOPIFY_ACCESS_TOKEN` | Shopify Admin API token |
| `SHOPIFY_WEBHOOK_SECRET` | Webhook signature secret |
| `RESEND_API_KEY` | Email service API key |

---

## Rate Limiting

- 100 requests per 15 minutes per IP
- Applies to all `/api/*` endpoints

## Security

- **Helmet.js** - HTTP security headers
- **CORS** - Cross-origin requests enabled
- **HMAC Verification** - Shopify webhooks are signature-verified
- **API Key Auth** - Admin endpoints require API key

---

## Local Development

```bash
cd backend
npm install
cp .env.example .env  # Edit with your credentials
npm start
```

## Deploying Updates

From the `backend` folder:
```bash
railway up
```

This uploads and deploys the latest code to Railway.

