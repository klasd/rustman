# Quick Test Instructions

## Step-by-step to test the input box:

1. **Start the app**
   ```bash
   cargo run
   ```

2. **Press 'n'**
   - You should now see the Request panel show:
     - Mode: Creating Connection Name
     - Input: []
     - "Enter to confirm | Esc to cancel"

3. **Type a name** (e.g., `myapi`)
   - Input: [myapi]

4. **Press Enter**
   - The connection "myapi" is created
   - You should see a message: "✓ Connection 'myapi' created"

5. **Press 'u'** to edit URL
   - Mode: Editing URL
   - Input: []

6. **Type URL** (e.g., `jsonplaceholder.typicode.com`)
   - Input: [jsonplaceholder.typicode.com]

7. **Press Enter**
   - You'll see: "✓ URL updated: jsonplaceholder.typicode.com"

8. **Press 'p'** to edit port
   - Mode: Editing Port

9. **Type port** (e.g., `80`)

10. **Press Enter**
    - You'll see: "✓ Port updated: 80"

11. **Press 'r'** to send request
    - You'll see the response in the Response panel

12. **Press 's'** to save
    - File "myapi.json" will be created in current directory

13. **Press Ctrl+Q** to exit

---

## Key Points:
- Input box now shows in all modes (even when no connections exist yet)
- Instructions are visible: "Enter to confirm | Esc to cancel"
- All feedback messages appear at the top of the screen
- Mode indicator always visible in Request panel
