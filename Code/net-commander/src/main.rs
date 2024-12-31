use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, Write};
use std::net::ToSocketAddrs;
use std::process::Command;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

// ==============================
//        MAIN ENTRY POINT
// ==============================
#[tokio::main]
async fn main() {
    // Attempt to enable raw mode for a nicer experience in the terminal.
    if let Err(e) = enable_raw_mode() {
        eprintln!("Failed to enable raw mode: {}", e);
    }

    // Hide cursor for a cleaner UI
    let _ = execute!(io::stdout(), Hide);

    loop {
        // Draw the main menu
        draw_main_menu();

        // Read user input (keyboard). We do a small loop to handle non-key events.
        let choice = if let Ok(Event::Key(key_event)) = read() {
            if let KeyCode::Char(c) = key_event.code {
                c
            } else {
                // If it's some other key event (e.g., ArrowKey), just ignore
                // and loop again
                continue;
            }
        } else {
            // If we get a mouse event, resize event, etc., just ignore
            continue;
        };

        // Clear screen
        clear_screen();

        match choice {
            '1' => {
                ping_host_menu().await;
            }
            '2' => {
                dns_lookup_menu().await;
            }
            '3' => {
                port_scan_menu().await;
            }
            '4' => {
                ping_sweep_menu().await;
            }
            '5' => {
                list_network_interfaces();
                wait_for_keypress().await;
            }
            '6' => {
                subnet_scan_menu().await;
            }
            '7' => {
                detect_firewall_and_vpn();
                wait_for_keypress().await;
            }
            '8' => {
                latency_monitoring_menu().await;
            }
            '9' => {
                traceroute_menu().await;
            }
            'q' | 'Q' => {
                // Exit
                exit_app();
                return;
            }
            _ => {
                // Unknown choice
                print_styled_line("Unknown choice. Press any key to return.", Color::Red);
                wait_for_keypress().await;
            }
        }
    }
}

// ==============================
//         UI & HELPERS
// ==============================

/// Clears the screen using crossterm and moves cursor to (0,0).
fn clear_screen() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, Clear(ClearType::All), MoveTo(0, 0));
}

/// Prints a stylized line of text in a particular color, then resets color,
/// using `\r\n` for better cross-platform carriage-return + line-feed.
fn print_styled_line(text: &str, color: Color) {
    let _ = execute!(
        io::stdout(),
        SetForegroundColor(color),
        Print(format!("{}\r\n", text)),
        ResetColor
    );
}

/// Draws the main menu with crossterm styling.
fn draw_main_menu() {
    clear_screen();
    print_styled_line("=== net-commander ===", Color::Blue);

    // Replace println! with print! + "\r\n"
    print!("1) Ping a host\r\n");
    print!("2) DNS lookup\r\n");
    print!("3) Port scan\r\n");
    print!("4) Ping sweep\r\n");
    print!("5) List network interfaces\r\n");
    print!("6) Subnet scanning\r\n");
    print!("7) Firewall & VPN detection\r\n");
    print!("8) Latency monitoring (continuous ping)\r\n");
    print!("9) Traceroute\r\n");
    print!("Q) Quit\r\n");

    print_styled_line("Please enter a choice (1-9, Q to quit):", Color::Green);
    print!("> ");
    let _ = io::stdout().flush();
}

/// Prompts the user for input (line-based).
/// We disable raw mode momentarily to read line-based input more easily.
fn get_user_input(prompt: &str) -> String {
    // Turn off raw mode to allow normal line input
    let _ = disable_raw_mode();
    print_styled_line(prompt, Color::Green);
    print!("> ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Re-enable raw mode
    let _ = enable_raw_mode();

    input.trim().to_string()
}

/// Waits for a single keypress, discarding the result.
async fn wait_for_keypress() {
    loop {
        if let Ok(Event::Key(_)) = read() {
            break;
        }
    }
}

/// Gracefully exit app: show cursor, disable raw mode, clear screen.
fn exit_app() {
    let _ = execute!(io::stdout(), Show);
    let _ = disable_raw_mode();
    clear_screen();
    print_styled_line("Exiting net-commander. Goodbye!", Color::Blue);
}

// ============================
//     1) PING A SINGLE HOST
// ============================
async fn ping_host_menu() {
    let host = get_user_input("Enter host/IP to ping:");
    if host.is_empty() {
        print_styled_line("No host specified.", Color::Red);
        wait_for_keypress().await;
        return;
    }

    clear_screen();
    print_styled_line(&format!("Pinging {} ...", host), Color::Yellow);
    let output = Command::new("ping").args(get_ping_args(&host)).output();
    match output {
        Ok(o) => {
            if !o.stdout.is_empty() {
                // Replace println! with print! + "\r\n"
                print!("{}\r\n", String::from_utf8_lossy(&o.stdout));
            }
            if !o.stderr.is_empty() {
                eprint!("{}\r\n", String::from_utf8_lossy(&o.stderr));
            }
        }
        Err(e) => {
            print_styled_line(&format!("Failed to execute ping: {}", e), Color::Red);
        }
    }

    print_styled_line("Press any key to return to main menu...", Color::Green);
    wait_for_keypress().await;
}

/// Returns OS-specific ping arguments (4 pings).
/// This is simplistic; real usage might differ for each OS.
fn get_ping_args(host: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        vec!["-n".to_string(), "4".to_string(), host.to_string()]
    } else {
        vec!["-c".to_string(), "4".to_string(), host.to_string()]
    }
}

