use crate::canvas::Canvas;
use crate::state::State;

use std::fs::File;

use wayland_client::{
    protocol::*,
};

use wayland_protocols_wlr::layer_shell::v1::client::*;

pub enum BarPosition {
    Top,
    Bottom,
}

pub struct Bar {
    pub(crate) height: u32,
    pub(crate) draw: Box<dyn Fn(&mut Canvas) -> ()>,

    pub(crate) base_surface: wl_surface::WlSurface,

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
            draw: Box::new(draw),

            base_surface,
            
            tmpfile: None,
            canvas: None,
            buffer: None,
            shm_pool: None,
        }
    }
}