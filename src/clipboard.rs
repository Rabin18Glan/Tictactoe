use base64::{Engine, engine::general_purpose::STANDARD};
use std::io::{self, Write};

#[derive(Default)]
pub struct Clipboard {
    native: Option<arboard::Clipboard>,
}
impl Clipboard {
    pub fn copy(&mut self, text: &str) -> io::Result<&'static str> {
        if self.native.is_none() {
            self.native = arboard::Clipboard::new().ok();
        }
        if let Some(native) = &mut self.native
            && native.set_text(text.to_owned()).is_ok()
        {
            return Ok("Copied!");
        }
        // Supports remote terminals without a desktop clipboard service.
        write!(io::stdout(), "\x1b]52;c;{}\x07", STANDARD.encode(text))?;
        io::stdout().flush()?;
        Ok("Sent to terminal clipboard (OSC 52)")
    }
}
