use base64::prelude::*;
use std::f64::consts::TAU;
use std::fmt;
use std::io::BufWriter;
use std::io::Cursor;
use std::ops::{Add, Mul, Sub};
use wasm_bindgen::prelude::*;
const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;
const PHI: f64 = 1.618_033_988_749_895;
const PHI_SQUARED: f64 = PHI * PHI;

#[wasm_bindgen]
pub fn data_url() -> String {
    let total_frames = 500;
    let mut fake_file: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    {
        // Start a block so that the APNG file will be closed after all of the frames have been written
        let w = &mut BufWriter::new(&mut fake_file);

        let mut encoder = png::Encoder::new(w, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        match encoder.set_animated(total_frames, 0) {
            Ok(_) => (),
            Err(error) => eprintln!("encoder.set_animated result: {error}"),
        };
        match encoder.set_frame_delay(2, 100) {
            Ok(_) => (),
            Err(error) => eprintln!("encoder.set_frame_delay result: {error}"),
        };
        let mut writer = encoder.write_header().unwrap();
        for frame_index in 0..total_frames {
            eprintln!("Football frame {frame_index:03} of {total_frames}");
            let animation_fraction = frame_index as f64 / total_frames as f64;
            let football = Group::football(animation_fraction, ColourScheme::Explanatory)
                .rotate(TAU / 8.0, &Point3d::x_axis())
                .expect("The x_axis vector is not zero.")
                * 0.25;
            save_apng_frame(&mut writer, WIDTH, HEIGHT, &football.spheres);
        }
    }

    let base64_data = BASE64_STANDARD.encode(fake_file.get_ref());
    format!("data:image/png;base64,{}", base64_data)
}

fn save_apng_frame(
    writer: &mut png::Writer<&mut BufWriter<&mut Cursor<Vec<u8>>>>,
    width: u32,
    height: u32,
    spheres: &[Sphere],
) {
    let mut data = vec![];
    for y in 0..width {
        for x in 0..height {
            let normalised_x: f64 = x as f64 / width as f64 * 2.0 - 1.0;
            let normalised_y: f64 = y as f64 / height as f64 * 2.0 - 1.0;
            let result = spheres
                .iter()
                .filter_map(sphere_with_intersection_distance(
                    normalised_x,
                    normalised_y,
                ))
                .reduce(|current, next| {
                    if next.distance < current.distance {
                        next
                    } else {
                        current
                    }
                });
            let grey = match result {
                Some(intersection) => intersection.sphere.grey_value,
                _ => 0,
            };
            data.push(grey)
        }
    }
    writer.write_image_data(&data).unwrap();
}

struct Group {
    spheres: Vec<Sphere>,
}

impl Group {
    fn new(spheres: Vec<Sphere>) -> Self {
        Self { spheres }
    }

    fn dodecahedron(animation_fraction: f64, colour_scheme: ColourScheme) -> Self {
        let grey_values = [30, 50, 70, 90];
        Self::new(
            [
                (0.0, 1.0, PHI, grey_values[1]),
                (0.0, -1.0, PHI, grey_values[3]),
                (0.0, 1.0, -PHI, grey_values[2]),
                (0.0, -1.0, -PHI, grey_values[3]),
                (PHI, 0.0, 1.0, grey_values[2]),
                (-PHI, 0.0, 1.0, grey_values[2]),
                (PHI, 0.0, -1.0, grey_values[1]),
                (-PHI, 0.0, -1.0, grey_values[0]),
                (1.0, PHI, 0.0, grey_values[0]),
                (-1.0, PHI, 0.0, grey_values[3]),
                (1.0, -PHI, 0.0, grey_values[0]),
                (-1.0, -PHI, 0.0, grey_values[1]),
            ]
            .iter()
            .map(|values| Sphere {
                centre: Point3d {
                    x: values.0,
                    y: values.1,
                    z: values.2,
                }
                .normalise()
                .expect("The point is not the origin.")
                .rotate(animation_fraction * TAU, &Point3d::y_axis())
                .expect("The y_axis vector is not zero.")
                    * (4.0 * football_orbital_radius(animation_fraction * TAU) + 0.002)
                    * 0.974
                    + Point3d {
                        x: football_horizontal_offset(animation_fraction * TAU) * 2.0,
                        y: 0.0,
                        z: 0.0,
                    },
                radius: 1.0,
                grey_value: match colour_scheme {
                    ColourScheme::Standard => 0,
                    ColourScheme::Explanatory => values.3,
                },
            })
            .collect(),
        )
    }

    fn icosahedron(animation_fraction: f64, colour_scheme: ColourScheme) -> Self {
        let grey_values = [255, 235, 215];
        Self::new(
            [
                (PHI_SQUARED, 1.0, 0.0, grey_values[0]),
                (-PHI_SQUARED, 1.0, 0.0, grey_values[1]),
                (PHI_SQUARED, -1.0, 0.0, grey_values[2]),
                (-PHI_SQUARED, -1.0, 0.0, grey_values[2]),
                (1.0, 0.0, PHI_SQUARED, grey_values[0]),
                (-1.0, 0.0, PHI_SQUARED, grey_values[2]),
                (1.0, 0.0, -PHI_SQUARED, grey_values[0]),
                (-1.0, 0.0, -PHI_SQUARED, grey_values[1]),
                (0.0, -PHI_SQUARED, 1.0, grey_values[0]),
                (0.0, PHI_SQUARED, 1.0, grey_values[1]),
                (0.0, -PHI_SQUARED, -1.0, grey_values[2]),
                (0.0, PHI_SQUARED, -1.0, grey_values[0]),
                (PHI, PHI, PHI, grey_values[2]),
                (-PHI, PHI, PHI, grey_values[0]),
                (PHI, -PHI, PHI, grey_values[1]),
                (-PHI, -PHI, PHI, grey_values[1]),
                (PHI, PHI, -PHI, grey_values[1]),
                (-PHI, PHI, -PHI, grey_values[2]),
                (PHI, -PHI, -PHI, grey_values[1]),
                (-PHI, -PHI, -PHI, grey_values[0]),
            ]
            .iter()
            .map(|values| Sphere {
                centre: Point3d {
                    x: values.0,
                    y: values.1,
                    z: values.2,
                }
                .normalise()
                .expect("The point is not the origin.")
                .rotate(animation_fraction * TAU, &Point3d::y_axis())
                .expect("The y_axis vector is not zero.")
                    * (4.0 * football_orbital_radius(animation_fraction * TAU) + 0.002)
                    + Point3d {
                        x: -football_horizontal_offset(animation_fraction * TAU) * 2.0,
                        y: 0.0,
                        z: 0.0,
                    },
                radius: 1.0,
                grey_value: match colour_scheme {
                    ColourScheme::Standard => 255,
                    ColourScheme::Explanatory => values.3,
                },
            })
            .collect(),
        )
    }

    fn football(animation_fraction: f64, colour_scheme: ColourScheme) -> Self {
        Self::dodecahedron(animation_fraction, colour_scheme.clone())
            + Self::icosahedron(animation_fraction, colour_scheme)
    }

    fn rotate(&self, angle: f64, axis: &Point3d) -> Option<Self> {
        match axis {
            x if *x == Point3d::origin() => None,
            _ => Some(Self {
                spheres: self
                    .spheres
                    .iter()
                    .filter_map(|sphere| sphere.rotate(angle, axis))
                    .collect(),
            }),
        }
    }
}

impl Add for Group {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut spheres = self.spheres;
        spheres.extend(rhs.spheres);
        Self { spheres }
    }
}

