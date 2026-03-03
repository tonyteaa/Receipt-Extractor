# PowerShell test script for Receipt Extractor API
# Usage: .\test-api.ps1

$API_URL = "http://127.0.0.1:3000"

Write-Host "🧪 Testing Receipt Extractor API" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

# Test 1: Health Check
Write-Host "1️⃣  Testing health endpoint..." -ForegroundColor Yellow
$response = Invoke-RestMethod -Uri "$API_URL/health" -Method Get
$response | ConvertTo-Json
Write-Host ""

# Test 2: Validate Pro License
Write-Host "2️⃣  Testing Pro license validation..." -ForegroundColor Yellow
$body = @{
    license_key = "PRO-JIMK-NJ7G-AZRO"
    device_fingerprint = "test-device-123"
    device_name = "Test Laptop"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$API_URL/api/license/validate" -Method Post -Body $body -ContentType "application/json"
    $response | ConvertTo-Json
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Validate Budget License
Write-Host "3️⃣  Testing Budget license validation..." -ForegroundColor Yellow
$body = @{
    license_key = "BUDGET-4RGW-8MCE-LM46"
    device_fingerprint = "test-device-456"
    device_name = "Test Desktop"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$API_URL/api/license/validate" -Method Post -Body $body -ContentType "application/json"
    $response | ConvertTo-Json
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 4: Invalid License
Write-Host "4️⃣  Testing invalid license..." -ForegroundColor Yellow
$body = @{
    license_key = "INVALID-KEY-1234"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$API_URL/api/license/validate" -Method Post -Body $body -ContentType "application/json"
    $response | ConvertTo-Json
} catch {
    Write-Host "Expected error: $_" -ForegroundColor Gray
}
Write-Host ""

# Test 5: Admin Stats
Write-Host "5️⃣  Testing admin stats..." -ForegroundColor Yellow
$adminKey = $env:ADMIN_API_KEY
if ($adminKey) {
    try {
        $headers = @{
            "X-Admin-API-Key" = $adminKey
        }
        $response = Invoke-RestMethod -Uri "$API_URL/api/admin/stats" -Method Get -Headers $headers
        $response | ConvertTo-Json -Depth 5
    } catch {
        Write-Host "Error: $_" -ForegroundColor Red
    }
} else {
    Write-Host "   ⚠️  Skipped (no ADMIN_API_KEY environment variable set)" -ForegroundColor Gray
}
Write-Host ""

Write-Host "✅ Tests complete!" -ForegroundColor Green