// ==========================
//     2) DNS LOOKUP
// ==========================
async fn dns_lookup_menu() {
    let host = get_user_input("Enter hostname for DNS lookup:");
    if host.is_empty() {
        print_styled_line("No hostname specified.", Color::Red);
        wait_for_keypress().await;
        return;
    }

    clear_screen();
    print_styled_line(&format!("Resolving DNS for {} ...", host), Color::Yellow);

    let socket_str = format!("{}:0", host);
    match socket_str.to_socket_addrs() {
        Ok(addrs) => {
            let v: Vec<_> = addrs.collect();
            if v.is_empty() {
                print!("No DNS records found for {}\r\n", host);
            } else {
                print!("Resolved addresses:\r\n");
                for (i, addr) in v.iter().enumerate() {
                    print!("  {}. {}\r\n", i + 1, addr);
                }
            }
        }
        Err(e) => {
            print_styled_line(&format!("DNS lookup error: {}", e), Color::Red);
        }
    }

    print_styled_line("Press any key to return to main menu...", Color::Green);
    wait_for_keypress().await;
}

// ==========================
//     3) PORT SCAN
// ==========================
async fn port_scan_menu() {
    let host = get_user_input("Enter host/IP to port-scan:");
    if host.is_empty() {
        print_styled_line("No host specified.", Color::Red);
        wait_for_keypress().await;
        return;
    }

    let start_port_str = get_user_input("Enter start port:");
    let end_port_str = get_user_input("Enter end port:");
    let start_port = start_port_str.parse().unwrap_or(1);
    let end_port = end_port_str.parse().unwrap_or(1024);

    clear_screen();
    print_styled_line(
        &format!(
            "Scanning TCP ports on {} from {} to {}...",
            host, start_port, end_port
        ),
        Color::Yellow,
    );

    let mut handles = Vec::new();
    for port in start_port..=end_port {
        let host_clone = host.clone();
        handles.push(tokio::spawn(
            async move { scan_port(&host_clone, port).await },
        ));
    }

    let mut open_ports = Vec::new();
    for handle in handles {
        if let Ok(Some(port)) = handle.await {
            open_ports.push(port);
        }
    }

    if open_ports.is_empty() {
        print!("No open TCP ports found in the specified range.\r\n");
    } else {
        print!("Open TCP ports: {:?}\r\n", open_ports);
    }

    print_styled_line("Press any key to return to main menu...", Color::Green);
    wait_for_keypress().await;
}

/// Attempt to connect to a (host, port). Returns `Some(port)` if open, else `None`.
async fn scan_port(host: &str, port: u16) -> Option<u16> {
    let addr = format!("{}:{}", host, port);
    match timeout(Duration::from_millis(500), TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => Some(port), // Connected => open
        _ => None,               // Timed out or error => closed/filtered
    }
}

// ==========================
//     4) PING SWEEP
// ==========================
async fn ping_sweep_menu() {
    let base_ip = get_user_input("Enter base IPv4 (e.g. 192.168.1):");
    if base_ip.is_empty() {
        print_styled_line("No base IP specified.", Color::Red);
        wait_for_keypress().await;
        return;
    }

    let start_id_str = get_user_input("Enter start host ID (e.g. 1):");
    let end_id_str = get_user_input("Enter end host ID (e.g. 10):");
    let start_id = start_id_str.parse().unwrap_or(1);
    let end_id = end_id_str.parse().unwrap_or(10);

    clear_screen();
    print_styled_line(
        &format!(
            "Performing ping sweep from {}.{} to {}.{}",
            base_ip, start_id, base_ip, end_id
        ),
        Color::Yellow,
    );

    let mut tasks = Vec::new();
    for id in start_id..=end_id {
        let ip_string = format!("{}.{}", base_ip, id);
        tasks.push(tokio::spawn(async move {
            if is_reachable(&ip_string).await {
                Some(ip_string)
            } else {
                None
            }
        }));
    }

    let mut reachable = Vec::new();
    for t in tasks {
        if let Ok(Some(ip)) = t.await {
            reachable.push(ip);
        }
    }

    if reachable.is_empty() {
        print!("No hosts responded to ping in that range.\r\n");
    } else {
        print!("Hosts responding to ping:\r\n");
        for ip in reachable {
            print!("  {}\r\n", ip);
        }
    }

    print_styled_line("Press any key to return to main menu...", Color::Green);
    wait_for_keypress().await;
}

