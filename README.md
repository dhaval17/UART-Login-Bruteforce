# UART Login Bruteforce Tool

A high-performance Rust tool for automated brute-force login attacks over UART serial connections. Designed for authorized security testing on embedded systems and IoT devices.

## Features

- Sudo Permission Verification - Ensures root access for serial port operations
- Flexible UART Configuration - Customizable device path, baud rate, and timeouts
- Smart Prompt Detection - Regex and literal string matching for username/password prompts
- Intelligent Failure Learning - Uses obscure credentials to learn what a failed login looks like
- Wordlist Support - Reads passwords from file (one per line)
- Real-time Feedback - Shows attempt count, successes, and failures
- Success Detection - Analyzes responses to identify successful authentication
- Error Recovery - Handles timeouts and reconnection attempts
- Verbose Debugging - Optional detailed output for troubleshooting

## Requirements

- **OS**: Linux (for UART access via /dev/ttyUSB* or /dev/ttyS*)
- **Rust**: 1.70+ (for building)
- **Root Access**: Required for serial port operations
- **Dependencies**:
  - `serialport` - Serial port communication
  - `clap` - CLI argument parsing
  - `regex` - Pattern matching
  - `libc` - Sudo permission checking

## Device Specific Branches

- DLink-DSL-224: https://github.com/dhaval17/UART-Login-Bruteforce/tree/DLink-DSL-224

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/dhaval17/UART-Login-Bruteforce.git
cd UART-Login-Bruteforce

# Build the release binary
cargo build --release

# The binary will be at: target/release/uart-bruteforce
```

### Quick Start

```bash
# Make sure you have root access
sudo ./target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 115200 \
    --username-prompt "Username:" \
    --password-prompt "Password:" \
    --username admin \
    --passwords wordlist.txt
```

## Usage

### Basic Usage

```bash
sudo ./target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 115200 \
    --username-prompt "login:" \
    --password-prompt "password:" \
    --username root \
    --passwords passwords.txt
```

### Command-Line Arguments

| Argument | Short | Default | Description |
|----------|-------|---------|-------------|
| `--device` | `-d` | `/dev/ttyUSB0` | Serial device path |
| `--baudrate` | `-b` | `115200` | Serial port baud rate |
| `--username-prompt` | | *required* | String/regex to detect username prompt |
| `--password-prompt` | | *required* | String/regex to detect password prompt |
| `--username` | `-u` | *required* | Username for login attempts |
| `--passwords` | `-p` | *required* | Path to password wordlist file |
| `--timeout` | | `3000` | Read timeout in milliseconds |
| `--verbose` | `-v` | `false` | Enable verbose debug output |

### Examples

#### Linux System with Standard Prompts
```bash
sudo ./target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 115200 \
    --username-prompt "login:" \
    --password-prompt "Password:" \
    --username admin \
    --passwords wordlist.txt
```

#### Embedded System with Custom Prompts
```bash
sudo ./target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 9600 \
    --username-prompt "Enter User ID:" \
    --password-prompt "Enter PIN:" \
    --username operator \
    --passwords passwords.txt \
    --timeout 5000
```

#### With Verbose Debug Output
```bash
sudo ./target/release/uart-bruteforce \
    -d /dev/ttyUSB0 \
    -b 115200 \
    --username-prompt "login:" \
    --password-prompt "password:" \
    -u admin \
    -p wordlist.txt \
    --verbose
```

## How It Works

### Phase 1: Connection & Learning
1. Connects to specified serial device with given baud rate
2. Generates obscure username/password credentials
3. Waits for username prompt and sends obscure username
4. Waits for password prompt and sends obscure password
5. Captures and learns what a "failed login" response looks like

### Phase 2: Brute Force Attack
1. For each password in wordlist:
   - Waits for username prompt
   - Sends configured username
   - Waits for password prompt
   - Sends current password attempt
   - Analyzes response for success/failure
   - Reports results

### Phase 3: Success Detection
- Looks for absence of common failure indicators (invalid, failed, incorrect, etc.)
- Looks for presence of success indicators (welcome, logged in, shell, etc.)
- Checks for shell prompts ($, #, >)
- Reports successful credentials and exits

## Troubleshooting

### "Permission Denied" on Serial Port
```bash
# Ensure running with sudo
sudo ./target/release/uart-bruteforce ...

# Or add user to dialout group (requires logout/login)
sudo usermod -a -G dialout $USER
```

### Device Not Found
```bash
# List available serial devices
ls -la /dev/ttyUSB*
ls -la /dev/ttyS*

# Check device info
dmesg | grep -i usb
```

### Wrong Prompts Not Detected
```bash
# Use screen to identify exact prompts
screen /dev/ttyUSB0 115200

