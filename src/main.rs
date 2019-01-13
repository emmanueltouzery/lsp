extern crate libc;
extern crate image;
#[macro_use] extern crate quick_error;
use std::io::Write; // <--- bring flush() into scope

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: std::io::Error) { from() }
        Image(err: image::ImageError) { from() }
    }
}

fn main() {
    match get_terminal_size() {
        Ok(size) => println!("{}x{}", size.ws_row, size.ws_col),
        Err(e) => println!("Error: {}", e)
    }
    display_png().unwrap();
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

fn display_png() -> Result<(), Error> {
    let img = image::open("/home/emmanuel/Pictures/voscilo-2017.png")
        .map(|img| img.thumbnail(64, 64))?;
    let mut png_thumb_vec = Vec::new(); // must be Vec<u8>
    img.write_to(&mut png_thumb_vec, image::ImageOutputFormat::PNG)?;

    // https://docs.rs/base64/0.10.0/base64/fn.encode_config_slice.html#example
    let mut base64buf = Vec::new();
    // make sure we'll have a slice big enough for base64 + padding
    base64buf.resize(png_thumb_vec.len()*4/3+4, 0);
    let bytes_written = base64::encode_config_slice(
        &png_thumb_vec, base64::STANDARD, &mut base64buf);
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
    std::io::stdout().flush()?;

    Ok(())
}