/// Calls out to the system `ping` to check if host is reachable.
async fn is_reachable(ip: &str) -> bool {
    let output = Command::new("ping").args(get_ping_args(ip)).output();
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_lowercase();
            // naive check: "0% packet loss" often indicates success on Unix
            stdout.contains("0% packet loss") || stdout.contains(" no loss")
        }
        Err(_) => false,
    }
}

// ==============================
//     5) NETWORK INTERFACES
// ==============================
fn list_network_interfaces() {
    print_styled_line("Network Interfaces:", Color::Yellow);

    // Naive approach: shell out to ifconfig/ipconfig
    if cfg!(target_os = "windows") {
        let _ = Command::new("ipconfig").status();
    } else {
        // On most Unix systems:
        let _ = Command::new("ifconfig").status();
        // Or "ip addr show", depending on your environment
    }
}

// ==============================
//     6) SUBNET SCANNING
// ==============================
async fn subnet_scan_menu() {
    let cidr_input = get_user_input("Enter subnet in CIDR notation (e.g., 192.168.1.0/24):");
    if cidr_input.is_empty() {
        print_styled_line("No subnet specified.", Color::Red);
        wait_for_keypress().await;
        return;
    }

    clear_screen();
    print_styled_line(&format!("Subnet scanning {}", cidr_input), Color::Yellow);

    // Very naive approach: parse x.x.x.x/24, iterate .1 to .254
    let parts: Vec<&str> = cidr_input.split('/').collect();
    if parts.len() != 2 {
        print_styled_line("Invalid CIDR format.", Color::Red);
        wait_for_keypress().await;
        return;
    }
    let base_ip_str = parts[0];
    let cidr_bits: u8 = parts[1].parse().unwrap_or(24);

    if cidr_bits != 24 {
        print_styled_line("Only /24 subnets are supported in this demo.", Color::Red);
        wait_for_keypress().await;
        return;
    }

    let mut tasks = Vec::new();
    for i in 1..255 {
        let ip_string = increment_base_ip(base_ip_str, i);
        tasks.push(tokio::spawn(async move {
            if is_reachable(&ip_string).await {
                Some(ip_string)
            } else {
                None
            }
        }));
    }

    let mut reachable = Vec::new();
    for t in tasks {
        if let Ok(Some(ip)) = t.await {
            reachable.push(ip);
        }
    }

    if reachable.is_empty() {
        print!("No hosts responded to ping in that /24 subnet.\r\n");
    } else {
        print!(
            "Hosts responding to ping in {}/{}:\r\n",
            base_ip_str, cidr_bits
        );
        for ip in reachable {
            print!("  {}\r\n", ip);
        }
    }

    print_styled_line("Press any key to return to main menu...", Color::Green);
    wait_for_keypress().await;
}

/// Simplistic function to combine base IP and offset.
/// e.g. 192.168.1.0 + offset => 192.168.1.offset
fn increment_base_ip(base_ip: &str, offset: u8) -> String {
    let mut parts: Vec<u8> = base_ip.split('.').filter_map(|s| s.parse().ok()).collect();
    if parts.len() == 4 {
        parts[3] = offset;
        return format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3]);
    }
    base_ip.to_string()
}

