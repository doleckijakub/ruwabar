use std::{fs::File, os::unix::io::AsFd};
use std::io::Write;
use std::io::Seek;

use wayland_client::{
    delegate_noop,
    protocol::*,
};

use wayland_protocols::xdg::shell::client::*;

use wayland_protocols_wlr::layer_shell::v1::client::*;

fn main() {
    let mut client = Client::new();

    client.add_bar(BarPosition::Top, 40, |canvas| {
        canvas.pixels.fill(0xFFCF4345u32);
        canvas.fill_rounded_rect(5, 5, 100, 30, 15, 0xFF181818);
        canvas.fill_rounded_rect(10, 10, 90, 20, 10, 0xFFBA1245);
    });

    client.add_bar(BarPosition::Bottom, 40, |canvas| {
        canvas.pixels.fill(0xFF44848Cu32);
        canvas.draw_oval(5, 5, 30, 30, 0u32);
    });

    client.start();
}

struct Canvas {
    width: u32,
    height: u32,
    offset: u32, // in pixels, not bytes
    stride: u32, // in pixels, not bytes

    pixels: Vec<u32>,

    background_color: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32, background_color: u32) -> Self {
        Self {
            width,
            height,
            offset: 0,
            stride: width,
            pixels: vec![background_color; (width * height) as usize],
            background_color,
        }
    }

    pub fn data(&self) -> &Vec<u32> {
        &self.pixels
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(x + y * self.stride + self.offset) as usize] = color;
        }
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
        let x_end = x + width - 1;
        let y_end = y + height - 1;

        for px in x..=x_end {
            self.set_pixel(px, y, color);
            self.set_pixel(px, y_end, color);
        }

        for py in y..=y_end {
            self.set_pixel(x, py, color);
            self.set_pixel(x_end, py, color);
        }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
        for j in y..y + height {
            for i in x..x + width {
                self.set_pixel(i, j, color);
            }
        }
    }

    pub fn draw_line(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, color: u32) {
        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = -(y1 as i32 - y0 as i32).abs();
        let mut sx = if x0 < x1 { 1 } else { -1 };
        let mut sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let (mut x, mut y) = (x0 as i32, y0 as i32);

        while x != x1 as i32 || y != y1 as i32 {
            self.set_pixel(x as u32, y as u32, color);
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_oval(&mut self, cx: u32, cy: u32, width: u32, height: u32, color: u32) {
        let rx = width / 2;
        let ry = height / 2;
        let cx = cx + rx;
        let cy = cy + ry;

        for y in 0..height {
            for x in 0..width {
                let dx = x as i32 - rx as i32;
                let dy = y as i32 - ry as i32;
                if dx * dx * ry as i32 * ry as i32 + dy * dy * rx as i32 * rx as i32
                    <= (rx * ry) as i32 * (rx * ry) as i32
                {
                    self.set_pixel(cx + dx as u32, cy + dy as u32, color);
                }
            }
        }
    }

    pub fn fill_oval(&mut self, cx: u32, cy: u32, width: u32, height: u32, color: u32) {
        let rx = width / 2;
        let ry = height / 2;
        let cx = cx + rx;
        let cy = cy + ry;

        for y in 0..height {
            for x in 0..width {
                let dx = x as i32 - rx as i32;
                let dy = y as i32 - ry as i32;
                if dx * dx * ry as i32 * ry as i32 + dy * dy * rx as i32 * rx as i32
                    <= (rx * ry) as i32 * (rx * ry) as i32
                {
                    self.set_pixel(cx + dx as u32, cy + dy as u32, color);
                }
            }
        }
    }

    pub fn draw_rounded_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        arc_width: u32,
        arc_height: u32,
        color: u32,
    ) {
        self.draw_line(x + arc_width, y, x + width - arc_width, y, color);
        self.draw_line(x + arc_width, y + height - 1, x + width - arc_width, y + height - 1, color);
        self.draw_line(x, y + arc_height, x, y + height - arc_height, color);
        self.draw_line(x + width - 1, y + arc_height, x + width - 1, y + height - arc_height, color);
        self.set_pixel(x + arc_width - 1, y + arc_height - 1, color);
        self.set_pixel(x + width - arc_width, y + arc_height - 1, color);
        self.set_pixel(x + arc_width - 1, y + height - arc_height, color);
        self.set_pixel(x + width - arc_width, y + height - arc_height, color);
    }

    pub fn fill_rounded_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        radius: u32,
        color: u32,
    ) {
        if radius == 0 {
            self.fill_rect(x, y, width, height, color);
            return;
        }

        let radius = radius.min(width / 2).min(height / 2);

        self.fill_oval(x, y, radius * 2, radius * 2, color);
        self.fill_oval(x + width - radius * 2, y, radius * 2, radius * 2, color);
        self.fill_oval(x, y + height - radius * 2, radius * 2, radius * 2, color);
        self.fill_oval(
            x + width - radius * 2,
            y + height - radius * 2,
            radius * 2,
            radius * 2,
            color,
        );

        self.fill_rect(x + radius, y, width - radius * 2, radius, color);
        self.fill_rect(
            x + radius,
            y + height - radius,
            width - radius * 2,
            radius,
            color,
        );

        self.fill_rect(x, y + radius, radius, height - radius * 2, color);
        self.fill_rect(
            x + width - radius,
            y + radius,
            radius,
            height - radius * 2,
            color,
        );

        self.fill_rect(
            x + radius,
            y + radius,
            width - radius * 2,
            height - radius * 2,
            color,
        );
    }
}

