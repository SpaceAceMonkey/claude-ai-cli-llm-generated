# Color Configuration Hotkey and Command Update

## Changes Made

### Updated Keyboard Shortcut
- **Old**: `Ctrl+C` (inappropriate - conflicts with terminal interrupt)
- **New**: `Alt+Shift+C` (safe and unlikely to conflict)

### Added Slash Command
- **New**: `/colors` command to open color configuration dialog
- Works both with Enter and Space after typing `/colors`

## Why Alt+Shift+C?

1. **Safe Choice**: `Alt+Shift+C` is unlikely to be intercepted by:
   - Terminal applications
   - Operating system shortcuts
   - Other applications

2. **Intuitive**: The 'C' stands for "Colors" making it memorable

3. **Standard Practice**: Multi-modifier combinations are commonly used for application-specific shortcuts

## Updated Files

### Code Changes
- `src/handlers/events/shortcuts.rs` - Updated keyboard shortcut handler
- `src/handlers/events/input.rs` - Added `/colors` slash command support

### Documentation Updates
- `README.md` - Updated keyboard shortcuts and commands sections
- `COLOR_USAGE.md` - Updated interactive color selection instructions

## Usage Examples

### Keyboard Shortcut
```
# While application is running:
Alt+Shift+C  # Opens color configuration dialog
```

### Slash Command
```
# In the input field:
/colors      # Then press Enter
/colors      # Then press Space (also works)
```

## Testing

Both methods successfully:
1. Open the color configuration dialog
2. Reset dialog state to default selection
3. Clear the input field (for slash command)
4. Maintain all existing functionality

The application compiles and runs successfully with these changes.
