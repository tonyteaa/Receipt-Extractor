#!/bin/bash

# Test script for Receipt Extractor API
# Usage: bash test-api.sh

API_URL="http://localhost:3000"

echo "🧪 Testing Receipt Extractor API"
echo "=================================="
echo ""

# Test 1: Health Check
echo "1️⃣  Testing health endpoint..."
curl -s "$API_URL/health" | json_pp
echo ""
echo ""

# Test 2: Validate Pro License
echo "2️⃣  Testing Pro license validation..."
curl -s -X POST "$API_URL/api/license/validate" \
  -H "Content-Type: application/json" \
  -d '{
    "license_key": "PRO-LVBZ-HHB3-1DCS",
    "device_fingerprint": "test-device-123",
    "device_name": "Test Laptop"
  }' | json_pp
echo ""
echo ""

# Test 3: Validate Budget License
echo "3️⃣  Testing Budget license validation..."
curl -s -X POST "$API_URL/api/license/validate" \
  -H "Content-Type: application/json" \
  -d '{
    "license_key": "BUDGET-T8I5-MEAM-MRGR",
    "device_fingerprint": "test-device-456",
    "device_name": "Test Desktop"
  }' | json_pp
echo ""
echo ""

# Test 4: Invalid License
echo "4️⃣  Testing invalid license..."
curl -s -X POST "$API_URL/api/license/validate" \
  -H "Content-Type: application/json" \
  -d '{
    "license_key": "INVALID-KEY-1234"
  }' | json_pp
echo ""
echo ""

# Test 5: Admin Stats (requires admin key)
echo "5️⃣  Testing admin stats..."
echo "   (Set ADMIN_API_KEY environment variable to test)"
if [ -n "$ADMIN_API_KEY" ]; then
  curl -s "$API_URL/api/admin/stats" \
    -H "X-Admin-API-Key: $ADMIN_API_KEY" | json_pp
else
  echo "   ⚠️  Skipped (no ADMIN_API_KEY set)"
fi
echo ""
echo ""

echo "✅ Tests complete!"

