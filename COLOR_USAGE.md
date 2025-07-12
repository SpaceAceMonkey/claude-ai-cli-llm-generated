# Color Configuration Usage Guide

## Overview
This Claude AI CLI application now supports user-configurable colors for a personalized TUI experience.

## Available Colors
The application supports the following ANSI standard colors:
- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- `bright-black`, `bright-red`, `bright-green`, `bright-yellow`, `bright-blue`, `bright-magenta`, `bright-cyan`, `bright-white`

## Command Line Usage

### Basic Color Configuration
```bash
# Set background to dark blue and border to bright white
./claudecli --background-color blue --border-color bright-white

# Set user name to bright yellow and assistant name to bright cyan
./claudecli --user-name-color bright-yellow --assistant-name-color bright-cyan

# Complete color theme example
./claudecli --background-color black \
           --border-color bright-white \
           --text-color white \
           --user-name-color bright-blue \
           --assistant-name-color bright-green
```

### Environment Variables
You can also set colors using environment variables:
```bash
export BACKGROUND_COLOR=blue
export BORDER_COLOR=bright-white
./claudecli
```

## Interactive Color Selection

### Opening the Color Dialog
While the application is running, you can:
- Press `Alt+Shift+C` to open the color configuration dialog
- Type `/colors` and press Enter to open the dialog
- Type `/colors` and press Space to open the dialog

### Using the Color Dialog
1. **Navigation**: Use `Left` and `Right` arrow keys to navigate between color type options (Background, Border, Text, etc.)
2. **Selection**: Use `Up` and `Down` arrow keys to navigate between available colors for the selected type
3. **Apply**: Press `Enter` to apply the selected colors
4. **Cancel**: Press `Esc` to cancel changes and return to the main interface

### Color Options in Dialog
- **Background Color**: The main background color of the application
- **Border Color**: The color of window borders and dividers
- **Text Color**: The color of regular text content
- **User Name Color**: The color used for displaying your name in conversations
- **Assistant Name Color**: The color used for displaying the assistant's name

## Default Colors
If no colors are specified, the application uses these defaults:
- Background: Black
- Border: White
- Text: White
- User Name: Bright Blue
- Assistant Name: Bright Green

## Color Persistence
Color changes made through the interactive dialog are applied immediately to the current session. For permanent changes, use the command line arguments when starting the application.

## Examples

### Professional Theme
```bash
./claudecli --background-color black \
           --border-color white \
           --text-color white \
           --user-name-color bright-blue \
           --assistant-name-color bright-green
```

### Dark Theme
```bash
./claudecli --background-color bright-black \
           --border-color bright-white \
           --text-color bright-white \
           --user-name-color bright-cyan \
           --assistant-name-color bright-magenta
```

### Light Theme
```bash
./claudecli --background-color white \
           --border-color black \
           --text-color black \
           --user-name-color blue \
           --assistant-name-color green
```

## Troubleshooting

### Invalid Color Names
If you specify an invalid color name, you'll see an error message. Make sure to use one of the supported ANSI color names listed above.

### Terminal Compatibility
The color display depends on your terminal's support for ANSI colors. Most modern terminals support these colors, but some older or minimal terminals might not display them correctly.

### Color Contrast
Choose colors with sufficient contrast for readability. For example, avoid using dark text on a dark background or light text on a light background.
