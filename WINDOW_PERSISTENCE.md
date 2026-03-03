# Window Persistence and Controls

## Overview
The application now has standard window controls (minimize, maximize, close) and remembers your window size and position between sessions.

## Features Added

### 1. Window Control Buttons
The application now has standard window controls:
- ✅ **Minimize button** - Minimize the window to taskbar
- ✅ **Maximize/Restore button** - Toggle between maximized and windowed mode
- ✅ **Close button** - Close the application

### 2. Window Size and Position Persistence
The application now remembers:
- ✅ **Window size** - The last size you set
- ✅ **Window position** - Where you placed the window on screen
- ✅ **Maximized state** - Whether the window was maximized

When you close and reopen the app, it will restore to the same size, position, and state.

## Implementation Details

### Changes Made

#### `src/main.rs` (Lines 11-18)
Added window configuration options:

```rust
viewport: egui::ViewportBuilder::default()
    .with_inner_size([900.0, 700.0])        // Default size on first launch
    .with_min_inner_size([600.0, 400.0])    // Minimum allowed size
    .with_decorations(true)                  // Enable title bar with buttons
    .with_resizable(true)                    // Allow window resizing
    .with_maximize_button(true)              // Enable maximize button
    .with_minimize_button(true),             // Enable minimize button
persist_window: true,                        // Save window state between sessions
```

### How It Works

1. **First Launch**: 
   - Window opens at default size (900x700 pixels)
   - Positioned by the OS (usually centered)

2. **Subsequent Launches**:
   - Window opens at the last saved size
   - Positioned where you last placed it
   - Restores maximized state if it was maximized

3. **Persistence Storage**:
   - Window state is saved automatically by eframe
   - Stored in platform-specific config directory:
     - **Linux**: `~/.config/receipt_extractor/`
     - **Windows**: `%APPDATA%\receipt_extractor\`
     - **macOS**: `~/Library/Application Support/receipt_extractor/`

### User Experience

#### Resizing the Window
- Drag any edge or corner to resize
- Minimum size: 600x400 pixels
- Maximum size: Your screen size

#### Maximizing the Window
- Click the maximize button (square icon) in title bar
- Or double-click the title bar
- Click again to restore to previous size

#### Minimizing the Window
- Click the minimize button in title bar
- Window goes to taskbar/dock
- Click taskbar icon to restore

## Testing

After rebuilding:
```bash
cargo build --release
./run.sh
```

Try these actions:
1. ✅ Resize the window to a custom size
2. ✅ Move it to a different position on screen
3. ✅ Close the application
4. ✅ Reopen - it should restore to the same size and position
5. ✅ Maximize the window
6. ✅ Close and reopen - it should open maximized
7. ✅ Restore to windowed mode
8. ✅ Close and reopen - it should open in windowed mode at the last size

## Benefits

1. **Better User Experience**: No need to resize every time you open the app
2. **Workflow Efficiency**: Window stays where you want it
3. **Multi-Monitor Support**: Remembers which monitor you used
4. **Standard Controls**: Familiar maximize/minimize buttons like other apps

## Related Files
- `src/main.rs` - Window configuration
- `src/app.rs` - Application implementation (no changes needed for persistence)

