# OpenVoice

Voice-to-clipboard transcription app using OpenRouter API.

## Features

- **Global Shortcut**: Press `Ctrl+Shift+R` to start/stop recording
- **Transparent Overlay**: Full-screen click-through window with visual feedback
- **Auto-clipboard**: Transcribed text is automatically copied to clipboard
- **Configurable**: Select audio input device and set API key via settings

## Requirements

### System Dependencies (Ubuntu/Debian)

```bash
# Tauri dependencies
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

# Audio dependencies
sudo apt install libasound2-dev
```

### Rust

```bash
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```

### Bun (recommended) or Node.js

```bash
curl -fsSL https://bun.sh/install | bash
```

## Setup

1. Clone the repository
2. Install dependencies:
   ```bash
   bun install
   ```

3. Get an API key from [OpenRouter](https://openrouter.ai/keys)

## Development

```bash
bun run dev
```

## Build

```bash
bun run build
```

The built app will be in `src-tauri/target/release/openvoice`

## Usage

1. **Start the app** - A system tray icon will appear
2. **Configure** - Click the tray icon or use settings to:
   - Enter your OpenRouter API key
   - Select audio input device
3. **Record** - Press `Ctrl+Shift+R` to start recording
   - A green pulsing border will appear around your screen
4. **Stop** - Press `Ctrl+Shift+R` again to stop
   - The audio will be transcribed and copied to clipboard
5. **Paste** - Use `Ctrl+V` to paste the transcription anywhere

## Visual Feedback

| State | Border Color |
|-------|--------------|
| Recording | Green (pulsing) |
| Processing | Orange (pulsing) |
| Success | Green (fade out) |
| Error | Red |

## Technical Details

- **Audio Format**: WAV (mono, 16kHz, 16-bit)
- **Transcription Model**: google/gemini-2.5-flash (via OpenRouter)
- **Framework**: Tauri v2
- **Audio Library**: cpal

## Troubleshooting

### No audio devices found

Make sure you have ALSA installed and your microphone is connected:
```bash
arecord -l
```

### Global shortcut not working

On Wayland, global shortcuts may require additional permissions. Try running with:
```bash
WAYLAND_DISPLAY= ./openvoice  # Force X11
```

### API errors

- Check your OpenRouter API key
- Ensure you have credits in your OpenRouter account
- Check your internet connection

## License

MIT
