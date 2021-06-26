use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

// mod timer {
//   pub struct Timer {
//     started: Option<f64>,
//     paused: Option<f64>,
//   }
//   // pause(f64)
//   // resume(f64)
//   // reset(f64)
//   // read() -> f64
//   // update(f64)
// }

static MOUSE_X: AtomicI32 = AtomicI32::new(0);
static MOUSE_Y: AtomicI32 = AtomicI32::new(0);

fn get_mouse() -> (i32, i32) {
  (MOUSE_X.load(Relaxed), MOUSE_Y.load(Relaxed))
}

fn set_mouse(x: i32, y: i32) {
  MOUSE_X.store(x, Relaxed);
  MOUSE_Y.store(y, Relaxed);
}

pub mod playground {
  use super::log;
  use num::BigRational;
  use rgeometry::data::*;

  use gloo_events::{EventListener, EventListenerOptions};
  use num::*;
  use rand::distributions::Standard;
  // use rand::distributions::Uniform;
  use rand::Rng;
  use std::convert::*;
  use std::ops::Deref;
  // use std::ops::DerefMut;
  use std::ops::Index;
  use std::sync::Once;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen::{JsCast, UnwrapThrowExt};
  use web_sys::Path2d;

  use once_cell::sync::Lazy;
  use std::sync::Mutex;

  pub fn upd_mouse(event: &web_sys::MouseEvent) {
    super::set_mouse(event.offset_x(), event.offset_y())
  }

  pub fn upd_touch(event: &web_sys::TouchEvent) {
    let x = event.touches().get(0).unwrap().client_x();
    let y = event.touches().get(0).unwrap().client_y();
    super::set_mouse(x, y)
  }

