use std::{fs::File, os::unix::io::AsFd};
use std::io::Write;
use std::io::Seek;

use wayland_client::{
    delegate_noop,
    protocol::*,
};

use wayland_protocols::xdg::shell::client::*;

use wayland_protocols_wlr::layer_shell::v1::client::*;

use fontdue::{Font, FontSettings};

#[derive(Default)]
pub struct State {
    pub(crate) running: bool,
    pub(crate) configured: bool,

    pub(crate) compositor: Option<wl_compositor::WlCompositor>,
    pub(crate) layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub(crate) shm: Option<wl_shm::WlShm>,
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
