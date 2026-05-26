//! Camera model and the render loop.
//!
//! A [`Camera`] is constructed through [`CameraBuilder`], which lets you
//! override individual parameters fluently and falls back to sensible
//! defaults for the rest. The camera then owns the [`Camera::render`]
//! method that walks the image plane pixel-by-pixel, casts rays into a
//! scene, accumulates samples, and writes the result as a PPM (P3) image.
//!
//! ### Coordinate setup
//!
//! - The camera is positioned at `look_from` and oriented to face
//!   `look_at`, with `up` fixing the roll. From those three inputs the
//!   builder derives an orthonormal basis `(u, v, w)`: `w` points *away*
//!   from the target (so the camera looks down `-w`), `u` is the local
//!   right, and `v` is the local up.
//! - The viewport (the rectangle of world-space the image plane samples
//!   through) sits one `focus_distance` in front of the camera along
//!   `-w`, *on the focus plane*. Its height is
//!   `2 * tan(vfov/2) * focus_distance`, and its width is scaled to
//!   match the integer image aspect ratio. Placing the viewport on the
//!   focus plane is what makes defocus blur fall out naturally: every
//!   ray for a given pixel passes through the same point on that plane,
//!   so world points sitting there stay sharp regardless of how the ray
//!   origin is jittered across the lens.
//! - Image rows count top-to-bottom; the viewport's `v` axis therefore
//!   points in `-v` (the camera's local *down*) so that increasing pixel
//!   index `j` corresponds to moving down the image.
//!
//! ### Anti-aliasing
//!
//! Each output pixel averages `samples_per_pixel` rays, with each ray
//! jittered by a uniform offset inside the pixel's square footprint. This
//! is plain box-filter supersampling — cheap, and enough to smooth the
//! hard staircase you'd get from a single ray per pixel.

use crate::{
    color::Color,
    framebuffer::Framebuffer,
    hittable::Hittable,
    interval::Interval,
    ray::Ray,
    vec3::{Point3, Vec3},
};

/// Fluent builder for [`Camera`]. Each setter consumes and returns `Self`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraBuilder {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub samples_per_pixel: u32,
    pub maximum_depth: u32,
    pub vertical_fov_deg: f64,
    pub look_from: Point3,
    pub look_at: Point3,
    pub up: Vec3,
    /// Apex angle (in degrees) of the cone whose base is the defocus disk
    /// at the camera and whose tip lies on the focus plane. `0` disables
    /// defocus blur and falls back to pinhole rays from `look_from`.
    pub defocus_angle_deg: f64,
    /// Distance from `look_from` to the focus plane (where the scene is
    /// perfectly sharp). Independent of `look_at`: aim and focus can
    /// disagree, just like a real camera.
    pub focus_distance: f64,
}

