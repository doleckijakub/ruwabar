use std::{fs::File, os::unix::io::AsFd};
use std::io::Write;

use wayland_client::{
    delegate_noop,
    protocol::*,
};

use wayland_protocols::xdg::shell::client::*;

use wayland_protocols_wlr::layer_shell::v1::client::*;

fn main() {
    let mut client = Client::new();

    client.add_bar(BarPosition::Top, 32, || 0xFFFFFF00u32);
    client.add_bar(BarPosition::Bottom, 32, || 0xFF00FFFFu32);

    client.start();
}

enum BarPosition {
    Top,
    Bottom,
}

struct Bar {
    height: u32,
    position: BarPosition,
    draw: Box<dyn Fn() -> u32>,

    base_surface: wl_surface::WlSurface,
    layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
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
    ) -> Self where F: Fn() -> u32 + 'static {
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
            buffer: None,
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

    fn add_bar<F: Fn() -> u32 + 'static>(&mut self, position: BarPosition, height: u32, draw: F) {
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
            if self.state.configured && bar.buffer.is_none() {
                if let Some(shm) = &self.state.shm {
                    let width = 1920;
                    let height = bar.height;
                    let stride = width * 4;
                    let size = stride * height;

                    let mut tmpfile = tempfile::tempfile().unwrap();
                    tmpfile.set_len(size as u64).unwrap();

                    { // drawing
                        let color = (bar.draw)();
                        let data = vec![color as i32; (width * height) as usize];
                        // for y in 0..height {
                        //     for x in 0..width {
                        //         data[(x + y * width) as usize] |= x as i32;
                        //     }
                        // }
                        tmpfile.write_all(bytemuck::cast_slice(&data)).unwrap();
                    }

                    let pool = shm.create_pool(tmpfile.as_fd(), size as i32, &self.qh, ());
                    let buffer = pool.create_buffer(
                        0,
                        width as i32,
                        height as i32,
                        stride as i32,
                        wl_shm::Format::Argb8888,
                        &self.qh,
                        ()
                    );

                    bar.buffer = Some(buffer.clone());

                    bar.base_surface.attach(Some(&buffer), 0, 0);
                    bar.base_surface.commit();
                }
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
