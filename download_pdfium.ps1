# Download PDFium library for Windows x64
$url = "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7568/pdfium-win-x64.tgz"
$output = "pdfium-win-x64.tgz"

Write-Host "Downloading PDFium library..."
Invoke-WebRequest -Uri $url -OutFile $output

Write-Host "Extracting..."
tar -xzf $output

Write-Host "Copying DLL to project directory..."
Copy-Item "pdfium-win-x64\bin\pdfium.dll" -Destination "." -Force

Write-Host "Done! pdfium.dll is now in the project directory."
Write-Host "You can delete the pdfium-win-x64 folder and pdfium-win-x64.tgz file if you want."

