# Rustman - A TUI-based Postman Client for Rust

A lightweight, terminal-based HTTP client built with Rust and Ratatui, inspired by Postman. Manage HTTP connections, craft requests, and view responses all from your terminal.

## Features

- 🚀 **Lightweight TUI** - Built with Ratatui for a responsive, modern terminal interface
- 📝 **Connection Management** - Create, save, and manage multiple HTTP connections
- 🔗 **Query Parameters** - Add and manage query strings for your requests
- 📦 **Payload Support** - Send POST/PUT/PATCH requests with JSON or custom payloads
- 💾 **Persistent Storage** - Save connections as JSON files and auto-load them on startup
- 🎯 **HTTP Methods** - Support for GET, POST, PUT, PATCH methods
- ⚡ **Async Requests** - Non-blocking HTTP requests powered by Tokio and Reqwest
- 🔒 **HTTPS Support** - Automatic HTTPS detection on port 443
- ⏱️ **Request Timeout** - 10-second timeout with visual feedback and cancellation support
- 🎨 **Auto-Format Responses** - JSON pretty-printing and smart text wrapping

## Installation

### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))

### Building from Source

```bash
git clone <repository-url>
cd rustman
cargo build --release
./target/release/rustman
```

Or run directly:

```bash
cargo run
```

## Quick Start

1. **Launch the application**
   ```bash
   cargo run
   ```
   Your previously saved connections will automatically load.

2. **Create a new connection** (press `n`)
   - A dialog box appears in the Request panel
   - Enter a name for your connection (e.g., "my-api")
   - Press `Enter` to confirm
   - You'll see: "✓ Connection 'my-api' created"

3. **Edit the connection details** (press `e`)
   - A dialog appears showing all fields: Name, URL, Port, Method
   - Each field is highlighted when active
   - Press `Tab` to move to the next field
   - Press `Shift+Tab` to move to the previous field
   - Edit each field as needed (e.g., change URL to `api.example.com`)
   - Press `Enter` to confirm each field
   - Press `Esc` at any time to cancel editing

4. **Send a request** (press `r`)
   - A "Connecting..." dialog appears with the target URL
   - You can press `Esc` to cancel if needed
   - The response appears in the Response panel (or timeout/error after 10 seconds)
   - Status code and body will be displayed
   - Use Page Up/Down to scroll large responses

5. **Save your connection** (press `s`)
   - Your connection is saved as `<connection-name>.json`
   - Connections automatically load when you restart the app

## Connection Management

Connections are displayed by **name** in the left panel, not by URL. This lets you create multiple connections to the same URL with different settings:
- "api-dev" pointing to `api.example.com:3000` (development)
- "api-prod" pointing to `api.example.com:443` (production)
- "api-test" pointing to `api.example.com:8000` (testing)

## Controls & Shortcuts

### Navigation & Connection Management

| Key | Action |
|-----|--------|
| `n` | Create a new connection |
| `e` | Edit selected connection (all fields in one dialog) |
| `d` | Delete the selected connection |
| `↑` / `↓` | Navigate between connections |
| `s` | Save selected connection to JSON file |
| `l` | Load connection from JSON file (coming soon) |

### Request Execution & Connection Status

| Key | Action |
|-----|--------|
| `r` | Send HTTP request |
| `Esc` | Cancel active request (while Connecting dialog is shown) |

### Response Viewing

| Key | Action |
|-----|--------|
| `Page Down` | Scroll response down 5 lines |
| `Page Up` | Scroll response up 5 lines |
| `Home` | Jump to top of response |
| `End` | Jump to bottom of response |

### Edit Dialog Controls

When editing a connection with `e`, the following controls are available:

| Key | Action |
|-----|--------|
| `Tab` | Move to next field |
| `Shift+Tab` | Move to previous field |
| `Enter` | Confirm field edit |
| `Esc` | Cancel editing |

### Global

| Key | Action |
|-----|--------|
| `Ctrl+Q` | Exit the application |

## Visual Feedback

Rustman provides real-time feedback for every action:

- **Status Messages**: When you press a key, a message appears at the top of the screen
  - ✓ Green checkmarks for successful operations (create, save, update)
  - ✗ Red X marks for errors
  - Messages auto-dismiss after ~3 seconds

