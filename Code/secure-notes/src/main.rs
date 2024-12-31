use anyhow::{anyhow, Result};
use crossterm::{
    cursor::MoveTo,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent,
        MouseEventKind,
    },
    execute,
    style::{Print, Stylize}, // removed unused `Color`
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetTitle,
    },
};
use ring::{aead, pbkdf2, rand as ring_rand};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    num::NonZeroU32,
    path::Path,
    process,
};
use zeroize::Zeroize;

const SALT: &[u8] = b"fixed-salt-demo"; // For demonstration only—use a unique salt in production!
const PBKDF2_ITERATIONS: u32 = 100_000;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Note {
    id: usize,
    title: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    clear_screen()?;
    print_welcome_banner()?;

    let mut password = prompt_user_input("Enter master password: ")?;
    let key = derive_key_from_password(&password, SALT, PBKDF2_ITERATIONS)?;
    password.zeroize();

    let mut notes = load_notes("secure_notes.json.enc", &key).unwrap_or_default();

    loop {
        print!("\r\n");
        print_menu()?;

        let choice = prompt_user_input("Choose an option [1-5]: ")?;
        match choice.trim() {
            "1" => {
                view_notes(&notes)?;
            }
            "2" => {
                let new_note = create_note()?;
                notes.push(new_note);
                cprintln("Note created.")?;
                save_notes("secure_notes.json.enc", &notes, &key)?;
            }
            "3" => {
                edit_note(&mut notes)?;
                save_notes("secure_notes.json.enc", &notes, &key)?;
            }
            "4" => {
                delete_note(&mut notes)?;
                save_notes("secure_notes.json.enc", &notes, &key)?;
            }
            "5" => {
                cprintln("Goodbye!")?;
                process::exit(0);
            }
            _ => {
                cprintln("Invalid choice.")?;
            }
        }
    }
}

fn clear_screen() -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

fn print_welcome_banner() -> Result<()> {
    let banner = r#"
                                                    _
                                                   | |
 ___   ___   ___  _   _  _ __   ___   _ __    ___  | |_   ___  ___
/ __| / _ \ / __|| | | || '__| / _ \ | '_ \  / _ \ | __| / _ \/ __|
\__ \|  __/| (__ | |_| || |   |  __/ | | | || (_) || |_ |  __/\__ \
|___/ \___| \___| \__,_||_|    \___| |_| |_| \___/  \__| \___||___/
"#;

    let colored_banner = banner.magenta();
    cprintln(&colored_banner.to_string())?;

    let mut stdout = io::stdout();
    execute!(stdout, SetTitle("Secure Notes - Rust CLI"))?;
    Ok(())
}

/// A helper to print with "\r\n" then flush.
fn cprintln(content: &str) -> Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "{}\r\n", content)?;
    stdout.flush()?;
    Ok(())
}

fn prompt_user_input(prompt_msg: &str) -> Result<String> {
    print!("{}\r\n> ", prompt_msg);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim_end().to_string())
}

fn derive_key_from_password(password: &str, salt: &[u8], iterations: u32) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(iterations).unwrap(),
        salt,
        password.as_bytes(),
        &mut key,
    );
    Ok(key)
}

fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let sealing_key = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|_| anyhow!("Failed to create encryption key"))?,
    );

    let rng = ring_rand::SystemRandom::new();
    let nonce_bytes =
        ring_rand::generate::<[u8; 12]>(&rng).map_err(|_| anyhow!("Failed to generate nonce"))?;
    let nonce_array = nonce_bytes.expose();
    let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

    let mut in_out = plaintext.to_vec();
    in_out.resize(in_out.len() + sealing_key.algorithm().tag_len(), 0);

    sealing_key
        .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| anyhow!("Encryption failed"))?;

    let mut result = Vec::with_capacity(12 + in_out.len());
    result.extend_from_slice(&nonce_array);
    result.extend_from_slice(&in_out);
    Ok(result)
}

fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(anyhow!("Ciphertext too short"));
    }
    let (nonce_bytes, encrypted) = ciphertext.split_at(12);
    let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| anyhow!("Invalid nonce"))?;

    let opening_key = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|_| anyhow!("Failed to create decryption key"))?,
    );

    let mut in_out = encrypted.to_vec();
    let decrypted_data = opening_key
        .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| anyhow!("Decryption failed"))?;
    Ok(decrypted_data.to_vec())
}

fn load_notes<P: AsRef<Path>>(path: P, key: &[u8]) -> Result<Vec<Note>> {
    if !path.as_ref().exists() {
        return Err(anyhow!("No existing notes file found"));
    }
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;
    let decrypted_bytes = decrypt_data(&ciphertext, key)?;
    let notes: Vec<Note> = serde_json::from_slice(&decrypted_bytes)?;
    Ok(notes)
}

fn save_notes<P: AsRef<Path>>(path: P, notes: &[Note], key: &[u8]) -> Result<()> {
    let json_data = serde_json::to_vec(notes)?;
    let ciphertext = encrypt_data(&json_data, key)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(&ciphertext)?;
    file.flush()?;
    Ok(())
}

fn print_menu() -> Result<()> {
    cprintln("===== Secure Notes Menu =====")?;
    cprintln("1) View Notes")?;
    cprintln("2) Create Note")?;
    cprintln("3) Edit Note")?;
    cprintln("4) Delete Note")?;
    cprintln("5) Exit")?;
    Ok(())
}

fn view_notes(notes: &[Note]) -> Result<()> {
    if notes.is_empty() {
        cprintln("No notes found.")?;
        return Ok(());
    }
    for n in notes {
        // Use `.as_str()` so we only borrow the title, preventing move errors
        cprintln(&format!(
            "ID: {} | Title: {} | Content (truncated): {}",
            n.id,
            n.title.as_str().blue().bold(),
            n.content.chars().take(30).collect::<String>()
        ))?;
    }
    Ok(())
}

fn create_note() -> Result<Note> {
    let title = prompt_user_input("Enter note title: ")?;
    let content = launch_text_editor(
        "",
        "Type your note, press [Esc] to finish. Click or arrow-key to move cursor.",
    )?;
    let note_id = rand::random::<usize>();
    Ok(Note {
        id: note_id,
        title,
        content,
    })
}

fn edit_note(notes: &mut [Note]) -> Result<()> {
    if notes.is_empty() {
        cprintln("No notes to edit.")?;
        return Ok(());
    }
    let id_str = prompt_user_input("Enter note ID to edit: ")?;
    let id = id_str
        .trim()
        .parse::<usize>()
        .map_err(|_| anyhow!("Invalid ID"))?;
    if let Some(n) = notes.iter_mut().find(|n| n.id == id) {
        cprintln(&format!("Editing note with ID: {}", n.id))?;
        cprintln(&format!("Existing content:\r\n{}\r\n", n.content))?;
        let new_content = launch_text_editor(
            &n.content,
            "Modify content, press [Esc] to finish. Click or arrow-key to move cursor.",
        )?;
        n.content = new_content;
        cprintln("Note updated.")?;
    } else {
        cprintln("Note not found.")?;
    }
    Ok(())
}

fn delete_note(notes: &mut Vec<Note>) -> Result<()> {
    if notes.is_empty() {
        cprintln("No notes to delete.")?;
        return Ok(());
    }
    let id_str = prompt_user_input("Enter note ID to delete: ")?;
    let id = id_str
        .trim()
        .parse::<usize>()
        .map_err(|_| anyhow!("Invalid ID"))?;
    let old_len = notes.len();
    notes.retain(|n| n.id != id);
    if notes.len() < old_len {
        cprintln("Note deleted.")?;
    } else {
        cprintln("No note found with that ID.")?;
    }
    Ok(())
}

