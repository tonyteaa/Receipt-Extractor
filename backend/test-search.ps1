# Test script for new search endpoints
# Make sure the backend server is running first!

$baseUrl = "http://127.0.0.1:3000"

Write-Host "🔍 Testing License Search Endpoints" -ForegroundColor Cyan
Write-Host "===================================`n" -ForegroundColor Cyan

# Test 1: Search by customer email
Write-Host "1️⃣  Search by customer email (partial match)..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$baseUrl/api/admin/search/customer?email=test" -Method Get
    Write-Host "Found $($response.count) licenses:" -ForegroundColor Green
    $response.licenses | ForEach-Object {
        Write-Host "  - $($_.license_key) | $($_.tier) | $($_.customer_email) | Devices: $($_.active_devices)" -ForegroundColor White
    }
} catch {
    Write-Host "Error: $_" -ForegroundColor Red
}

Write-Host ""

# Test 2: Get activations for a specific license
Write-Host "2️⃣  Get device activations for a license..." -ForegroundColor Yellow
Write-Host "Enter a license key (or press Enter to skip): " -NoNewline -ForegroundColor Gray
$licenseKey = Read-Host

if ($licenseKey) {
    try {
        $response = Invoke-RestMethod -Uri "$baseUrl/api/admin/license/$licenseKey/activations" -Method Get
        Write-Host "Found $($response.count) activations for $($response.license_key):" -ForegroundColor Green
        $response.activations | ForEach-Object {
            $status = if ($_.is_active) { "✅ Active" } else { "❌ Inactive" }
            Write-Host "  - Device: $($_.device_name)" -ForegroundColor White
            Write-Host "    Fingerprint: $($_.device_fingerprint.Substring(0, 16))..." -ForegroundColor Gray
            Write-Host "    Activated: $($_.activated_at) | Last seen: $($_.last_seen)" -ForegroundColor Gray
            Write-Host "    Status: $status" -ForegroundColor $(if ($_.is_active) { "Green" } else { "Red" })
            Write-Host ""
        }
    } catch {
        Write-Host "Error: $_" -ForegroundColor Red
    }
}

Write-Host ""

# Test 3: Universal search
Write-Host "3️⃣  Universal search (searches everything)..." -ForegroundColor Yellow
Write-Host "Enter search term (or press Enter to skip): " -NoNewline -ForegroundColor Gray
$searchTerm = Read-Host

if ($searchTerm) {
    try {
        $response = Invoke-RestMethod -Uri "$baseUrl/api/admin/search/all?q=$searchTerm" -Method Get
        Write-Host "`nSearch results for '$($response.query)':" -ForegroundColor Green
        
        Write-Host "`n📋 Licenses ($($response.results.licenses.count)):" -ForegroundColor Cyan
        $response.results.licenses.data | ForEach-Object {
            Write-Host "  - $($_.license_key) | $($_.tier) | $($_.customer_email)" -ForegroundColor White
        }
        
        Write-Host "`n💻 Devices ($($response.results.devices.count)):" -ForegroundColor Cyan
        $response.results.devices.data | ForEach-Object {
            Write-Host "  - $($_.device_name) on license $($_.license_key)" -ForegroundColor White
            Write-Host "    Customer: $($_.customer_email) | Tier: $($_.tier)" -ForegroundColor Gray
        }
    } catch {
        Write-Host "Error: $_" -ForegroundColor Red
    }
}

Write-Host "`n✅ Search tests complete!" -ForegroundColor Green
Write-Host "`n📚 Available Search Endpoints:" -ForegroundColor Cyan
Write-Host "  GET /api/admin/search/customer?email=<email>" -ForegroundColor White
Write-Host "  GET /api/admin/license/<key>/activations" -ForegroundColor White
Write-Host "  GET /api/admin/search/device?fingerprint=<fingerprint>" -ForegroundColor White
Write-Host "  GET /api/admin/search/all?q=<searchterm>" -ForegroundColor White