impl Clone for Canvas {
    fn clone (&self) -> Self {
        Self {
            pixels: self.pixels.clone(),
            ..*self
        }
    }
}

enum BarPosition {
    Top,
    Bottom,
}

struct Bar {
    height: u32,
    position: BarPosition,
    draw: Box<dyn Fn(&mut Canvas) -> ()>,

    base_surface: wl_surface::WlSurface,
    layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,

    tmpfile: Option<File>,
    canvas: Option<Canvas>,
    shm_pool: Option<wl_shm_pool::WlShmPool>,
    buffer: Option<wl_buffer::WlBuffer>,
}

impl Bar {
    fn new<F>(
        compositor: &wl_compositor::WlCompositor,
        layer_shell: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        position: BarPosition,
        height: u32,
        draw: F,
        qh: &wayland_client::QueueHandle<State>,
    ) -> Self where F: Fn(&mut Canvas) -> () + 'static {
        let base_surface = compositor.create_surface(qh, ());
        let layer_surface = layer_shell.get_layer_surface(
            &base_surface,
            None, // Default output
            zwlr_layer_shell_v1::Layer::Top,
            "ruwabar".to_string(),
            qh,
            (),
        );

        layer_surface.set_size(0, height);
        layer_surface.set_anchor(
            match position {
                BarPosition::Top => zwlr_layer_surface_v1::Anchor::Top,
                BarPosition::Bottom => zwlr_layer_surface_v1::Anchor::Bottom,
            } 
                | zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Right,
        );
        layer_surface.set_exclusive_zone(height as i32);
        base_surface.commit();

        Self {
            height,
            position,
            draw: Box::new(draw),

            base_surface,
            layer_surface,
            
            tmpfile: None,
            canvas: None,
            buffer: None,
            shm_pool: None,
        }
    }
}

#[derive(Default)]
struct State {
    running: bool,
    configured: bool,

    compositor: Option<wl_compositor::WlCompositor>,
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    shm: Option<wl_shm::WlShm>,
}

struct Client {
    state: State,
    
    connection: wayland_client::Connection,
    event_queue: wayland_client::EventQueue<State>,
    qh: wayland_client::QueueHandle<State>,

    bars: Vec<Bar>,
}

impl Client {
    fn new() -> Self {
        let connection = wayland_client::Connection::connect_to_env().unwrap();
        let mut event_queue = connection.new_event_queue();
        let qh = event_queue.handle();

        let display = connection.display();
        display.get_registry(&qh, ());

        let mut state = State {
            running: true,
            ..State::default()
        };

        event_queue.roundtrip(&mut state).unwrap();

        Self {
            state,
            connection,
            event_queue,
            qh,
            bars: Vec::new(),
        }
    }

    fn add_bar<F: Fn(&mut Canvas) -> () + 'static>(&mut self, position: BarPosition, height: u32, draw: F) {
        let compositor = self
            .state
            .compositor
            .as_ref()
            .expect("Compositor not initialized");

        let layer_shell = self
            .state
            .layer_shell
            .as_ref()
            .expect("Layer shell not initialized");

        let bar = Bar::new(
            compositor,
            layer_shell,
            position,
            height,
            draw,
            &self.qh
        );
        self.bars.push(bar);
    }

    fn render(&mut self) {
        for bar in &mut self.bars {
            if !self.state.configured {
                continue;
            }
    
            if let Some(shm) = &self.state.shm {
                let width = 1920;
                let height = bar.height;
                let stride = width * 4;
                let size = stride * height;
    
                let tmpfile = bar.tmpfile.get_or_insert_with(|| {
                    let tmpfile = tempfile::tempfile().unwrap();
                    tmpfile.set_len(size as u64).unwrap();
                    tmpfile
                });
    
                let mut canvas = bar.canvas.get_or_insert_with(|| {
                    let background_color = 0xFF000000u32;
                    Canvas::new(width, height, background_color)
                });
    
                (bar.draw)(&mut canvas);
    
                let data = canvas.data();
                tmpfile.rewind().unwrap();
                tmpfile.write_all(bytemuck::cast_slice(&data)).unwrap();
    
                let shm_pool = bar.shm_pool.get_or_insert_with(|| {
                    shm.create_pool(tmpfile.as_fd(), size as i32, &self.qh, ())
                });
    
                let buffer = bar.buffer.get_or_insert_with(|| {
                    let buffer = shm_pool.create_buffer(
                        0,
                        width as i32,
                        height as i32,
                        stride as i32,
                        wl_shm::Format::Argb8888,
                        &self.qh,
                        (),
                    );
                    buffer
                });
    
                bar.base_surface.attach(Some(buffer), 0, 0);
                bar.base_surface.commit();
            }
        }
    }    

    fn start(&mut self) {
        while self.state.running {
            self.event_queue.blocking_dispatch(&mut self.state).unwrap();
            self.render();
        }
    }
}

impl wayland_client::Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, .. } = event {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ()),
                    );
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell = Some(
                        registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(name, 1, qh, ()),
                    );
                }
                "wl_shm" => {
                    state.shm = Some(
                        registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ()),
                    )
                }
                _ => {
                    // eprintln!("[{name}]: {interface}");
                }
            }
        }
    }
}

impl wayland_client::Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for State {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Closed => {
                state.running = false;
            }
            zwlr_layer_surface_v1::Event::Configure { serial, width, height, .. } => {
                eprintln!("{width}x{height}");
                layer_surface.ack_configure(serial);
                state.configured = true;
            }
            _ => {}
        }
    }
}

delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore zwlr_layer_shell_v1::ZwlrLayerShellV1);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);
