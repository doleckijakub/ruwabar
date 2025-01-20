use crate::canvas::Canvas;

pub trait Module {
    fn get_width(&self) -> u32;
    fn draw(&self, canvas: &mut Canvas);
}

#[allow(dead_code)]
pub enum ModulePosition {
    Left,
    Center,
    Right
}

pub struct Modules {
    pub(crate) modules: Vec<Box<dyn Module>>,
}

impl Modules {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn add(mut self, module: impl Module + 'static) -> Self {
        self.modules.push(Box::new(module));
        self
    }
}

pub struct SpacingModule {
    pub width: u32,
}

impl Module for SpacingModule {
    fn get_width(&self) -> u32 { self.width }
    fn draw(&self, _canvas: &mut Canvas) {}
}

pub struct ColorModule {
    pub width: u32,
    pub color: u32,
}

impl Module for ColorModule {
    fn get_width(&self) -> u32 { self.width }
    
    fn draw(&self, canvas: &mut Canvas) {
        canvas.fill(self.color);
    }
}
