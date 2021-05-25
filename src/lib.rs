use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

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
  use num::*;
  use std::fmt;
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

  pub fn get_context_2d(canvas: &web_sys::HtmlCanvasElement) -> web_sys::CanvasRenderingContext2d {
    canvas
      .get_context("2d")
      .unwrap()
      .unwrap()
      .dyn_into::<web_sys::CanvasRenderingContext2d>()
      .unwrap()
  }

  pub fn clear_screen() {
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);
    context.save();
    context.reset_transform().unwrap();
    context.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);
    context.restore();
  }

  pub fn absolute_mouse_position() -> (i32, i32) {
    super::MOUSE.with(|mouse| mouse.position.get())
  }

  pub fn mouse_position() -> (f64, f64) {
    let (x, y) = absolute_mouse_position();
    inv_canvas_position(x, y)
  }

  pub fn inv_canvas_position(x: i32, y: i32) -> (f64, f64) {
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);
    let transform = &context.get_transform().unwrap();
    let inv = transform.inverse();
    let mut pt = web_sys::DomPointInit::new();
    pt.x(x as f64);
    pt.y(y as f64);
    let out = inv.transform_point_with_point(&pt);
    (out.x(), out.y())
  }

  pub fn set_viewport(width: f64, height: f64) {
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);

    context.reset_transform().unwrap();

    let aspect = width / height;
    let ratio_width = canvas.width() as f64 / width;
    let ratio_height = canvas.height() as f64 / height;
    let ratio = if ratio_width < ratio_height {
      ratio_width
    } else {
      ratio_height
    };
    context.scale(ratio, -ratio).unwrap();
    context
      .translate(
        canvas.width() as f64 / ratio / 2.,
        -(canvas.height() as f64 / ratio / 2.),
      )
      .unwrap();
    context.set_line_width(5. / ratio);
  }

  pub fn render_polygon(poly: &Polygon<BigRational>) {
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);

    context.begin_path();
    context.set_line_join("round");
    let mut iter = poly
      .iter()
      .map(|(pt, _meta)| pt.cast(|v| BigRational::to_f64(&v).unwrap()));
    if let Some(origin) = iter.next() {
      let [x, y] = origin.array;
      context.move_to(x, y);
      while let Some(pt) = iter.next() {
        let [x2, y2] = pt.array;
        context.line_to(x2, y2);
      }
    }
    context.close_path();
    context.stroke();
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

  pub fn on_mousemove<F>(callback: F)
  where
    F: Fn(&web_sys::MouseEvent) + 'static,
  {
    let canvas = super::playground::get_canvas();
    let listener = EventListener::new(&canvas, "mousemove", move |event| {
      let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }
  /*
  polygon.scale_to_fit(width, height)
  polygon.scale_to_fit_height(height)
  polygon.scale_to_fit_width(width)
  */
}
