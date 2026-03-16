# Testing Connection Timeout and Cancel Feature

## Scenario 1: Normal Request (Google)
1. Create a new connection with name "Google Search"
   - URL: google.se
   - Port: 443
   - Method: GET
2. Press 's' to save
3. Press 'r' to send request
4. You should see a "Connecting..." dialog with:
   - Title in cyan: "─ Connecting... ─"
   - Spinning indicator: "⟳ Sending request..."
   - The URL and port being requested
   - Instructions: "Ctrl+C or Esc to cancel"
5. Wait for response (should complete in a few seconds)

## Scenario 2: Unreachable Port (Your original issue)
1. Create a new connection with name "Unreachable"
   - URL: google.se
   - Port: 43 (unreachable)
   - Method: GET
2. Press 's' to save
3. Press 'r' to send request
4. The "Connecting..." dialog appears
5. Wait up to 10 seconds:
   - If the connection succeeds, you'll see the response
   - If it times out or fails, you'll see an error message
6. You can press Esc or Ctrl+C to cancel the request immediately

## Scenario 3: Cancel Active Request
1. Start any request by pressing 'r'
2. See the "Connecting..." dialog appear
3. Press Esc or try Ctrl+C
4. The dialog should disappear and you should see "Request cancelled" message

## Features
- **10-second timeout**: Requests automatically timeout after 10 seconds
- **Manual cancellation**: Press Esc to cancel at any time
- **Visual feedback**: Dialog clearly shows the request is in progress
- **Error messages**: Failed/cancelled requests show what went wrong
