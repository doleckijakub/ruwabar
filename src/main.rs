use std::{fs::File, os::unix::io::AsFd};

use wayland_client::{
    delegate_noop,
    protocol::*,
    Connection, Dispatch, QueueHandle,
};

use wayland_protocols::xdg::{
    xdg_output::zv1::client::*,
    shell::client::*
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1, zwlr_layer_surface_v1,
};

fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let mut event_queue = conn.new_event_queue();
    let qhandle = event_queue.handle();

    let display = conn.display();
    display.get_registry(&qhandle, ());

    let mut state = State {
        running: true,
        ..State::default()
    };

    while state.running {
        eprintln!("loop da loop");
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}

#[derive(Default)]
struct State {
    running:    bool,
    configured: bool,

    wm_base:       Option<xdg_wm_base::XdgWmBase>,
    base_surface:  Option<wl_surface::WlSurface>,
    buffer:        Option<wl_buffer::WlBuffer>,
    layer_shell:   Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    layer_surface: Option<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    xdg_surface:   Option<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, .. } = event {
            match &interface[..] {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());
                    let surface = compositor.create_surface(qh, ());
                    state.base_surface = Some(surface);

                    if state.layer_shell.is_some() && state.layer_surface.is_none() {
                        state.init_layer_surface(qh);
                    }
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());

                    let (init_w, init_h) = (1920, 20); // Set the width and height for the bar

                    let mut file = tempfile::tempfile().unwrap();
                    draw(&mut file, (init_w, init_h));
                    let pool = shm.create_pool(file.as_fd(), (init_w * init_h * 4) as i32, qh, ());
                    let buffer = pool.create_buffer(
                        0,
                        init_w as i32,
                        init_h as i32,
                        (init_w * 4) as i32,
                        wl_shm::Format::Argb8888,
                        qh,
                        (),
                    );
                    state.buffer = Some(buffer.clone());

                    if state.configured {
                        let surface = state.base_surface.as_ref().unwrap();
                        surface.attach(Some(&buffer), 0, 0);
                        surface.commit();
                    }
                }
                "zwlr_layer_shell_v1" => {
                    let layer_shell =
                        registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, _, _>(name, 1, qh, ());
                    state.layer_shell = Some(layer_shell);

                    if state.base_surface.is_some() && state.layer_surface.is_none() {
                        state.init_layer_surface(qh);
                    }
                }
                "xdg_wm_base" => {
                    let wm_base = registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, 1, qh, ());
                    state.wm_base = Some(wm_base);

                    if state.base_surface.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(qh);
                    }
                }
                _ => {}
            }
        }
    }
}

// Ignore events from these object types in this example.
delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);

fn draw(tmp: &mut File, (buf_x, buf_y): (u32, u32)) {
    use std::{cmp::min, io::Write};
    let mut buf = std::io::BufWriter::new(tmp);
    for y in 0..buf_y {
        for x in 0..buf_x {
            let a = 0xFF;
            let r = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let g = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
            let b = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
            buf.write_all(&[b as u8, g as u8, r as u8, a as u8]).unwrap();
        }
    }
    buf.flush().unwrap();
}

impl State {
    fn init_layer_surface(&mut self, qh: &QueueHandle<State>) {
        eprintln!("::init_layer_surface");
        
        let layer_shell = self.layer_shell.as_ref().unwrap();
        let base_surface = self.base_surface.as_ref().unwrap();
    
        let layer_surface = layer_shell.get_layer_surface(
            base_surface,
            None, // No specific output, use default
            zwlr_layer_shell_v1::Layer::Top, // Place at the top layer
            "ruwabar".to_string(),
            qh,
            (),
        );
    
        layer_surface.set_size(0, 40); // Adjust height as needed
        layer_surface.set_anchor(
            zwlr_layer_surface_v1::Anchor::Top
                | zwlr_layer_surface_v1::Anchor::Left
                | zwlr_layer_surface_v1::Anchor::Right,
        );
    
        layer_surface.set_exclusive_zone(40); // Reserve 40px at the top
        base_surface.commit();
    
        self.layer_surface = Some(layer_surface);
    }
    

    fn init_xdg_surface(&mut self, qh: &QueueHandle<State>) {
        eprintln!("::init_xdg_surface");

        let wm_base = self.wm_base.as_ref().unwrap();
        let base_surface = self.base_surface.as_ref().unwrap();
    
        let xdg_surface = wm_base.get_xdg_surface(base_surface, qh, ());
        let toplevel = xdg_surface.get_toplevel(qh, ());
        
        // Set the toplevel type to "panel" (status bar)
        toplevel.set_app_id("status-bar".to_string());
        toplevel.set_title("Status Bar".to_string());
        
        // TODO
        // Prevent resizing or maximizing
        // toplevel.set_minimized(false);
    
        // Commit the surface changes
        base_surface.commit();
    
        self.xdg_surface = Some((xdg_surface, toplevel));
    }

    fn set_bar_geometry(&self, width: i32, height: i32) {
        if let Some(surface) = &self.base_surface {
            // Attach the buffer and position it
            if let Some(buffer) = &self.buffer {
                surface.attach(Some(buffer), 0, 0);
            }
            // Commit the surface to apply the new geometry
            surface.commit();
        }
    }    
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for State {
    fn event(
        state: &mut Self,
        layer_surface: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure { width, height, .. } = event {
            state.configured = true;
            let surface = state.base_surface.as_ref().unwrap();
            if let Some(ref buffer) = state.buffer {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for State {
    fn event(
        state: &mut Self,
        layer_shell: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        event: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        eprintln!("State::Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> {event:?}");
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            xdg_surface.ack_configure(serial);
            state.configured = true;

            // Set bar geometry after the surface is configured
            state.set_bar_geometry(1920, 40); // Replace with actual screen width and height
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        xdg_toplevel: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        eprintln!("dupa");
    }
}