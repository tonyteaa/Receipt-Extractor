# Test Shopify webhook locally
$orderId = "test-order-$(Get-Date -Format 'yyyyMMddHHmmss')"
$orderNumber = "#TEST-$(Get-Random -Minimum 1000 -Maximum 9999)"

$body = @{
    id = $orderId
    order_number = $orderNumber
    email = "new-customer-$(Get-Random)@example.com"
    currency = "USD"
    customer = @{
        first_name = "New"
        last_name = "Customer"
        email = "new-customer-$(Get-Random)@example.com"
    }
    line_items = @(
        @{
            title = "Receipt Extractor Pro License"
            sku = "RECEIPT-PRO"
            quantity = 1
            price = "49.99"
        }
    )
    total_price = "49.99"
    created_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
} | ConvertTo-Json -Depth 10

Write-Host "`n🧪 Testing Shopify Webhook...`n" -ForegroundColor Cyan
Write-Host "Order ID: $orderId" -ForegroundColor Yellow
Write-Host "Customer: test-customer@example.com" -ForegroundColor Yellow
Write-Host "Product: Receipt Extractor Pro License`n" -ForegroundColor Yellow

try {
    $response = Invoke-RestMethod -Uri "http://127.0.0.1:3000/api/shopify/order-created" `
        -Method Post `
        -Body $body `
        -ContentType "application/json" `
        -Headers @{
            "X-Shopify-Topic" = "orders/create"
            "X-Shopify-Shop-Domain" = "test-shop.myshopify.com"
        }
    
    Write-Host "✅ SUCCESS! Webhook processed`n" -ForegroundColor Green
    Write-Host "Response:" -ForegroundColor Cyan
    $response | ConvertTo-Json -Depth 3
    
    if ($response.license_key) {
        Write-Host "`n🎉 License Created:" -ForegroundColor Green
        Write-Host "   License Key: $($response.license_key)" -ForegroundColor Yellow
        Write-Host "   Tier: $($response.tier)" -ForegroundColor Yellow
        Write-Host "   Customer: $($response.customer_email)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ FAILED!" -ForegroundColor Red
    Write-Host "Error: $_" -ForegroundColor Red
}

