use crate::canvas::Canvas;
use crate::state::State;

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

pub enum BarPosition {
    Top,
    Bottom,
}

pub struct Bar {
    pub(crate) height: u32,
    position: BarPosition,
    pub(crate) draw: Box<dyn Fn(&mut Canvas) -> ()>,

    pub(crate) base_surface: wl_surface::WlSurface,
    layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,

    pub(crate) tmpfile: Option<File>,
    pub(crate) canvas: Option<Canvas>,
    pub(crate) shm_pool: Option<wl_shm_pool::WlShmPool>,
    pub(crate) buffer: Option<wl_buffer::WlBuffer>,
}

impl Bar {
    pub fn new<F>(
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