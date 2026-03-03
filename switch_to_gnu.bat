@echo off
echo Switching to GNU toolchain (no Visual Studio needed)...
echo.

echo Installing GNU toolchain...
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu

echo.
echo Done! Now try building with: cargo build --release
pause

