use crate::client::frontend::{Viewport as ViewportBase, Renderer};

pub struct Viewport {

}

impl ViewportBase for Viewport {
    type Renderer = super::renderer::Renderer;

    fn new_full_screen() -> Self {
        Viewport { }
    }

    fn open(&mut self) {

    }

    fn close(&mut self) {

    }

    fn create_renderer(&mut self) -> Self::Renderer {
       Self::Renderer::new()
    }
}