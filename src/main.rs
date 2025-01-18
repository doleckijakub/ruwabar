mod bar;
mod canvas;
mod client;
mod state;

use bar::BarPosition;
use client::Client;

use fontdue::{Font, FontSettings};

fn main() {
    let mut client = Client::new();

    client.add_bar(BarPosition::Top, 40, |canvas| {
        canvas.fill(0xFFCF4345u32);

        canvas.fill_rounded_rect(5, 5, 100, 30, 15, 0xFF181818);
        canvas.fill_rounded_rect(10, 10, 90, 20, 10, 0xFFBA1245);

        let font = Font::from_bytes(include_bytes!("/usr/share/fonts/TTF/HackNerdFontMono-Regular.ttf") as &[u8], FontSettings::default()).unwrap();

        canvas.draw_string(120, 30, "Hello, World!", 0xFF000000, &font, 20.0);
    });

    client.add_bar(BarPosition::Bottom, 40, |canvas| {
        canvas.fill(0xFF44848Cu32);
        canvas.draw_oval(5, 5, 30, 30, 0u32);
    });

    client.start();
}
