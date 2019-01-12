extern crate libc;
use std::io::Write; // <--- bring flush() into scope
use std::io::prelude::*;
use std::fs::File;

fn main() {
    match get_terminal_size() {
        Ok(size) => println!("{}x{}", size.ws_row, size.ws_col),
        Err(e) => println!("Error: {}", e)
    }
    display_png();
}

fn get_terminal_size() -> Result<libc::winsize, String> {
    let size = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0
    };
    let r = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &size) };
    match r {
        0 => return Ok(size),
        e => return Err(format!("Error reading terminal size: {}", e))
    }
}

// https://sw.kovidgoyal.net/kitty/graphics-protocol.html
fn display_png_file() {
    print!("\x1b_Gf=100,t=f,a=T;{}\x1b\\",
           base64::encode("/home/emmanuel/Pictures/voscilo-2017.png"));
    std::io::stdout().flush().unwrap();
}

fn display_png() -> std::io::Result<()> {
    let mut f = File::open("/home/emmanuel/Pictures/voscilo-2017.png")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    // https://docs.rs/base64/0.10.0/base64/fn.encode_config_slice.html#example
    let mut base64buf = Vec::new();
    // make sure we'll have a slice big enough for base64 + padding
    base64buf.resize(buffer.len()*4/3+4, 0);
    let bytes_written = base64::encode_config_slice(
        &buffer, base64::STANDARD, &mut base64buf);
    println!("base64 is {} bytes long", bytes_written);
    // shorten our vec down to just what was written
    base64buf.resize(bytes_written, 0);

    let mut is_first = true;
    let mut chunks = base64buf.chunks(4096).peekable();
    // https://stackoverflow.com/a/48103219/516188
    while let Some(chunk) = chunks.next() {
        let code = format!(
            "{}{}",
            if is_first { "f=100,a=T," } else { ""},
            if chunks.peek().is_some() { "m=1" } else { "m=0" });
        is_first = !is_first;
        print!("\x1b_G{};{}\x1b\\",
               code, String::from_utf8_lossy(&chunk.to_vec()));
    }
    std::io::stdout().flush().unwrap();

    Ok(())
}
