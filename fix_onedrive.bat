@echo off
echo Fixing OneDrive sync conflict...
echo.

echo Step 1: Cleaning build artifacts...
if exist target rmdir /s /q target
if exist Cargo.lock del Cargo.lock

echo.
echo Step 2: Setting target folder to not sync with OneDrive...
echo Creating target folder...
mkdir target

echo.
echo Step 3: Marking target folder as "Free up space" (OneDrive won't sync)...
attrib +U target

echo.
echo Done! Now try building again with: cargo build --release
echo.
pause