impl CameraBuilder {
    /// Builder pre-populated with default settings.
    pub fn new() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 10,
            maximum_depth: 10,
            vertical_fov_deg: 90.0,
            look_from: Point3::default(),
            look_at: Point3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle_deg: 0.0,
            focus_distance: 10.0,
        }
    }

    pub const fn aspect_ratio(mut self, aspect_ratio: f64) -> Self {
        self.aspect_ratio = aspect_ratio;
        self
    }

    pub const fn image_width(mut self, image_width: u32) -> Self {
        self.image_width = image_width;
        self
    }

    pub const fn samples_per_pixel(mut self, samples_per_pixel: u32) -> Self {
        self.samples_per_pixel = samples_per_pixel;
        self
    }

    pub const fn maximum_depth(mut self, maximum_depth: u32) -> Self {
        self.maximum_depth = maximum_depth;
        self
    }

    pub const fn vertical_fov_deg(mut self, vertical_fov: f64) -> Self {
        self.vertical_fov_deg = vertical_fov;
        self
    }

    pub const fn look_from(mut self, look_from: Point3) -> Self {
        self.look_from = look_from;
        self
    }

    pub const fn look_at(mut self, look_at: Point3) -> Self {
        self.look_at = look_at;
        self
    }

    pub const fn up(mut self, up: Vec3) -> Self {
        self.up = up;
        self
    }

    pub const fn defocus_angle_deg(mut self, defocus_angle_deg: f64) -> Self {
        self.defocus_angle_deg = defocus_angle_deg;
        self
    }

    pub const fn focus_distance(mut self, focus_distance: f64) -> Self {
        self.focus_distance = focus_distance;
        self
    }

    /// Computes viewport geometry and returns a ready [`Camera`].
    ///
    /// Image height is `image_width / aspect_ratio`, floored at 1 (very
    /// wide ratios can otherwise truncate to zero). The viewport is sized
    /// from the *actual* integer image dimensions so pixels stay square
    /// after rounding.
    ///
    /// `focal_length` is taken as the distance from `look_from` to
    /// `look_at` — i.e. the image plane passes through the look-at point.
    /// Combined with `vertical_fov_deg`, this fixes the viewport's
    /// world-space size. The camera's orthonormal basis `(u, v, w)` is
    /// then derived from `look_from`, `look_at`, and `up`, and the
    /// viewport edge vectors are expressed in that basis so the rest of
    /// the math is identical to the axis-aligned case.
    pub fn build(self) -> Camera {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let image_height = (f64::from(self.image_width) / self.aspect_ratio) as u32;
        let image_height = if image_height < 1 { 1 } else { image_height };
        let pixel_samples_scale = 1.0 / f64::from(self.samples_per_pixel);

        let camera_center = self.look_from;

        // Viewport is placed *on the focus plane*, one `focus_distance` in
        // front of the camera. Sized so its vertical extent subtends
        // `vertical_fov_deg` at the eye, with width chosen to match the
        // integer image aspect ratio so pixels stay square. Putting the
        // viewport on the focus plane is what makes the disk-origin rays
        // converge there — see the defocus disk setup below.
        let theta_rad = self.vertical_fov_deg.to_radians();
        let h = (theta_rad / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_distance;
        let viewport_width =
            viewport_height * (f64::from(self.image_width) / f64::from(image_height));

        // Right-handed orthonormal basis for the camera frame:
        //   w = unit vector pointing *away* from the target (camera looks down -w)
        //   u = local right, perpendicular to w and the world-up hint
        //   v = local up, completing the basis
        let w = (self.look_from - self.look_at).unit_vector();
        let u = self.up.cross(w).unit_vector();
        let v = w.cross(u);

        // Edge vectors along the viewport, expressed in the camera basis.
        // `viewport_v` points in `-v` (local *down*) because pixel rows
        // are written top-to-bottom.
        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        // Per-pixel spacing in world units.
        let pixel_delta_u = viewport_u / f64::from(self.image_width);
        let pixel_delta_v = viewport_v / f64::from(image_height);

        // Place pixel (0, 0) half a pixel in from the viewport's top-left
        // corner, so each pixel's center — not its corner — lies on the
        // grid. `camera_center - focus_distance * w` is the viewport's
        // midpoint, from which we step back by half a viewport in each
        // edge direction to reach the upper-left corner.
        let viewport_upper_left =
            camera_center - (self.focus_distance * w) - (viewport_u / 2.0) - (viewport_v / 2.0);
        let pixel00_location = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // Defocus disk: a circular "lens" centered on the camera. Its
        // radius is parameterized as the half-angle of the cone whose tip
        // sits on the focus plane and whose base is the disk, so the
        // perceived blur stays consistent as `focus_distance` changes.
        // `defocus_disk_u` / `_v` scale a point sampled from the unit disk
        // into world-space lens coordinates.
        let defocus_radius =
            self.focus_distance * (self.defocus_angle_deg / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Camera {
            image_width: self.image_width,
            image_height,
            center: camera_center,
            pixel00_location,
            pixel_delta_u,
            pixel_delta_v,
            samples_per_pixel: self.samples_per_pixel,
            pixel_samples_scale,
            maximum_depth: self.maximum_depth,
            defocus_enabled: self.defocus_angle_deg > 0.0,
            defocus_disk_u,
            defocus_disk_v,
        }
    }
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 10,
            maximum_depth: 10,
            vertical_fov_deg: 90.0,
            look_from: Point3::default(),
            look_at: Point3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle_deg: 0.0,
            focus_distance: 10.0,
        }
    }
}

