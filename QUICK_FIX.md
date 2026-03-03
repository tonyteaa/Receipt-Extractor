# 🔧 Quick Fix: "linker link.exe not found"

## The Problem
You're seeing this error because Rust on Windows needs a C++ compiler/linker to build native applications.

## ✅ Solution 1: Install Visual Studio Build Tools (Recommended)

### Step-by-step:

1. **Open this link**: https://visualstudio.microsoft.com/downloads/

2. **Scroll down** to find "**Tools for Visual Studio 2022**"

3. **Download**: "**Build Tools for Visual Studio 2022**"

4. **Run the installer** (vs_BuildTools.exe)

5. **Select workload**: 
   - Check ✅ **"Desktop development with C++"**
   - (You'll see it in the left panel)

6. **Click Install** 
   - Size: ~6-7 GB
   - Time: 10-15 minutes

7. **Restart your computer** (important!)

8. **Open terminal** in your project folder

9. **Try building again**:
   ```bash
   cargo build --release
   ```

### What this installs:
- MSVC compiler (Microsoft Visual C++)
- Windows SDK
- C++ build tools
- The `link.exe` linker that Rust needs

---

## ✅ Solution 2: Use GNU Toolchain (Alternative)

If you don't want to install Visual Studio Build Tools (saves ~7 GB):

### Step-by-step:

1. **Double-click** `switch_to_gnu.bat` in your project folder

   OR run these commands in terminal:
   ```bash
   rustup toolchain install stable-x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   ```

2. **Try building again**:
   ```bash
   cargo build --release
   ```

### Pros & Cons:
- ✅ No need for Visual Studio (saves disk space)
- ✅ Faster to set up
- ⚠️ May have compatibility issues with some Windows libraries
- ⚠️ Slightly larger executables

---

## Which Should I Choose?

### Choose **Solution 1 (MSVC)** if:
- You want the most compatible builds
- You have 7+ GB of disk space
- You plan to do more Rust development on Windows
- You want the best performance

### Choose **Solution 2 (GNU)** if:
- You want to start quickly
- You have limited disk space
- You just want to try this app
- You don't mind potential compatibility issues

---

## After Installing

Once you've completed either solution:

1. **Restart your terminal** (or computer for Solution 1)

2. **Navigate to project folder**:
   ```bash
   cd c:\Users\a_tru\OneDrive\Documents\Coding
   ```

3. **Build the project**:
   ```bash
   cargo build --release
   ```

4. **Wait** (first build takes 5-10 minutes)

5. **Run the app**:
   ```bash
   cargo run --release
   ```
   
   OR double-click `run.bat`

---

## Still Having Issues?

### Error: "cargo: command not found"
- Rust isn't installed or terminal wasn't restarted
- Install from: https://rustup.rs/
- Restart terminal

### Error: "failed to download dependencies"
- Check internet connection
- Try: `cargo clean` then rebuild

### Error: "could not compile..."
- Make sure you completed Solution 1 or 2 above
- Restart your computer
- Try: `cargo clean` then `cargo build --release`

### Build is very slow
- First build takes 5-10 minutes (normal!)
- Subsequent builds are much faster (30 seconds)
- Release builds are slower than debug builds

---

## Quick Commands Reference

```bash
# Check if Rust is installed
cargo --version

# Check which toolchain you're using
rustup show

# Switch to MSVC (after installing Visual Studio Build Tools)
rustup default stable-x86_64-pc-windows-msvc

# Switch to GNU (no Visual Studio needed)
rustup default stable-x86_64-pc-windows-gnu

# Clean build artifacts
cargo clean

# Build (debug mode - faster compile)
cargo build

# Build (release mode - optimized)
cargo build --release

# Build and run
cargo run --release
```

---

## Need More Help?

1. Check `SETUP.md` for detailed setup instructions
2. Check `README.md` for usage instructions
3. Check `PROJECT_STRUCTURE.md` for technical details

The error you're seeing is **very common** for Rust beginners on Windows. Once you install the build tools, everything will work smoothly! 🚀

