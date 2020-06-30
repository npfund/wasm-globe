use nalgebra::geometry::Perspective3;
use nalgebra::geometry::Point3;
use nalgebra::Vector3;
use nalgebra::{Isometry3, Matrix4, Orthographic3, Rotation3};
use ncollide3d::procedural;
use ncollide3d::procedural::TriMesh;
use rand_distr::Normal;
use std::cell::RefCell;
use std::f64::consts::FRAC_PI_2;
use std::f64::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const DOT_COLOR: &str = "rgba(0,0,0,1)";
const TRANSPARENT: &str = "rgba(0,0,0,0)";

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

struct Dot {
    x: f64,
    y: f64,
}

struct DotSet {
    mesh: TriMesh<f64>,
    radius: f64,
    rotation: [f64; 3],
}

impl DotSet {
    fn new(config: &SetConfig) -> DotSet {
        DotSet {
            mesh: procedural::sphere(config.diameter, config.u, config.v, false),
            radius: config.radius,
            rotation: config.ro,
        }
    }
}

struct SetConfig {
    l: f64,
    v: u32,
    u: u32,
    diameter: f64,
    radius: f64,
    ro: [f64; 3],
    period: f64,
    amplitude: f64,
    speed: f64,
    spread: Normal<f64>,
}

#[wasm_bindgen(start)]
pub fn start() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    let width = window.inner_width().unwrap().as_f64().unwrap();
    let height = window.inner_height().unwrap().as_f64().unwrap();
    let resolution = width * height;
    let scale = if resolution > 1800000.0 {
        4
    } else if resolution > 300000.0 {
        3
    } else {
        2
    };

    let model = Isometry3::new(Vector3::x(), nalgebra::zero());
    // let projection = Perspective3::new(width/height, 75.0_f64.to_radians(), 1.0, 1000.0);
    let projection = Orthographic3::new(0.0, 0.25, 0.0, 0.25, 1.0, 100.0);
    let eye = Point3::new(0.0, 0.0, 30.0);
    let target = Point3::new(0.0, 0.0, 0.0);
    let view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());
    let camera = projection.as_matrix() * (view * model).to_homogeneous();

    let sets: Vec<DotSet> = vec![
        SetConfig {
            l: 1.0,
            v: 1 * scale,
            u: 2 * scale,
            diameter: 20.0,
            radius: 4.0,
            ro: [-2.0, -1.0, 3.0],
            period: 0.004,
            amplitude: 100.0,
            speed: 1.0,
            spread: Normal::new(0.0, 50.0).unwrap(),
        },
        SetConfig {
            l: 2.0,
            v: 2 * scale,
            u: 4 * scale,
            diameter: 40.0,
            radius: 3.0,
            ro: [-1.0, 1.0, 2.0],
            period: 0.001,
            amplitude: 200.0,
            speed: 1.0,
            spread: Normal::new(0.0, 150.0).unwrap(),
        },
        SetConfig {
            l: 3.0,
            v: 4 * scale,
            u: 8 * scale,
            diameter: 60.0,
            radius: 2.0,
            ro: [-1.0, 3.0, 1.0],
            period: 0.006,
            amplitude: 150.0,
            speed: 1.0,
            spread: Normal::new(0.0, 300.0).unwrap(),
        },
    ]
    .iter()
    .map(|config| DotSet::new(config))
    .collect();

    let render = Rc::new(RefCell::new(None));
    let g = render.clone();
    let mut t = 0.0;

    let translation = Vector3::new(width / 2.0, height / 2.0, 0.0);

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        context.clear_rect(0.0, 0.0, width, height);
        for set in &sets {
            let rotation_x =
                Rotation3::from_axis_angle(&Vector3::x_axis(), FRAC_PI_2 * t * set.rotation[0]);
            let rotation_y =
                Rotation3::from_axis_angle(&Vector3::y_axis(), FRAC_PI_2 * t * set.rotation[1]);
            let rotation_z =
                Rotation3::from_axis_angle(&Vector3::z_axis(), FRAC_PI_2 * t * set.rotation[2]);
            let rotation = rotation_x * rotation_y * rotation_z;
            draw_dotset(&context, &camera, &translation, &rotation, &set, t);
        }

        t += 0.002;
        request_animation_frame(render.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn draw_dot(context: &CanvasRenderingContext2d, point: &Point3<f64>, radius: f64) {
    context.begin_path();

    context
        .arc(point.x, point.y, radius, 0.0, 2.0 * PI)
        .unwrap();

    let mut gradient = context
        .create_radial_gradient(point.x, point.y, 0.0, point.x, point.y, radius)
        .unwrap();
    gradient.add_color_stop(0.0, DOT_COLOR).unwrap();
    gradient.add_color_stop(1.0, TRANSPARENT).unwrap();

    context.set_fill_style(&gradient);
    context.fill();

    context.close_path();
}

fn draw_line(
    context: &CanvasRenderingContext2d,
    start: &Point3<f64>,
    end: &Point3<f64>,
    color: &str,
) {
    context.begin_path();

    context.set_stroke_style(&JsValue::from_str(color));
    context.move_to(start.x, start.y);
    context.line_to(end.x, end.y);
    context.stroke();

    context.close_path();
}

fn draw_dotset(
    context: &CanvasRenderingContext2d,
    camera: &Matrix4<f64>,
    translation: &Vector3<f64>,
    rotation: &Rotation3<f64>,
    dotset: &DotSet,
    t: f64,
) {
    for point in &dotset.mesh.coords {
        let point1 = camera.transform_point(&(rotation * point)) + translation;
        // let point = (rotation * point) + translation;
        // log!("{:?} {:?}", point, point1);
        draw_dot(context, &point1, dotset.radius);
    }
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
