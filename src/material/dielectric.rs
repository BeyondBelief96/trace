use crate::{
    color::Color,
    hittable::HitRecord,
    material::{Material, ScatterResult},
    ray::Ray,
};

/// A clear, non-absorbing dielectric material — glass, water, diamond, etc.
///
/// Refracts rays that pass through it according to Snell's law and does not
/// tint them. The surrounding medium is assumed to be air/vacuum (IOR ≈ 1.0);
/// `refraction_index` stores only the material's own IOR. Colored or
/// absorbing glass is out of scope for this model.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dielectric {
    /// The material's index of refraction, measured against air/vacuum
    /// (η_material / η_air). Typical values: 1.0 (air), 1.33 (water),
    /// 1.5 (glass), 2.4 (diamond).
    pub refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }
}

impl Material for Dielectric {
    /// Scatters by refracting through the surface.
    ///
    /// The IOR ratio passed to [`Vec3::refract`] depends on whether the ray
    /// is entering or leaving the material, which we read from
    /// `hit.front_face`:
    ///
    /// - `front_face = true` — ray hit the outside of the surface, so it's
    ///   moving from air into the material. The ratio is
    ///   `η_air / η_material = 1.0 / refraction_index`.
    /// - `front_face = false` — ray hit the back of the surface, meaning
    ///   it's already inside the material and now exiting to air. The
    ///   ratio is `η_material / η_air = refraction_index`.
    ///
    /// This works because `HitRecord` always orients `hit.normal` against
    /// the incoming ray, so [`Vec3::refract`]'s convention (incident points
    /// into the surface, normal points out) holds in both cases — only the
    /// IOR ratio flips.
    ///
    /// `attenuation` is `(1, 1, 1)` because a pure dielectric absorbs no
    /// light; every wavelength passes through unchanged. Replace with a
    /// non-white color to model tinted glass.
    ///
    /// `ray.direction` is re-normalized defensively because
    /// [`Vec3::refract`] requires a unit-length incident vector.
    ///
    /// Does not currently handle total internal reflection — when the IOR
    /// ratio and incidence angle make refraction physically impossible,
    /// [`Vec3::refract`] returns `NaN`. Adding the reflect-instead branch
    /// is a follow-up.
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterResult> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if hit.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray.direction.unit_vector();
        let refracted = unit_direction.refract(&hit.normal, ri);

        let scattered = Ray::new(hit.point, refracted);
        Some(ScatterResult {
            attenuation,
            ray: scattered,
        })
    }
}