// ==============================
//   7) FIREWALL & VPN DETECTION
// ==============================
fn detect_firewall_and_vpn() {
    print_styled_line("Detecting Firewall & VPN ...", Color::Yellow);

    if cfg!(target_os = "windows") {
        // Check Windows firewall status
        let firewall_status = Command::new("netsh")
            .args(["advfirewall", "show", "allprofiles"])
            .output();
        match firewall_status {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout).to_lowercase();
                if out.contains("state on") {
                    print!("Windows firewall appears to be ON.\r\n");
                } else if out.contains("state off") {
                    print!("Windows firewall appears to be OFF.\r\n");
                } else {
                    print!("Could not determine Windows firewall state.\r\n");
                }
            }
            Err(e) => {
                print!("Error checking Windows firewall: {}\r\n", e);
            }
        }

        // VPN detection: naive check for "ipconfig" lines containing "TUN" or "PPP"
        let vpn_check = Command::new("ipconfig").output();
        match vpn_check {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout).to_lowercase();
                if out.contains("tun") || out.contains("ppp") || out.contains("vpn") {
                    print!("A VPN interface might be active.\r\n");
                } else {
                    print!("No obvious VPN interface found.\r\n");
                }
            }
            Err(e) => {
                print!("Error checking VPN: {}\r\n", e);
            }
        }
    } else {
        // On Linux/macOS, check common firewall services or iptables
        let firewall_status = Command::new("systemctl")
            .args(["is-active", "firewalld"])
            .output();
        if let Ok(o) = firewall_status {
            let out = String::from_utf8_lossy(&o.stdout).to_lowercase();
            if out.contains("active") {
                print!("firewalld service is ACTIVE.\r\n");
            } else {
                print!("firewalld service is not active or not found.\r\n");
            }
        }

        // Check for iptables
        let iptables_check = Command::new("iptables").arg("-L").output();
        if let Ok(o) = iptables_check {
            let out = String::from_utf8_lossy(&o.stdout);
            print!("`iptables -L` returned:\r\n{}\r\n", out);
        }

        // VPN check: naive check for 'tun0' or 'ppp0' or 'wg0'
        let ifconfig_check = Command::new("ifconfig").output();
        if let Ok(o) = ifconfig_check {
            let out = String::from_utf8_lossy(&o.stdout).to_lowercase();
            if out.contains("tun0") || out.contains("ppp0") || out.contains("wg0") {
                print!("A VPN or tunneling interface might be active.\r\n");
            } else {
                print!("No obvious VPN interface found (tun0/ppp0/wg0).\r\n");
            }
        }
    }
}

// ==============================
//    8) LATENCY MONITORING
// ==============================
async fn latency_monitoring_menu() {
    let host = get_user_input("Enter host/IP for continuous ping:");
    if host.is_empty() {
        print_styled_line("No host specified.", Color::Red);
        wait_for_keypress().await;
        return;
    }

    clear_screen();
    print_styled_line(
        &format!("Latency monitoring for {} (press any key to stop)...", host),
        Color::Yellow,
    );
    print!("Pinging once per second, measuring round-trip time...\r\n");

    // We'll set up a loop that runs until a key is pressed.
    loop {
        let output = Command::new("ping")
            .args(get_latency_ping_args(&host))
            .output();

        match output {
            Ok(o) => {
                // Attempt to parse out round-trip time from the last line (naive).
                // We do a short 1-ping check each iteration.
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                if let Some(line) = out.lines().last() {
                    print!("{}\r\n", line);
                } else {
                    print!("{}\r\n", out);
                }
            }
            Err(e) => {
                print_styled_line(&format!("Ping error: {}", e), Color::Red);
            }
        }

        // Check if a key was pressed. If so, break out.
        if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
            if let Ok(Event::Key(_)) = read() {
                break;
            }
        }

        // Sleep for ~1 second between pings
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    print_styled_line(
        "Stopped. Press any key to return to main menu...",
        Color::Green,
    );
    wait_for_keypress().await;
}

/// OS-specific arguments for *one* ping, so we can measure repeated pings in a loop.
fn get_latency_ping_args(host: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        // -n 1 => one ping
        vec!["-n".to_string(), "1".to_string(), host.to_string()]
    } else {
        // -c 1 => one ping
        vec!["-c".to_string(), "1".to_string(), host.to_string()]
    }
}

// ==============================
//        9) TRACEROUTE
// ==============================
async fn traceroute_menu() {
    let host = get_user_input("Enter host for traceroute:");
    if host.is_empty() {
        print_styled_line("No host specified.", Color::Red);
        wait_for_keypress().await;
        return;
    }

    clear_screen();
    print_styled_line(
        &format!("Performing traceroute to {} ...", host),
        Color::Yellow,
    );

    // On Windows, the command is "tracert"; on Unix-like systems, "traceroute".
    if cfg!(target_os = "windows") {
        let output = Command::new("tracert").arg(host.clone()).output();
        match output {
            Ok(o) => {
                print!("{}\r\n", String::from_utf8_lossy(&o.stdout));
            }
            Err(e) => {
                print_styled_line(&format!("Failed to run tracert: {}", e), Color::Red);
            }
        }
    } else {
        let output = Command::new("traceroute").arg(host.clone()).output();
        match output {
            Ok(o) => {
                print!("{}\r\n", String::from_utf8_lossy(&o.stdout));
            }
            Err(e) => {
                print_styled_line(&format!("Failed to run traceroute: {}", e), Color::Red);
            }
        }
    }

    print_styled_line("Press any key to return to main menu...", Color::Green);
    wait_for_keypress().await;
}
