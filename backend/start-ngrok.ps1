# Start ngrok without cloud endpoint
Write-Host "`n🚀 Starting ngrok tunnel to port 3000...`n" -ForegroundColor Cyan

# Kill any existing ngrok processes
Get-Process -Name "ngrok" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2

# Start ngrok with a fresh random URL
Write-Host "Starting ngrok (this will give you a random URL)..." -ForegroundColor Yellow
Write-Host "Press Ctrl+C to stop ngrok when done.`n" -ForegroundColor Gray

# Start ngrok in the background and show the URL
$ngrokJob = Start-Process -FilePath "ngrok" -ArgumentList "http", "3000", "--log=stdout" -NoNewWindow -PassThru

# Wait a moment for ngrok to start
Start-Sleep -Seconds 3

# Get the ngrok URL from the API
try {
    $tunnels = Invoke-RestMethod -Uri "http://127.0.0.1:4040/api/tunnels"
    $publicUrl = $tunnels.tunnels[0].public_url
    
    Write-Host "✅ ngrok is running!" -ForegroundColor Green
    Write-Host "`n📋 Your ngrok URL:" -ForegroundColor Cyan
    Write-Host "   $publicUrl" -ForegroundColor Yellow
    Write-Host "`n🔗 Shopify webhook URL:" -ForegroundColor Cyan
    Write-Host "   $publicUrl/api/shopify/order-created" -ForegroundColor Yellow
    Write-Host "`n🌐 ngrok Web Interface:" -ForegroundColor Cyan
    Write-Host "   http://127.0.0.1:4040" -ForegroundColor Yellow
    Write-Host "`n⚠️  Remember: This URL changes every time you restart ngrok!" -ForegroundColor Red
    Write-Host "   You'll need to update your Shopify webhook URL each time.`n" -ForegroundColor Red
    
    # Test the connection
    Write-Host "🧪 Testing connection..." -ForegroundColor Cyan
    $health = Invoke-RestMethod -Uri "$publicUrl/health"
    Write-Host "✅ Backend is reachable through ngrok!`n" -ForegroundColor Green
    
} catch {
    Write-Host "⚠️  Could not get ngrok URL automatically." -ForegroundColor Yellow
    Write-Host "   Open http://127.0.0.1:4040 in your browser to see the URL.`n" -ForegroundColor Yellow
}

Write-Host "Press any key to stop ngrok..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# Stop ngrok
Stop-Process -Id $ngrokJob.Id -Force
Write-Host "`n✅ ngrok stopped.`n" -ForegroundColor Green

