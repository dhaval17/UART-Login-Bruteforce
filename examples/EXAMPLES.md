# UART Login Bruteforce - Usage Examples

This directory contains example usage scenarios and configurations.

## Quick Start

### 1. Build the Project
```bash
cd ..
cargo build --release
cd examples
```

### 2. Prepare Your Password List
Use the included `passwords.txt` or create your own:
```bash
cat > my_passwords.txt << EOF
password1
password2
password3
EOF
```

## Scenario Examples

### Scenario 1: Standard Linux UART Login
Attempting to brute force a Linux system accessible via UART with standard login prompt.

```bash
sudo ../target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 115200 \
    --username-prompt "login:" \
    --password-prompt "Password:" \
    --username root \
    --passwords passwords.txt
```

### Scenario 2: Embedded System with Custom Prompts
Some embedded systems use custom login prompts.

```bash
sudo ../target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 9600 \
    --username-prompt "Enter User ID:" \
    --password-prompt "Enter PIN:" \
    --username admin \
    --passwords passwords.txt
```

### Scenario 3: Multiple Serial Ports
Testing different devices on different ports.

```bash
# Device 1
sudo ../target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 115200 \
    --username-prompt "Username:" \
    --password-prompt "Password:" \
    --username testuser \
    --passwords passwords.txt

# Device 2 (in another terminal)
sudo ../target/release/uart-bruteforce \
    --device /dev/ttyUSB1 \
    --baudrate 115200 \
    --username-prompt "Username:" \
    --password-prompt "Password:" \
    --username testuser \
    --passwords passwords.txt
```

### Scenario 4: Low Baud Rate Connection
Some older devices use slower baud rates with increased timeout.

```bash
sudo ../target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 9600 \
    --timeout 5000 \
    --username-prompt "login:" \
    --password-prompt "password:" \
    --username admin \
    --passwords passwords.txt
```

### Scenario 5: Custom Timeout for Slow Responses
Devices with slow response times may need increased timeout.

```bash
sudo ../target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 19200 \
    --timeout 10000 \
    --username-prompt "User:" \
    --password-prompt "Pass:" \
    --username operator \
    --passwords passwords.txt
```

## Finding Correct Prompt Strings

If you're unsure of the exact prompt strings, use `screen` or `minicom`:

```bash
# Connect to device
screen /dev/ttyUSB0 115200

# Observe the exact prompt text
# Note: Include any spaces or special characters exactly as they appear
# Exit screen with: Ctrl+A, then type 'quit'
```

## Creating Custom Password Lists

### From a Dictionary
```bash
# Create a password list from common words
cat > wordlist.txt << EOF
admin
password
letmein
default
test
EOF
```

### From Online Sources
```bash
# Download a popular wordlist (example)
wget https://raw.githubusercontent.com/danielmiessler/SecLists/master/Passwords/Common-Credentials/10-million-password-list-top-1000000.txt
```

### Filtered by Length
```bash
# Create passwords between 6-12 characters
awk 'length($0) >= 6 && length($0) <= 12' wordlist_raw.txt > wordlist_filtered.txt
```

## Monitoring UART Connection

Before running the brute force tool, verify your UART connection:

```bash
# List available serial devices
ls -la /dev/ttyUSB*

# Check device info
dmesg | grep -i usb

# Monitor UART traffic during test
sudo cat /dev/ttyUSB0 | strings  # in another terminal
```

## Performance Tips

1. **Optimize Timeout**: Adjust based on device response time
   ```bash
   --timeout 3000  # 3 seconds for fast devices
   --timeout 10000 # 10 seconds for slow devices
   ```

2. **Correct Baud Rate**: Verify the device's actual baud rate
   - Common: 9600, 19200, 38400, 57600, 115200

3. **Accurate Prompts**: Copy exact prompt strings
   - Include trailing spaces if present
   - Case-sensitive matching may be required

4. **Wordlist Size**: Start with smaller lists for testing
   - Test with 10-20 passwords first
   - Verify the tool is working correctly
   - Then use larger wordlists

## Expected Output

```
╔════════════════════════════════════════════════╗
║       UART Login Bruteforce Tool v0.1         ║
╚════════════════════════════════════════════════╝

[*] Connected to /dev/ttyUSB0 at 115200 baud
[*] Learning failure pattern with obscure credentials...
[*] Waiting for username prompt...
[+] Detected username prompt, sending: OBSCURE_USER_XXXXXXXXX
[*] Waiting for password prompt...
[+] Detected password prompt, sending: OBSCURE_PASS_XXXXXXXXX
[+] Learned failure pattern:
    Login failed. Invalid credentials.

[*] Starting brute force attack...
[*] Username: admin

[*] Attempt 1: Trying password: admin
[+] Detected username prompt
[+] Detected password prompt
[x] Login failed
...
[*] Attempt 5: Trying password: admin123
[+] Detected username prompt
[+] Detected password prompt

[SUCCESS] Login successful!
[SUCCESS] Password found: admin123
[SUCCESS] Response: Welcome! You are now logged in as admin
```

## Troubleshooting

### Device Not Found
```bash
# Check what devices are available
ls -la /dev/ttyUSB*
ls -la /dev/ttyS*
```

### Wrong Prompts Detected
Use `screen` to manually verify:
```bash
screen /dev/ttyUSB0 115200
# Type username and observe exact response
# Copy the exact prompt string
```

### Timeout Issues
- Increase `--timeout` value
- Check baud rate matches device
- Verify UART cable connection

### Permission Issues
```bash
# Ensure running with sudo
sudo ./target/release/uart-bruteforce ...

# Or add user to dialout group (requires logout/login)
sudo usermod -a -G dialout $USER
```

## Security Best Practices

✅ **Do:**
- Use only on systems you own or have permission to test
- Document all testing activities
- Use unique username/password combinations
- Test in isolated environments first

❌ **Don't:**
- Use on systems without explicit authorization
- Leave the tool running unattended on production systems
- Use default credentials without testing
- Ignore system security policies

---

For more information, see the main [README.md](../README.md) in the project root.
