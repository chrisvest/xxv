use crate::hex_reader::HexReader;
use cursive::traits::View;
use cursive::Printer;
use cursive::XY;
use cursive::event::Event;
use cursive::view::Selector;
use cursive::event::EventResult;
use std::any::Any;
use cursive::direction::Direction;
use cursive::rect::Rect;
use cursive::Vec2;
use cursive::event::AnyCb;

pub struct HexView {

}

impl HexView {
    pub fn new(reader: &mut HexReader) -> HexView {
        HexView {}
    }
}

impl View for HexView {
    fn draw(&self, printer: &Printer) {
        unimplemented!()
    }

    fn layout(&mut self, constraint: Vec2) {
        unimplemented!()
    }

    fn needs_relayout(&self) -> bool {
        unimplemented!()
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        unimplemented!()
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        unimplemented!()
    }

    fn call_on_any<'a>(&mut self, selector: &Selector, callback: AnyCb<'a>) {
        unimplemented!()
    }

    fn focus_view(&mut self, selector: &Selector) -> Result<(), ()> {
        unimplemented!()
    }

    fn take_focus(&mut self, source: Direction) -> bool {
        unimplemented!()
    }

    fn important_area(&self, view_size: Vec2) -> Rect {
        unimplemented!()
    }
}