  pub fn get_device_pixel_ratio() -> f64 {
    web_sys::window().unwrap().device_pixel_ratio()
  }
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
    super::get_mouse()
  }

  pub fn mouse_position() -> (f64, f64) {
    let (x, y) = absolute_mouse_position();
    inv_canvas_position(x, y)
  }

  pub fn inv_canvas_position(x: i32, y: i32) -> (f64, f64) {
    let ratio = get_device_pixel_ratio();
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);
    let transform = &context.get_transform().unwrap();
    let inv = transform.inverse();
    let mut pt = web_sys::DomPointInit::new();
    pt.x(x as f64 * ratio);
    pt.y(y as f64 * ratio);
    let out = inv.transform_point_with_point(&pt);
    (out.x(), out.y())
  }

  pub fn from_pixels(pixels: u32) -> f64 {
    let (vw, vh) = get_viewport();
    let canvas = get_canvas();
    let ratio = get_device_pixel_ratio();
    if vw < vh {
      (vw / canvas.width() as f64) * pixels as f64 * ratio
    } else {
      (vh / canvas.height() as f64) * pixels as f64 * ratio
    }
  }
  pub fn get_viewport() -> (f64, f64) {
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);
    let transform = context.get_transform().unwrap();
    let scale = transform.a();
    // let ratio = get_device_pixel_ratio();
    (
      canvas.width() as f64 / scale,
      canvas.height() as f64 / scale,
    )
  }
  pub fn set_viewport(width: f64, height: f64) {
    let pixel_ratio = get_device_pixel_ratio();
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);

    context.reset_transform().unwrap();

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
    context.set_line_width(2. / ratio * pixel_ratio);
  }

  pub fn render_polygon(poly: &Polygon<BigRational>) {
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);

    context.begin_path();
    context.set_line_join("round");
    let mut iter = poly
      .iter_boundary()
      .map(|pt| pt.point().cast(|v| BigRational::to_f64(&v).unwrap()));
    if let Some(origin) = iter.next() {
      let [x, y] = origin.array;
      context.move_to(x, y);
      for pt in iter {
        let [x2, y2] = pt.array;
        context.line_to(x2, y2);
      }
    }
    context.close_path();
    context.stroke();
  }

  pub fn render_line(pts: &[Point<BigRational, 2>]) {
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);

    context.begin_path();
    context.set_line_join("round");
    let mut iter = pts.iter().map(|pt| {
      let pt: Point<f64, 2> = pt.try_into().unwrap();
      pt
    });
    if let Some(origin) = iter.next() {
      let [x, y] = origin.array;
      context.move_to(x, y);
      for pt in iter {
        let [x2, y2] = pt.array;
        context.line_to(x2, y2);
      }
    }
    context.stroke();
  }

  pub fn point_path_2d(pt: &Point<BigRational, 2>) -> Path2d {
    let pt: Point<f64, 2> = pt.into();

    let path = Path2d::new().unwrap();
    path
      .arc(
        *pt.x_coord(),
        *pt.y_coord(),
        from_pixels(15), // radius
        0.0,
        std::f64::consts::PI * 2.,
      )
      .unwrap();
    path
  }

  pub fn render_point(pt: &Point<BigRational, 2>) {
    let canvas = get_canvas();
    let context = get_context_2d(&canvas);

    let path = point_path_2d(pt);

    context.set_fill_style(&JsValue::from_str("green"));
    context.fill_with_path_2d(&path);
    context.stroke_with_path(&path);
  }

  pub fn with_points(n: usize) -> Vec<Point<BigRational, 2>> {
    with_points_from(n, vec![])
  }
  pub fn with_points_from(
    n: usize,
    mut original_pts: Vec<Point<BigRational, 2>>,
  ) -> Vec<Point<BigRational, 2>> {
    static SELECTED: Lazy<Mutex<Option<(usize, i32, i32)>>> = Lazy::new(|| Mutex::new(None));
    static POINTS: Lazy<Mutex<Vec<Point<BigRational, 2>>>> = Lazy::new(|| Mutex::new(vec![]));

    static START: Once = Once::new();

    START.call_once(|| {
      log("Installing handlers.");

      {
        let mut pts = POINTS.lock().unwrap();
        let mut rng = rand::thread_rng();
        pts.append(&mut original_pts);
        let (width, height) = get_viewport();
        let t = Transform::scale(Vector([width * 0.8, height * 0.8]))
          * Transform::translate(Vector([-0.5, -0.5]));
        while pts.len() < n {
          let pt: Point<f64, 2> = rng.sample(Standard);
          let pt = &t * pt;
          pts.push(pt.try_into().unwrap())
        }
      }

      let handle_select = || {
        let (x, y) = absolute_mouse_position();
        let ratio = get_device_pixel_ratio();
        let canvas = get_canvas();
        let context = get_context_2d(&canvas);
        let pts = POINTS.lock().unwrap();

        for (i, pt) in pts.deref().iter().enumerate() {
          // let pt: &Point<BigRational, 2> = &pt;
          let path = point_path_2d(pt);
          let in_path = context.is_point_in_path_with_path_2d_and_f64(
            &path,
            x as f64 * ratio,
            y as f64 * ratio,
          );
          let in_stroke = context.is_point_in_stroke_with_path_and_x_and_y(
            &path,
            x as f64 * ratio,
            y as f64 * ratio,
          );
          if in_path || in_stroke {
            let mut selected = SELECTED.lock().unwrap();
            *selected = Some((i, x, y));
            break;
          }
        }
      };
      on_mousedown(move |event| {
        upd_mouse(event);
        handle_select();
      });
      on_touchstart(move |event| {
        upd_touch(event);
        handle_select();
      });
      on_mouseup(|_event| *SELECTED.lock().unwrap() = None);
      on_touchend(|_event| *SELECTED.lock().unwrap() = None);
      on_touchmove(move |event| {
        if SELECTED.lock().unwrap().is_some() {
          event.prevent_default();
        }
      });
    });

    // Update points if mouse moved.
    {
      let mut selected = SELECTED.lock().unwrap();

      let (mouse_x, mouse_y) = absolute_mouse_position();
      if let Some((i, x, y)) = *selected {
        let (x, y) = inv_canvas_position(x, y);
        let (ox, oy) = inv_canvas_position(mouse_x, mouse_y);
        let dx = (ox - x) as f64;
        let dy = (oy - y) as f64;
        *selected = Some((i, mouse_x, mouse_y));

        let mut pts = POINTS.lock().unwrap();
        let pt = pts.index(i);
        let vector: Vector<BigRational, 2> = Vector([dx, dy]).try_into().unwrap();
        pts[i] = pt + &vector;
      }
    }

    POINTS.lock().unwrap().clone()
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
  pub fn on_mousedown<F>(callback: F)
  where
    F: Fn(&web_sys::MouseEvent) + 'static,
  {
    let canvas = super::playground::get_canvas();
    let listener = EventListener::new(&canvas, "mousedown", move |event| {
      let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }
  pub fn on_mouseup<F>(callback: F)
  where
    F: Fn(&web_sys::MouseEvent) + 'static,
  {
    let canvas = super::playground::get_canvas();
    let listener = EventListener::new(&canvas, "mouseup", move |event| {
      let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }

  pub fn on_touchstart<F>(callback: F)
  where
    F: Fn(&web_sys::TouchEvent) + 'static,
  {
    let options = EventListenerOptions::enable_prevent_default();
    let canvas = super::playground::get_canvas();
    let listener = EventListener::new_with_options(&canvas, "touchstart", options, move |event| {
      let event = event.dyn_ref::<web_sys::TouchEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }

  pub fn on_touchend<F>(callback: F)
  where
    F: Fn(&web_sys::TouchEvent) + 'static,
  {
    let options = EventListenerOptions::enable_prevent_default();
    let canvas = super::playground::get_canvas();
    let listener = EventListener::new_with_options(&canvas, "touchend", options, move |event| {
      let event = event.dyn_ref::<web_sys::TouchEvent>().unwrap_throw();
      callback(event)
    });
    listener.forget();
  }

  pub fn on_touchmove<F>(callback: F)
  where
    F: Fn(&web_sys::TouchEvent) + 'static,
  {
    let options = EventListenerOptions::enable_prevent_default();
    let canvas = super::playground::get_canvas();
    let listener = EventListener::new_with_options(&canvas, "touchmove", options, move |event| {
      let event = event.dyn_ref::<web_sys::TouchEvent>().unwrap_throw();
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
