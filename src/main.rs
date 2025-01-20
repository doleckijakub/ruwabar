mod bar;
mod canvas;
mod client;
mod state;
mod modules;

use bar::BarPosition;
use client::Client;
use modules::*;

use fontdue::{Font, FontSettings};

fn main() {
    let mut client = Client::new();

    let font = Font::from_bytes(include_bytes!("/usr/share/fonts/TTF/HackNerdFontMono-Regular.ttf") as &[u8], FontSettings::default()).unwrap();

    let c1 = 0xFFCF4345u32;
    let c2 = 0xFF44848Cu32;
    let c3 = 0xFF181818u32;
    let c4 = 0xFFBA1245u32;

    let transparent = 0x0u32;
    let black = 0xFF000000u32;

    let modules_top_left = Modules::new()
        .add(SpacingModule { width: 5 })
        .add(ColorModule { width: 40, color: 0xFFFF0018u32 })
        .add(SpacingModule { width: 5 })
        .add(ColorModule { width: 40, color: 0xFF00FF18u32 });

    client.add_bar(BarPosition::Top, 40, move |canvas| {
        canvas.fill(c1 & 0x7FFFFFFF);
        canvas.draw_modules(&modules_top_left, ModulePosition::Left);
    });

    client.add_bar(BarPosition::Bottom, 40, move |canvas| {
        canvas.fill(c2);

        canvas.fill_rounded_rect(5, 5, 100, 30, 15, c3);
        canvas.fill_rounded_rect(10, 10, 90, 20, 10, c4);

        canvas.fill_oval(350, 5, 30, 30, transparent);

        canvas.draw_string(120, 30, "Hello, World!", black, &font, 20.0);
    });

    client.start();
}
