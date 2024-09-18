use base64::prelude::*;
use std::f64::consts::TAU;
use std::fmt;
use std::io::BufWriter;
use std::io::Cursor;
use std::ops::{Add, Mul, Sub};
use wasm_bindgen::prelude::*;
const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;

#[wasm_bindgen]
pub fn data_url() -> String {
    let total_frames = 500;
    let forward_tilt = TAU * 3.0 / 16.0;
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
            let animation_fraction = frame_index as f64 / total_frames as f64;
            let eightball = Group::background(128)
                + Group::eightball(animation_fraction)
                    .rotate(forward_tilt, &Point3d::x_axis())
                    .expect("The x_axis vector is not zero.")
                    * 0.4;
            save_apng_frame(&mut writer, WIDTH, HEIGHT, &eightball.spheres);
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

    fn background(grey_value: u8) -> Self {
        Self::new(vec![Sphere {
            centre: Point3d {
                x: 0.0,
                y: 0.0,
                z: 1000.0,
            },
            radius: 2.0,
            grey_value,
        }])
    }

    fn eightball(animation_fraction: f64) -> Self {
        let main_sphere = Sphere {
            centre: Point3d {
                x: 0.0,
                y: 0.003125 + 2.5 * eightball_main_sphere_radial_offset(animation_fraction),
                z: 0.0,
            },
            radius: 1.0,
            grey_value: 0,
        };
        let white_circle_sphere = Sphere {
            centre: Point3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 0.9975,
            grey_value: 255,
        };
        let eight_upper_circle_sphere = Sphere {
            centre: Point3d {
                x: 0.00,
                y: -0.00263 - 3.75 * eightball_circle_radial_upper_offset(animation_fraction),
                z: 0.0,
            }
            .rotate(TAU / 30.0, &Point3d::z_axis())
            .expect("The z_axis vector is not zero.")
            .rotate(
                if animation_fraction > 10.0 / 11.0 {
                    (1.0 - positive_cos((animation_fraction - 10.0 / 11.0) * 11.0 * TAU / 2.0))
                        * TAU
                        / 2.0
                } else {
                    0.0
                },
                &Point3d::y_axis(),
            )
            .expect("The y_axis vector is not zero.")
                + Point3d {
                    x: eightball_horizontal_upper_offset(animation_fraction),
                    y: 0.0,
                    z: 0.0,
                } * 1.25,
            radius: 0.995,
            grey_value: 0,
        };
        let eight_lower_circle_sphere = Sphere {
            centre: Point3d {
                x: 0.0,
                y: -0.00263 - 3.75 * eightball_circle_radial_lower_offset(animation_fraction),
                z: 0.0,
            }
            .rotate(-TAU / 30.0, &Point3d::z_axis())
            .expect("The z_axis vector is not zero.")
            .rotate(
                if animation_fraction > 10.0 / 11.0 {
                    (1.0 - positive_cos((animation_fraction - 10.0 / 11.0) * 11.0 * TAU / 2.0))
                        * TAU
                        / 2.0
                } else {
                    0.0
                },
                &Point3d::y_axis(),
            )
            .expect("The y_axis vector is not zero.")
                - Point3d {
                    x: eightball_horizontal_lower_offset(animation_fraction),
                    y: 0.0,
                    z: 0.0,
                } * 1.25,
            radius: 0.995,
            grey_value: 0,
        };
        let eight_upper_hole_sphere = Sphere {
            centre: Point3d {
                x: 0.0,
                y: -0.00515 - 3.75 * eightball_hole_radial_upper_offset(animation_fraction),
                z: 0.0,
            }
            .rotate(TAU / 30.0, &Point3d::z_axis())
            .expect("The z_axis vector is not zero.")
            .rotate(
                if animation_fraction > 10.0 / 11.0 {
                    (1.0 - positive_cos((animation_fraction - 10.0 / 11.0) * 11.0 * TAU / 2.0))
                        * TAU
                        / 2.0
                } else {
                    0.0
                },
                &Point3d::y_axis(),
            )
            .expect("The y_axis vector is not zero.")
                + Point3d {
                    x: eightball_horizontal_upper_offset(animation_fraction),
                    y: 0.0,
                    z: 0.0,
                } * 1.25,
            radius: 0.9925,
            grey_value: 255,
        };
        let eight_lower_hole_sphere = Sphere {
            centre: Point3d {
                x: 0.0,
                y: -0.00515 - 3.75 * eightball_hole_radial_lower_offset(animation_fraction),
                z: 0.0,
            }
            .rotate(-TAU / 30.0, &Point3d::z_axis())
            .expect("The z_axis vector is not zero.")
            .rotate(
                if animation_fraction > 10.0 / 11.0 {
                    (1.0 - positive_cos((animation_fraction - 10.0 / 11.0) * 11.0 * TAU / 2.0))
                        * TAU
                        / 2.0
                } else {
                    0.0
                },
                &Point3d::y_axis(),
            )
            .expect("The y_axis vector is not zero.")
                - Point3d {
                    x: eightball_horizontal_lower_offset(animation_fraction),
                    y: 0.0,
                    z: 0.0,
                } * 1.25,
            radius: 0.9925,
            grey_value: 255,
        };

        Self::new(vec![
            main_sphere,
            white_circle_sphere,
            eight_upper_circle_sphere,
            eight_lower_circle_sphere,
            eight_upper_hole_sphere,
            eight_lower_hole_sphere,
        ])
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

    fn z_axis() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
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

fn eightball_main_sphere_radial_offset(a: f64) -> f64 {
    match a {
        x if x < 0.0 => eightball_main_sphere_radial_offset(a + 1.0),
        x if x < 1.0 / 11.0 => 1.0 - positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 9.0 / 11.0 => 1.0,
        x if x < 10.0 / 11.0 => 1.0 - positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 1.0 => 0.0,
        _ => eightball_main_sphere_radial_offset(a % 1.0),
    }
}

fn eightball_circle_radial_upper_offset(a: f64) -> f64 {
    match a {
        x if x < 0.0 => eightball_circle_radial_upper_offset(a + 1.0),
        x if x < 1.0 / 11.0 => 0.0,
        x if x < 2.0 / 11.0 => positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 4.0 / 11.0 => 1.0,
        x if x < 6.0 / 11.0 => positive_cos(a * 11.0 / 2.0 * TAU) * 0.5 + 0.5,
        x if x < 7.0 / 11.0 => 1.0,
        x if x < 8.0 / 11.0 => 1.0 - positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 1.0 => 0.0,
        _ => eightball_circle_radial_upper_offset(a % 1.0),
    }
}

fn eightball_circle_radial_lower_offset(a: f64) -> f64 {
    match a {
        x if x < 0.0 => eightball_circle_radial_lower_offset(a + 1.0),
        x if x < 2.0 / 11.0 => 0.0,
        x if x < 3.0 / 11.0 => 1.0 - positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 4.0 / 11.0 => 1.0,
        x if x < 6.0 / 11.0 => positive_cos(a * 11.0 / 2.0 * TAU) * 0.5 + 0.5,
        x if x < 8.0 / 11.0 => 1.0,
        x if x < 9.0 / 11.0 => positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 1.0 => 0.0,
        _ => eightball_circle_radial_lower_offset(a % 1.0),
    }
}

fn eightball_hole_radial_upper_offset(a: f64) -> f64 {
    match a {
        x if x < 0.0 => eightball_hole_radial_upper_offset(a + 1.0),
        x if x < 1.0 / 11.0 => 0.0,
        x if x < 2.0 / 11.0 => positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 7.0 / 11.0 => 1.0,
        x if x < 8.0 / 11.0 => 1.0 - positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 1.0 => 0.0,
        _ => eightball_hole_radial_upper_offset(a % 1.0),
    }
}

fn eightball_hole_radial_lower_offset(a: f64) -> f64 {
    match a {
        x if x < 0.0 => eightball_hole_radial_lower_offset(a + 1.0),
        x if x < 2.0 / 11.0 => 0.0,
        x if x < 3.0 / 11.0 => 1.0 - positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 8.0 / 11.0 => 1.0,
        x if x < 9.0 / 11.0 => positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 1.0 => 0.0,
        _ => eightball_hole_radial_lower_offset(a % 1.0),
    }
}

fn eightball_horizontal_upper_offset(a: f64) -> f64 {
    match a {
        x if x < 0.0 => eightball_horizontal_upper_offset(a + 1.0),
        x if x < 2.0 / 11.0 => 0.0,
        x if x < 3.0 / 11.0 => 1.0 - positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 6.0 / 11.0 => 1.0,
        x if x < 7.0 / 11.0 => positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 1.0 => 0.0,
        _ => eightball_horizontal_upper_offset(a % 1.0),
    }
}

fn eightball_horizontal_lower_offset(a: f64) -> f64 {
    match a {
        x if x < 0.0 => eightball_horizontal_lower_offset(a + 1.0),
        x if x < 3.0 / 11.0 => 0.0,
        x if x < 4.0 / 11.0 => positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 7.0 / 11.0 => 1.0,
        x if x < 8.0 / 11.0 => 1.0 - positive_cos(a * 11.0 / 2.0 * TAU),
        x if x < 1.0 => 0.0,
        _ => eightball_horizontal_lower_offset(a % 1.0),
    }
}

fn positive_cos(a: f64) -> f64 {
    (a.cos() + 1.0) * 0.5
}