# Note the exact text (including spaces)
# Press Ctrl+A, then 'quit' to exit screen

# Run tool with verbose output
sudo ./target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 115200 \
    --username-prompt "exact_prompt_text" \
    --password-prompt "exact_password_text" \
    --username admin \
    --passwords wordlist.txt \
    --verbose
```

### Timeout Issues
- Increase `--timeout` value (in milliseconds)
- Verify baud rate matches device configuration
- Check UART cable connection quality
- Try slower baud rates (9600, 19200)

### No Passwords Working
1. Verify username/password prompts are correct
2. Test with verbose mode to see raw responses
3. Ensure password file is readable and not empty
4. Try connecting manually with `screen` to test credentials

## Password Wordlist Format

Password file should contain one password per line:

```
password1
password2
password3
admin123
letmein
```

Create your own wordlist:
```bash
# From online sources
wget https://raw.githubusercontent.com/danielmiessler/SecLists/master/Passwords/Common-Credentials/10-million-password-list-top-100.txt -O wordlist.txt

# Filter by length
awk 'length($0) >= 6 && length($0) <= 12' raw_wordlist.txt > filtered_wordlist.txt

# Create custom list
cat > my_passwords.txt << EOF
admin
password
letmein
EOF
```

## Performance Tuning

### For Fast Devices
```bash
--timeout 1000  # 1 second
--baudrate 115200
```

### For Slow Embedded Systems
```bash
--timeout 10000  # 10 seconds
--baudrate 9600
```

### Optimal Settings Discovery
1. Connect with `screen` to determine response time
2. Add 1-2 seconds buffer to timeout
3. Verify baud rate with device documentation
4. Test with 5-10 passwords first

## Security Considerations

### Best Practices
- Only use on systems you own or have explicit permission to test
- Document all testing activities
- Use unique username/password combinations
- Test in isolated environments first
- Follow local laws and regulations

### Illegal Usage
- Do NOT use without authorization
- Do NOT target production systems
- Do NOT leave running on unattended systems
- Do NOT ignore security policies

## Example Output

```
╔════════════════════════════════════════════════╗
║       UART Login Bruteforce Tool v0.1         ║
╚════════════════════════════════════════════════╝

[*] Loaded 25 passwords from wordlist.txt
[*] Connected to /dev/ttyUSB0 at 115200 baud

[*] Learning failure pattern with obscure credentials...
[*] Waiting for username prompt...
[+] Detected username prompt, sending: OBSCURE_USER_XXXXXXXXXX
[*] Waiting for password prompt...
[+] Detected password prompt, sending: OBSCURE_PASS_XXXXXXXXXX
[+] Learned failure pattern:
    Login failed. Invalid credentials.

[*] Starting brute force attack...
[*] Username: admin

[*] Attempt 1: Trying password: admin
[+] Detected username prompt
[+] Detected password prompt
[x] Login failed

[*] Attempt 2: Trying password: password
[+] Detected username prompt
[+] Detected password prompt
[x] Login failed

[*] Attempt 5: Trying password: admin123
[+] Detected username prompt
[+] Detected password prompt

[SUCCESS] Login successful!
[SUCCESS] Password found: admin123
[SUCCESS] Response: Welcome! You are now logged in as admin
```

## Debugging

Enable verbose output for detailed debugging:

```bash
sudo ./target/release/uart-bruteforce \
    --device /dev/ttyUSB0 \
    --baudrate 115200 \
    --username-prompt "login:" \
    --password-prompt "password:" \
    --username admin \
    --passwords wordlist.txt \
    --verbose
```

Output will show:
- All serial data received `[RX]`
- All data transmitted `[TX]`
- Prompt detection events
- Response parsing details

## Additional Resources

- [Rust serialport crate](https://docs.rs/serialport/)
- [Clap CLI framework](https://docs.rs/clap/)
- [Linux serial port programming](https://tldp.org/HOWTO/Serial-Programming-HOWTO/)
- [UART Communication Guide](https://en.wikipedia.org/wiki/Universal_asynchronous_receiver-transmitter)

## Contributing

Contributions are welcome! Please feel free to:
- Report bugs
- Suggest improvements
- Submit pull requests
- Improve documentation

## License

This project is provided as-is for educational and authorized security testing purposes. Users are responsible for ensuring they have permission to test any systems they target.

## Disclaimer

This tool is intended for authorized security testing only. Unauthorized access to computer systems is illegal. The author assumes no liability for misuse or damage caused by this tool.

---

**Last Updated**: 2026-05-10  
**Version**: 0.1.0  
**Author**: Dhaval Chauhan ([@dhaval17](https://github.com/dhaval17))