impl Add<Point3d> for Group {
    type Output = Self;

    fn add(self, rhs: Point3d) -> Self::Output {
        Self {
            spheres: self
                .spheres
                .iter()
                .map(|sphere| sphere.clone() + rhs.clone())
                .collect(),
        }
    }
}

impl Sub<Point3d> for Group {
    type Output = Self;

    fn sub(self, rhs: Point3d) -> Self::Output {
        Self {
            spheres: self
                .spheres
                .iter()
                .map(|sphere| sphere.clone() - rhs.clone())
                .collect(),
        }
    }
}

impl Mul<f64> for Group {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            spheres: self
                .spheres
                .iter()
                .map(|sphere| sphere.clone() * rhs)
                .collect(),
        }
    }
}

#[derive(Clone)]
enum ColourScheme {
    Standard,
    Explanatory,
}

#[derive(Clone)]
struct Sphere {
    centre: Point3d,
    radius: f64,
    grey_value: u8,
}

impl Sphere {
    fn rotate(&self, angle: f64, axis: &Point3d) -> Option<Self> {
        Some(Self {
            centre: self.centre.rotate(angle, axis)?,
            radius: self.radius,
            grey_value: self.grey_value,
        })
    }
}

impl fmt::Display for Sphere {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}, {}, {}],",
            self.centre, self.radius, self.grey_value
        )
    }
}

impl Mul<f64> for Sphere {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            centre: self.centre * rhs,
            radius: self.radius * rhs,
            grey_value: self.grey_value,
        }
    }
}