/// More advanced raw-mode text editor that supports:
/// - Multi-line input
/// - Arrow key navigation
/// - Backspace merging lines
/// - **Mouse click** to reposition cursor
/// - Exits on [Esc]
fn launch_text_editor(initial_text: &str, prompt_msg: &str) -> Result<String> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;

    cprintln(prompt_msg)?;
    cprintln("Press [Esc] to finish. Enter inserts a new line.\r\n")?;

    let mut lines: Vec<String> = if initial_text.is_empty() {
        vec![String::new()]
    } else {
        // Split the initial text by CRLF or LF
        initial_text
            .replace("\r\n", "\n")
            .split('\n')
            .map(|s| s.to_string())
            .collect()
    };

    let mut cursor_row = 0;
    let mut cursor_col = 0;
    let mut redraw = true;

    'editor_loop: loop {
        if redraw {
            execute!(stdout, Clear(ClearType::All), MoveTo(0, 2))?;
            for (row_i, line) in lines.iter().enumerate() {
                execute!(stdout, MoveTo(0, (row_i + 2) as u16))?;
                execute!(stdout, Print(line))?;
            }
            execute!(stdout, MoveTo(cursor_col as u16, (cursor_row + 2) as u16))?;
            stdout.flush()?;
            redraw = false;
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => match code {
                    KeyCode::Esc => {
                        break 'editor_loop;
                    }
                    KeyCode::Enter => {
                        let remainder = lines[cursor_row].split_off(cursor_col);
                        lines.insert(cursor_row + 1, remainder);
                        cursor_row += 1;
                        cursor_col = 0;
                        redraw = true;
                    }
                    KeyCode::Backspace => {
                        if cursor_col > 0 {
                            cursor_col -= 1;
                            lines[cursor_row].remove(cursor_col);
                        } else if cursor_row > 0 {
                            // Merge with previous line
                            let prev_len = lines[cursor_row - 1].len();
                            let current_line = lines.remove(cursor_row);
                            cursor_row -= 1;
                            cursor_col = prev_len;
                            lines[cursor_row].push_str(&current_line);
                        }
                        redraw = true;
                    }
                    KeyCode::Left => {
                        if cursor_col > 0 {
                            cursor_col -= 1;
                        } else if cursor_row > 0 {
                            cursor_row -= 1;
                            cursor_col = lines[cursor_row].len();
                        }
                        redraw = true;
                    }
                    KeyCode::Right => {
                        if cursor_col < lines[cursor_row].len() {
                            cursor_col += 1;
                        } else if cursor_row + 1 < lines.len() {
                            cursor_row += 1;
                            cursor_col = 0;
                        }
                        redraw = true;
                    }
                    KeyCode::Up => {
                        if cursor_row > 0 {
                            cursor_row -= 1;
                            if cursor_col > lines[cursor_row].len() {
                                cursor_col = lines[cursor_row].len();
                            }
                        }
                        redraw = true;
                    }
                    KeyCode::Down => {
                        if cursor_row + 1 < lines.len() {
                            cursor_row += 1;
                            if cursor_col > lines[cursor_row].len() {
                                cursor_col = lines[cursor_row].len();
                            }
                        }
                        redraw = true;
                    }
                    KeyCode::Char(ch) => {
                        lines[cursor_row].insert(cursor_col, ch);
                        cursor_col += 1;
                        redraw = true;
                    }
                    _ => {}
                },
                Event::Mouse(mouse_event) => {
                    // In newer Crossterm, MouseEvent is a struct with `kind`, `column`, `row`, `modifiers`.
                    // We check if the kind is a 'Down' event (left, right, or middle button).
                    if let MouseEvent {
                        kind: MouseEventKind::Down(_),
                        column: x,
                        row: y,
                        ..
                    } = mouse_event
                    {
                        // Subtract 2 for our prompt offset
                        let row = (y as usize).saturating_sub(2);
                        let col = x as usize;
                        if row < lines.len() {
                            cursor_row = row;
                            cursor_col = col.min(lines[cursor_row].len());
                            redraw = true;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    let result = lines.join("\r\n");
    Ok(result)
}