- **Input Mode Indicator**: The Request panel always shows your current mode
  - "Mode: Creating Connection Name" when creating a connection
  - "Mode: Editing URL" when editing the URL
  - "Mode: Normal" when no input is active

- **Input Box Display**: 
   - When in input mode, you see: `Input: [your text here]`
   - Instructions appear: "Enter to confirm | Esc to cancel"
   - This helps you know you're in edit mode

## Response Formatting

Rustman automatically formats responses for better readability:

- **JSON Responses**: Automatically pretty-printed with proper indentation
  - Nested objects and arrays are properly formatted
  - Easy to read and navigate

- **Line Wrapping**: Long lines are wrapped to fit your terminal width
  - No more horizontal scrolling through huge HTML pages
  - Text breaks at word boundaries for readability

- **Scrolling**: Use Page Up/Down or Home/End to navigate long responses
  - Perfect for viewing large API responses
  - All response content is accessible even in small terminals

- **Plain Text**: Non-JSON responses are displayed as-is with proper line breaks

## UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ Connections (25%)       │     Request Editor (75%)          │
│ - Connection 1          │ Method: GET                       │
│ - Connection 2          │ URL: api.example.com              │
│ - Connection 3          │ Port: 3000                        │
│                         │                                    │
│                         │ Query Parameters:                 │
│                         │   page=1                          │
│                         │   limit=10                        │
│                         │                                    │
│                         │ Payload:                          │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│ Response (20%)                                               │
│ Status: 200                                                  │
│                                                              │
│ Body:                                                        │
│ { "message": "Success", "data": [...] }                    │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│ Shortcuts (15%)                                              │
│ Controls: n-new d-delete e-edit r-send s-save                │
│ Request: Esc-cancel  Response: PgUp/PgDn-scroll             │
└─────────────────────────────────────────────────────────────┘
```

## Project Structure

```
rustman/
├── src/
│   ├── main.rs          # Application entry point, event loop
│   ├── app.rs           # Application state and logic
│   ├── models.rs        # Data structures (Connection, ApiResponse)
│   ├── ui.rs            # Ratatui UI rendering
│   └── handlers.rs      # Input handling and HTTP requests
├── Cargo.toml           # Dependencies
└── README.md            # This file
```

## Connection File Format

Connections are saved as JSON files with the following structure:

```json
{
  "name": "my-api",
  "url": "api.example.com",
  "port": 3000,
  "method": "GET",
  "query_params": {
    "page": "1",
    "limit": "10"
  },
  "payload": null
}
```

You can manually edit these JSON files or load them back into the application.

## Dependencies

- **ratatui** - TUI framework for building terminal applications
- **tokio** - Asynchronous runtime for handling async operations
- **reqwest** - HTTP client library
- **serde & serde_json** - Serialization/deserialization
- **crossterm** - Terminal manipulation (keyboard, mouse, colors)
- **anyhow** - Error handling

## Roadmap

### ✅ Completed
- [x] Save connections to JSON files
- [x] Auto-load connections from JSON files on startup
- [x] Connection editing with unified dialog
- [x] Request timeout (10 seconds) with visual feedback
- [x] Cancel active requests
- [x] HTTPS auto-detection on port 443
- [x] Input dialogs for creating connections
- [x] Response auto-formatting (JSON pretty-printing, text wrapping)

### 🔄 In Progress / Planned
- [ ] Load connections from JSON files via UI
- [ ] HTTP method selector (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
- [ ] Query parameter editor dialog
- [ ] Request body/payload editor dialog
- [ ] Response syntax highlighting (JSON, XML, HTML)
- [ ] Request history
- [ ] Authentication support (Basic Auth, Bearer tokens)
- [ ] Environment variables
- [ ] Response search/filter
- [ ] Request/Response logging
- [ ] Collections/folders organization

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the MIT License.

## Troubleshooting

### Application doesn't start
- Ensure you have Rust 1.70 or later: `rustc --version`
- Try cleaning the build: `cargo clean && cargo build`

### Terminal display issues
- Some terminals may not support all features. Try a different terminal emulator.
- If colors look off, try exporting `TERM=xterm-256color`

### HTTP requests failing
- Ensure your URL and port are correct
- Check that the target server is reachable from your network
- Try testing with `curl` first to verify connectivity

## Support

For issues, questions, or suggestions, please open an issue on GitHub or check the project's documentation.

---

**Built with ❤️ in Rust**
