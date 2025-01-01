////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use std::{
    io::{self, Write},
    net::ToSocketAddrs,
    process::Command,
};

use anyhow::Result;
use clap::Parser;

// Crossterm
use crossterm::{
    cursor::MoveTo,
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

// TUI (tui-rs)
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

////////////////////////////////////////////////////////////////////////////////
// Cross-Platform Line Endings
////////////////////////////////////////////////////////////////////////////////

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";

#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

////////////////////////////////////////////////////////////////////////////////
// CLI Arguments
////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about = "NetCommander (TUI-based CLI)", long_about = None)]
struct CliArgs {
    /// Example verbose flag
    #[arg(long, short, help = "Enable verbose mode")]
    verbose: bool,
}

////////////////////////////////////////////////////////////////////////////////
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();
    if args.verbose {
        print!("Verbose mode enabled...{}", LINE_ENDING);
    }

    // 2) Enable raw mode for TUI and construct a CrosstermBackend
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // 3) Clear the screen and display a welcome “banner”
    clear_screen(&mut terminal)?;
    draw_welcome_banner(&mut terminal)?;

    print!("CLI started successfully!{}", LINE_ENDING);

    // 4) Run the main TUI loop (menu navigation, etc.)
    if let Err(e) = run_main_menu(&mut terminal).await {
        eprint!("Application error: {}{}", e, LINE_ENDING);
    }

    // 5) Before exiting, restore the terminal to normal mode and clear
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Initial Screen / Banner
////////////////////////////////////////////////////////////////////////////////

/// Clears the terminal screen for a clean start using TUI + crossterm.
fn clear_screen<B: tui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

/// Draws a basic “banner” (ASCII art / heading) via a `Paragraph`.
fn draw_welcome_banner<B: tui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.size();

        // Simple layout: just center a paragraph with the name
        let block = Block::default().borders(Borders::NONE);
        frame.render_widget(block, size);

        // Example ASCII art or simple banner text
        let banner_text = vec![Spans::from(Span::raw("Welcome to NetCommander!"))];

        let paragraph = Paragraph::new(banner_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, size);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI Application State & Main Menu
////////////////////////////////////////////////////////////////////////////////

/// Our application state for the main menu.
struct App {
    /// The index of the currently selected menu item.
    selected_index: usize,
    /// The list of menu items to display.
    menu_items: Vec<&'static str>,
}

impl App {
    fn new() -> Self {
        App {
            selected_index: 0,
            menu_items: vec![
                "1) Ping a host",
                "2) DNS lookup",
                "3) Port scan",
                "4) Ping sweep",
                "5) List network interfaces",
                "6) Subnet scanning",
                "7) Firewall & VPN detection",
                "8) Latency monitoring (continuous ping)",
                "9) Traceroute",
                "Q) Quit",
            ],
        }
    }

    /// Moves selection up (in a circular list).
    fn up(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = self.menu_items.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Moves selection down (in a circular list).
    fn down(&mut self) {
        self.selected_index = (self.selected_index + 1) % self.menu_items.len();
    }
}

////////////////////////////////////////////////////////////////////////////////
// Main Menu Loop
////////////////////////////////////////////////////////////////////////////////

/// Runs the main TUI loop, handling user input and rendering.
async fn run_main_menu<B: tui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new();

    loop {
        // Draw the UI
        terminal.draw(|frame| {
            // Split the screen into top (title) and bottom (menu) sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                .split(frame.size());

            // Top section: display the application title
            let title_paragraph = Paragraph::new("net-commander")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::NONE));
            frame.render_widget(title_paragraph, chunks[0]);

            // Bottom section: main menu items
            let items: Vec<ListItem> = app
                .menu_items
                .iter()
                .enumerate()
                .map(|(i, &m)| {
                    // Highlight the selected item
                    if i == app.selected_index {
                        ListItem::new(Span::styled(
                            m,
                            Style::default()
                                .fg(Color::White)
                                .bg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else {
                        ListItem::new(Span::raw(m))
                    }
                })
                .collect();

            let list =
                List::new(items).block(Block::default().borders(Borders::ALL).title("Main Menu"));
            frame.render_widget(list, chunks[1]);
        })?;

        // Handle user inputs (non-blocking)
        if crossterm::event::poll(Duration::from_millis(200))? {
            match event::read()? {
                CEvent::Key(key_event) => match key_event.code {
                    KeyCode::Up => {
                        app.up();
                    }
                    KeyCode::Down => {
                        app.down();
                    }
                    KeyCode::Enter => {
                        // Execute based on selected menu item
                        let choice = match app.selected_index {
                            0 => '1',
                            1 => '2',
                            2 => '3',
                            3 => '4',
                            4 => '5',
                            5 => '6',
                            6 => '7',
                            7 => '8',
                            8 => '9',
                            9 => 'q',
                            _ => '?',
                        };
                        if !handle_menu_choice(choice).await? {
                            // If false returned => user wants to quit
                            return Ok(());
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        // Quit
                        return Ok(());
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Handle Menu Choices
////////////////////////////////////////////////////////////////////////////////

/// Returns Ok(true) if continuing, Ok(false) if user chose to quit.
async fn handle_menu_choice(choice: char) -> Result<bool> {
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
            return Ok(false);
        }
        _ => {
            // Unknown choice
            print!("Unknown choice. Press any key to return.{}", LINE_ENDING);
            wait_for_keypress().await;
        }
    }
    Ok(true)
}

////////////////////////////////////////////////////////////////////////////////
// Utility Functions
////////////////////////////////////////////////////////////////////////////////

/// Waits for a single keypress (discarding the result).
pub async fn wait_for_keypress() {
    loop {
        if let Ok(CEvent::Key(_)) = event::read() {
            break;
        }
    }
}

/// Gracefully exit app: show a goodbye message, etc.
fn exit_app() {
    print!("Exiting net-commander. Goodbye!{}", LINE_ENDING);
}

////////////////////////////////////////////////////////////////////////////////
// Sub-Menu & Network Functions
////////////////////////////////////////////////////////////////////////////////

async fn ping_host_menu() {
    let host = get_user_input("Enter host/IP to ping:");
    if host.is_empty() {
        print!("No host specified.{}", LINE_ENDING);
        wait_for_keypress().await;
        return;
    }

    print!("Pinging {} ...{}", host, LINE_ENDING);
    let output = Command::new("ping").args(get_ping_args(&host)).output();
    match output {
        Ok(o) => {
            if !o.stdout.is_empty() {
                print!("{}{}", String::from_utf8_lossy(&o.stdout), LINE_ENDING);
            }
            if !o.stderr.is_empty() {
                eprint!("{}{}", String::from_utf8_lossy(&o.stderr), LINE_ENDING);
            }
        }
        Err(e) => {
            print!("Failed to execute ping: {}{}", e, LINE_ENDING);
        }
    }

    print!("Press any key to return to main menu...{}", LINE_ENDING);
    wait_for_keypress().await;
}

/// Returns OS-specific ping arguments (4 pings).
fn get_ping_args(host: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        vec!["-n".to_string(), "4".to_string(), host.to_string()]
    } else {
        vec!["-c".to_string(), "4".to_string(), host.to_string()]
    }
}

async fn dns_lookup_menu() {
    let host = get_user_input("Enter hostname for DNS lookup:");
    if host.is_empty() {
        print!("No hostname specified.{}", LINE_ENDING);
        wait_for_keypress().await;
        return;
    }

    print!("Resolving DNS for {} ...{}", host, LINE_ENDING);
    let socket_str = format!("{}:0", host);
    match socket_str.to_socket_addrs() {
        Ok(addrs) => {
            let v: Vec<_> = addrs.collect();
            if v.is_empty() {
                print!("No DNS records found for {}{}", host, LINE_ENDING);
            } else {
                print!("Resolved addresses:{}", LINE_ENDING);
                for (i, addr) in v.iter().enumerate() {
                    print!("  {}. {}{}", i + 1, addr, LINE_ENDING);
                }
            }
        }
        Err(e) => {
            print!("DNS lookup error: {}{}", e, LINE_ENDING);
        }
    }

    print!("Press any key to return to main menu...{}", LINE_ENDING);
    wait_for_keypress().await;
}

async fn port_scan_menu() {
    let host = get_user_input("Enter host/IP to port-scan:");
    if host.is_empty() {
        print!("No host specified.{}", LINE_ENDING);
        wait_for_keypress().await;
        return;
    }

    let start_port_str = get_user_input("Enter start port:");
    let end_port_str = get_user_input("Enter end port:");
    let start_port = start_port_str.parse().unwrap_or(1);
    let end_port = end_port_str.parse().unwrap_or(1024);

    print!(
        "Scanning TCP ports on {} from {} to {}...{}",
        host, start_port, end_port, LINE_ENDING
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
        print!(
            "No open TCP ports found in the specified range.{}",
            LINE_ENDING
        );
    } else {
        print!("Open TCP ports: {:?}{}", open_ports, LINE_ENDING);
    }

    print!("Press any key to return to main menu...{}", LINE_ENDING);
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

async fn ping_sweep_menu() {
    let base_ip = get_user_input("Enter base IPv4 (e.g. 192.168.1):");
    if base_ip.is_empty() {
        print!("No base IP specified.{}", LINE_ENDING);
        wait_for_keypress().await;
        return;
    }

    let start_id_str = get_user_input("Enter start host ID (e.g. 1):");
    let end_id_str = get_user_input("Enter end host ID (e.g. 10):");
    let start_id = start_id_str.parse().unwrap_or(1);
    let end_id = end_id_str.parse().unwrap_or(10);

    print!(
        "Performing ping sweep from {}.{} to {}.{}{}",
        base_ip, start_id, base_ip, end_id, LINE_ENDING
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
        print!("No hosts responded to ping in that range.{}", LINE_ENDING);
    } else {
        print!("Hosts responding to ping:{}", LINE_ENDING);
        for ip in reachable {
            print!("  {}{}", ip, LINE_ENDING);
        }
    }

    print!("Press any key to return to main menu...{}", LINE_ENDING);
    wait_for_keypress().await;
}

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

fn list_network_interfaces() {
    print!("Network Interfaces:{}", LINE_ENDING);
    // Naive approach: shell out to ifconfig/ipconfig
    if cfg!(target_os = "windows") {
        let _ = Command::new("ipconfig").status();
    } else {
        let _ = Command::new("ifconfig").status();
        // Or "ip addr show"
    }
}

async fn subnet_scan_menu() {
    let cidr_input = get_user_input("Enter subnet in CIDR notation (e.g., 192.168.1.0/24):");
    if cidr_input.is_empty() {
        print!("No subnet specified.{}", LINE_ENDING);
        wait_for_keypress().await;
        return;
    }

    print!("Subnet scanning {}{}", cidr_input, LINE_ENDING);

    let parts: Vec<&str> = cidr_input.split('/').collect();
    if parts.len() != 2 {
        print!("Invalid CIDR format.{}", LINE_ENDING);
        wait_for_keypress().await;
        return;
    }

    let base_ip_str = parts[0];
    let cidr_bits: u8 = parts[1].parse().unwrap_or(24);

    if cidr_bits != 24 {
        print!(
            "Only /24 subnets are supported in this demo.{}",
            LINE_ENDING
        );
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
        print!(
            "No hosts responded to ping in that /24 subnet.{}",
            LINE_ENDING
        );
    } else {
        print!(
            "Hosts responding to ping in {}/{}:{}",
            base_ip_str, cidr_bits, LINE_ENDING
        );
        for ip in reachable {
            print!("  {}{}", ip, LINE_ENDING);
        }
    }

    print!("Press any key to return to main menu...{}", LINE_ENDING);
    wait_for_keypress().await;
}

fn increment_base_ip(base_ip: &str, offset: u8) -> String {
    let mut parts: Vec<u8> = base_ip.split('.').filter_map(|s| s.parse().ok()).collect();
    if parts.len() == 4 {
        parts[3] = offset;
        return format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3]);
    }
    base_ip.to_string()
}

fn detect_firewall_and_vpn() {
    print!("Detecting Firewall & VPN ...{}", LINE_ENDING);

    if cfg!(target_os = "windows") {
        let firewall_status = Command::new("netsh")
            .args(["advfirewall", "show", "allprofiles"])
            .output();
        match firewall_status {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout).to_lowercase();
                if out.contains("state on") {
                    print!("Windows firewall appears to be ON.{}", LINE_ENDING);
                } else if out.contains("state off") {
                    print!("Windows firewall appears to be OFF.{}", LINE_ENDING);
                } else {
                    print!("Could not determine Windows firewall state.{}", LINE_ENDING);
                }
            }
            Err(e) => {
                print!("Error checking Windows firewall: {}{}", e, LINE_ENDING);
            }
        }

        let vpn_check = Command::new("ipconfig").output();
        match vpn_check {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout).to_lowercase();
                if out.contains("tun") || out.contains("ppp") || out.contains("vpn") {
                    print!("A VPN interface might be active.{}", LINE_ENDING);
                } else {
                    print!("No obvious VPN interface found.{}", LINE_ENDING);
                }
            }
            Err(e) => {
                print!("Error checking VPN: {}{}", e, LINE_ENDING);
            }
        }
    } else {
        let firewall_status = Command::new("systemctl")
            .args(["is-active", "firewalld"])
            .output();
        if let Ok(o) = firewall_status {
            let out = String::from_utf8_lossy(&o.stdout).to_lowercase();
            if out.contains("active") {
                print!("firewalld service is ACTIVE.{}", LINE_ENDING);
            } else {
                print!(
                    "firewalld service is not active or not found.{}",
                    LINE_ENDING
                );
            }
        }

        let iptables_check = Command::new("iptables").arg("-L").output();
        if let Ok(o) = iptables_check {
            let out = String::from_utf8_lossy(&o.stdout);
            print!(
                "`iptables -L` returned:{}{}{}",
                LINE_ENDING, out, LINE_ENDING
            );
        }

        let ifconfig_check = Command::new("ifconfig").output();
        if let Ok(o) = ifconfig_check {
            let out = String::from_utf8_lossy(&o.stdout).to_lowercase();
            if out.contains("tun0") || out.contains("ppp0") || out.contains("wg0") {
                print!(
                    "A VPN or tunneling interface might be active.{}",
                    LINE_ENDING
                );
            } else {
                print!(
                    "No obvious VPN interface found (tun0/ppp0/wg0).{}",
                    LINE_ENDING
                );
            }
        }
    }
}

async fn latency_monitoring_menu() {
    let host = get_user_input("Enter host/IP for continuous ping:");
    if host.is_empty() {
        print!("No host specified.{}", LINE_ENDING);
        wait_for_keypress().await;
        return;
    }

    print!(
        "Latency monitoring for {} (press any key to stop)...{}",
        host, LINE_ENDING
    );
    print!("Pinging once per second...{}", LINE_ENDING);

    loop {
        let output = Command::new("ping")
            .args(get_latency_ping_args(&host))
            .output();

        match output {
            Ok(o) => {
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                if let Some(line) = out.lines().last() {
                    print!("{}{}", line, LINE_ENDING);
                } else {
                    print!("{}{}", out, LINE_ENDING);
                }
            }
            Err(e) => {
                print!("Ping error: {}{}", e, LINE_ENDING);
            }
        }

        // Check if a key was pressed. If so, break out.
        if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
            if let Ok(CEvent::Key(_)) = event::read() {
                break;
            }
        }
        // Sleep for ~1 second between pings
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    print!(
        "Stopped. Press any key to return to main menu...{}",
        LINE_ENDING
    );
    wait_for_keypress().await;
}

fn get_latency_ping_args(host: &str) -> Vec<String> {
    if cfg!(target_os = "windows") {
        // -n 1 => one ping
        vec!["-n".to_string(), "1".to_string(), host.to_string()]
    } else {
        // -c 1 => one ping
        vec!["-c".to_string(), "1".to_string(), host.to_string()]
    }
}

async fn traceroute_menu() {
    let host = get_user_input("Enter host for traceroute:");
    if host.is_empty() {
        print!("No host specified.{}", LINE_ENDING);
        wait_for_keypress().await;
        return;
    }

    print!("Performing traceroute to {} ...{}", host, LINE_ENDING);

    if cfg!(target_os = "windows") {
        let output = Command::new("tracert").arg(host.clone()).output();
        match output {
            Ok(o) => {
                print!("{}{}", String::from_utf8_lossy(&o.stdout), LINE_ENDING);
            }
            Err(e) => {
                print!("Failed to run tracert: {}{}", e, LINE_ENDING);
            }
        }
    } else {
        let output = Command::new("traceroute").arg(host.clone()).output();
        match output {
            Ok(o) => {
                print!("{}{}", String::from_utf8_lossy(&o.stdout), LINE_ENDING);
            }
            Err(e) => {
                print!("Failed to run traceroute: {}{}", e, LINE_ENDING);
            }
        }
    }

    print!("Press any key to return to main menu...{}", LINE_ENDING);
    wait_for_keypress().await;
}

////////////////////////////////////////////////////////////////////////////////
// get_user_input Helper
////////////////////////////////////////////////////////////////////////////////

/// Prompts the user for input in "cooked" mode (temporarily disabling raw mode).
fn get_user_input(prompt: &str) -> String {
    // Disable raw mode so we can do normal line-based input
    let _ = disable_raw_mode();

    print!("{}{}", prompt, LINE_ENDING);
    print!("> ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // Re-enable raw mode for TUI
    let _ = enable_raw_mode();

    input.trim().to_string()
}
