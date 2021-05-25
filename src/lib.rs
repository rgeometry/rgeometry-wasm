mod timer {
  pub struct Timer {
    started: Option<f64>,
    paused: Option<f64>,
  }
  // pause(f64)
  // resume(f64)
  // reset(f64)
  // read() -> f64
  // update(f64)
}

mod mouse {
  // use std::borrow::*;
  use std::cell::{Cell, RefCell};
  use std::ops::Deref;
  use std::rc::Rc;
  // use std::sync::Once;

  use gloo_events::EventListener;
  use wasm_bindgen::{JsCast, UnwrapThrowExt};
  use web_sys;

  pub struct MousePosition {
    // _once: Once,
    pub position: Rc<Cell<(i32, i32)>>,
  }

  impl MousePosition {
    pub fn new() -> MousePosition {
      let position = Rc::new(Cell::new((0, 0)));
      let position_ref = position.clone();
      let canvas = super::playground::get_canvas();
      let listener = EventListener::new(&canvas, "mousemove", move |event| {
        let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
        position_ref.set((event.offset_x(), event.offset_y()));
      });
      listener.forget();
      MousePosition { position }
    }
  }
}

thread_local! {
  static MOUSE: mouse::MousePosition = mouse::MousePosition::new();
}

pub mod playground {
  use super::timer::*;

  use gloo_render::*;
  use num::BigRational;
  use rgeometry::data::*;

  use gloo_events::EventListener;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen::{JsCast, UnwrapThrowExt};
  use web_sys;
  use web_sys::EventTarget;

  pub fn get_document() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
  }
  pub fn get_canvas() -> web_sys::HtmlCanvasElement {
    let document = get_document();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
      .dyn_into::<web_sys::HtmlCanvasElement>()
      .map_err(|_| ())
      .unwrap();
    canvas
  }

  pub fn absolute_mouse_position() -> (i32, i32) {
    super::MOUSE.with(|mouse| mouse.position.get())
  }

  pub fn mouse_position() -> (f64, f64) {
    // get canvas transformation matrix.
    // inverse the matrix.
    // transform the mouse position.
    todo!()
  }

  ///
  pub fn set_viewport_height(height: f64) {
    let canvas = get_canvas();
    let context = canvas
      .get_context("2d")
      .unwrap()
      .unwrap()
      .dyn_into::<web_sys::CanvasRenderingContext2d>()
      .unwrap();
    context.reset_transform().unwrap();

    // 20x10
    // height=1
    // ratio = 10 / 1 = 10
    // 2x1
    // scale( 20/ratio/2, -height/2)
    let ratio = canvas.height() as f64 / height;
    context.scale(ratio, -ratio).unwrap();
    context
      .translate(canvas.width() as f64 / ratio * ratio / 2., -height / 2.)
      .unwrap();
    todo!()
  }

  pub fn set_viewport_width(width: f64) {
    todo!()
  }

  pub fn set_viewport(width: f64, height: f64) {
    todo!()
  }

  pub fn render_polygon(poly: &Polygon<BigRational>) {
    todo!()
  }

  pub fn with_points() -> Vec<Point<BigRational, 2>> {
    todo!()
  }

  pub fn on_canvas_click<F>(callback: F)
  where
    F: Fn() + 'static,
  {
    let canvas = super::playground::get_canvas();
    let listener = EventListener::new(&canvas, "click", move |_event| callback());
    listener.forget();
  }
  /*
  polygon.scale_to_fit(width, height)
  polygon.scale_to_fit_height(height)
  polygon.scale_to_fit_width(width)
  */
}