impl Add<Point3d> for Sphere {
    type Output = Self;

    fn add(self, rhs: Point3d) -> Self::Output {
        Self {
            centre: self.centre + rhs,
            radius: self.radius,
            grey_value: self.grey_value,
        }
    }
}

impl Sub<Point3d> for Sphere {
    type Output = Self;

    fn sub(self, rhs: Point3d) -> Self::Output {
        Self {
            centre: self.centre - rhs,
            radius: self.radius,
            grey_value: self.grey_value,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Point3d {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3d {
    fn normalise(&self) -> Option<Self> {
        let distance = Self::distance(self, &Self::origin());
        if distance == 0.0 {
            return None;
        }
        let x = self.x / distance;
        let y = self.y / distance;
        let z = self.z / distance;
        Some(Self { x, y, z })
    }

    fn distance(a: &Self, b: &Self) -> f64 {
        ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0) + (a.z - b.z).powf(2.0)).powf(0.5)
    }

    fn origin() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    // Based on https://en.wikipedia.org/wiki/Rotation_matrix#Rotation_matrix_from_axis_and_angle
    fn rotate(&self, angle: f64, axis: &Self) -> Option<Self> {
        let u = axis.normalise()?;
        let s = angle.sin();
        let c = angle.cos();
        let d = 1.0 - c;
        Some(Self {
            x: self.x * (c + u.x.powf(2.0) * d)
                + self.y * (u.x * u.y * d - u.z * s)
                + self.z * (u.x * u.z * d + u.y * s),
            y: self.x * (u.y * u.x * d + u.z * s)
                + self.y * (c + u.y.powf(2.0) * d)
                + self.z * (u.y * u.z * d - u.x * s),
            z: self.x * (u.z * u.x * d - u.y * s)
                + self.y * (u.z * u.y * d + u.x * s)
                + self.z * (c + u.z.powf(2.0) * d),
        })
    }

    fn x_axis() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn y_axis() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }
}

impl Add for Point3d {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Point3d {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + Self {
            x: -rhs.x,
            y: -rhs.y,
            z: -rhs.z,
        }
    }
}

impl Mul<f64> for Point3d {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl fmt::Display for Point3d {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}, {}", self.x, self.y, self.z)
    }
}

struct Point2d {
    x: f64,
    y: f64,
}

impl Point2d {
    fn distance(a: Self, b: Self) -> f64 {
        ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0)).powf(0.5)
    }
}

struct Intersection {
    sphere: Sphere,
    distance: f64,
}

fn sphere_with_intersection_distance(
    x: f64,
    y: f64,
) -> impl FnMut(&Sphere) -> Option<Intersection> {
    move |sphere| {
        let distance = Point2d::distance(
            Point2d { x, y },
            Point2d {
                x: sphere.centre.x as f64,
                y: sphere.centre.y as f64,
            },
        );
        let radius = sphere.radius as f64;
        if distance < radius {
            Some(Intersection {
                sphere: sphere.clone(),
                distance: 1000.0 + sphere.centre.z as f64
                    - (radius.powf(2.0) - distance.powf(2.0)).powf(0.5)
                    - 1000.0,
            })
        } else {
            None
        }
    }
}

fn football_orbital_radius(a: f64) -> f64 {
    match a {
        x if x < 0.0 => football_orbital_radius(a + TAU),
        x if x < TAU / 12.0 => 0.0,
        x if x < TAU / 6.0 => positive_cos(6.0 * a),
        x if x < TAU * 5.0 / 12.0 => 1.0,
        x if x < TAU / 2.0 => 1.0 - positive_cos(6.0 * a),
        x if x < TAU * 7.0 / 12.0 => 0.0,
        x if x < TAU * 2.0 / 3.0 => positive_cos(6.0 * a),
        x if x < TAU * 11.0 / 12.0 => 1.0,
        x if x < TAU => 1.0 - positive_cos(6.0 * a),
        _ => football_orbital_radius(a - TAU),
    }
}

fn football_horizontal_offset(a: f64) -> f64 {
    match a {
        x if x < 0.0 => football_horizontal_offset(a + TAU),
        x if x < TAU / 4.0 => 0.0,
        x if x < TAU / 3.0 => positive_cos(6.0 * a),
        x if x < TAU * 3.0 / 4.0 => 1.0,
        x if x < TAU * 5.0 / 6.0 => 1.0 - positive_cos(6.0 * a),
        x if x < TAU => 0.0,
        _ => football_horizontal_offset(a - TAU),
    }
}

fn positive_cos(a: f64) -> f64 {
    (a.cos() + 1.0) * 0.5
}
