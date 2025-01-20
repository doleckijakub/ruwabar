use crate::state::State;
use crate::canvas::Canvas;
use crate::bar::{Bar, BarPosition};

use std::os::unix::io::AsFd;
use std::io::Write;
use std::io::Seek;

use wayland_client::{
    protocol::*,
};

pub struct Client {
    state: State,
    
    event_queue: wayland_client::EventQueue<State>,
    qh: wayland_client::QueueHandle<State>,

    bars: Vec<Bar>,
}

impl Client {
    pub fn new() -> Self {
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
            event_queue,
            qh,
            bars: Vec::new(),
        }
    }

    pub fn add_bar<F: Fn(&mut Canvas) -> () + 'static>(&mut self, position: BarPosition, height: u32, draw: F) {
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
    
                let data = canvas.pixels.lock().unwrap();
                tmpfile.rewind().unwrap();
                tmpfile.write_all(bytemuck::cast_slice(&data.clone())).unwrap();
    
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

    pub fn start(&mut self) {
        self.event_queue.blocking_dispatch(&mut self.state).unwrap();
        self.render();

        while self.state.running {
            self.event_queue.blocking_dispatch(&mut self.state).unwrap();
            self.render();
            std::thread::sleep(std::time::Duration::from_millis(1000)); // TODO: remove
        }
    }
}