/// A configured camera ready to render a scene. Construct via
/// [`CameraBuilder`]; fields are private because they must satisfy
/// invariants the builder enforces (matching deltas, derived height,
/// precomputed sample scale).
pub struct Camera {
    image_width: u32,
    image_height: u32,
    center: Point3,
    /// World-space center of pixel (0, 0).
    pixel00_location: Point3,
    /// World-space step between adjacent pixels in a row / column.
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    samples_per_pixel: u32,
    /// `1 / samples_per_pixel`, cached for averaging the per-pixel sum.
    pixel_samples_scale: f64,
    maximum_depth: u32,
    /// Whether to jitter ray origins across the defocus disk. Precomputed
    /// from `defocus_angle_deg > 0` so `get_ray` doesn't re-derive it per ray.
    defocus_enabled: bool,
    /// World-space basis vectors spanning the defocus disk at the camera.
    /// A point `p` sampled from the unit disk maps to lens position
    /// `center + p.x * defocus_disk_u + p.y * defocus_disk_v`.
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    /// Renders `world` into an in-memory [`Framebuffer`]. Caller is
    /// responsible for serializing it (see `framebuffer::write_ppm` /
    /// `write_png`) — keeps the renderer ignorant of output format and
    /// lets a single trace produce multiple files. Scanline progress is
    /// logged to stderr.
    pub fn render(&self, world: &dyn Hittable) -> Framebuffer {
        let mut framebuffer = Framebuffer::new(self.image_width, self.image_height);
        for j in 0..self.image_height {
            eprintln!("Scanlines remaining: {}", self.image_height - j);
            for i in 0..self.image_width {
                let mut pixel_color = Color::default();
                for _ in 0..self.samples_per_pixel {
                    let ray = self.get_ray(i, j);
                    pixel_color += Self::ray_color(&ray, world, self.maximum_depth);
                }
                framebuffer.set_pixel(i, j, pixel_color * self.pixel_samples_scale);
            }
        }
        eprintln!("Done!");
        framebuffer
    }

    /// Recursively traces `r`, returning the color it carries back.
    ///
    /// On a hit, the surface's material decides how to scatter; the
    /// scattered ray's color is multiplied by the attenuation. On a miss,
    /// returns a vertical sky gradient (white → pale blue). Bottoms out
    /// at black when `depth` reaches zero or scattering fails.
    fn ray_color(r: &Ray, world: &dyn Hittable, depth: u32) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        // 0.001 is to avoid shadow acne.
        if let Some(rec) = world.hit(r, Interval::new(0.001, f64::INFINITY)) {
            let scattered = rec.material.scatter(&r, &rec);
            if let Some(scattered) = scattered {
                return scattered.attenuation * Self::ray_color(&scattered.ray, world, depth - 1);
            }

            return Color::new(0.0, 0.0, 0.0);
        }

        let unit_direction = r.direction.unit_vector();
        let a = 0.5 * (unit_direction.y + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    /// Camera ray through pixel `(i, j)`, jittered for anti-aliasing and
    /// — when defocus is enabled — originating from a random point on the
    /// lens disk rather than the camera center. All such rays still pass
    /// through the same `pixel_sample` on the focus plane, so world points
    /// lying on that plane stay sharp and points off it blur.
    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = Self::sample_square();
        let pixel_sample = self.pixel00_location
            + (f64::from(i) + offset.x) * self.pixel_delta_u
            + (f64::from(j) + offset.y) * self.pixel_delta_v;
        let ray_origin = if self.defocus_enabled {
            self.defocus_disk_sample()
        } else {
            self.center
        };
        let ray_direction = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }

    /// Random point on the camera's defocus disk, in world space.
    fn defocus_disk_sample(&self) -> Point3 {
        let p = Vec3::random_in_unit_disk();
        self.center + p.x * self.defocus_disk_u + p.y * self.defocus_disk_v
    }

    /// Uniform random offset in `[-0.5, 0.5)² × {0}`.
    fn sample_square() -> Point3 {
        Point3::new(
            rand::random::<f64>() - 0.5,
            rand::random::<f64>() - 0.5,
            0.0,
        )
    }
}
