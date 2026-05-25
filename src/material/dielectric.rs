use crate::{
    color::Color,
    hittable::HitRecord,
    material::{Material, ScatterResult},
    ray::Ray,
    vec3::Vec3,
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

    /// Schlick's approximation to the Fresnel reflectance.
    ///
    /// The real Fresnel equations give the fraction of light reflected
    /// (vs. transmitted) at a dielectric interface, but they depend on
    /// polarization and require trig of both the incident and refracted
    /// angles. Schlick fits that curve with a cheap polynomial:
    ///
    /// ```text
    /// R(θ) ≈ R₀ + (1 - R₀)(1 - cos θ)⁵
    /// R₀  = ((n₁ - n₂) / (n₁ + n₂))²
    /// ```
    ///
    /// `R₀` is the reflectance at normal incidence (θ = 0, looking straight
    /// at the surface). The `(1 - cos θ)⁵` term grows quickly as θ
    /// approaches 90°, capturing the real-world effect that glass and
    /// water look more like mirrors at grazing angles — think of how the
    /// far surface of a lake reflects the sky.
    ///
    /// Here `n₁ = 1` (air) and `n₂ = refraction_index`, so
    /// `(1 - ri) / (1 + ri)` is `(n₁ - n₂) / (n₁ + n₂)`.
    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
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
    /// Total internal reflection is handled explicitly. Snell's law gives
    /// `sin(θ') = ri · sin(θ)`, so refraction is only physically possible
    /// when `ri · sin(θ) ≤ 1`. When that bound is exceeded — typically a
    /// glancing ray exiting a denser medium into a less dense one — there
    /// is no transmitted direction, and the surface mirrors the ray back
    /// via [`Vec3::reflect`] instead. Without this branch,
    /// [`Vec3::refract`] would produce `NaN` from the negative square root
    /// inside its perpendicular component.
    ///
    /// `cos_theta` uses `-unit_direction · normal` because the incident
    /// ray points *into* the surface while the normal points *out*, so the
    /// raw dot product is negative; the sign flip yields the geometric
    /// angle. `.min(1.0)` guards against floating-point drift pushing the
    /// value slightly above 1 (which would make `sin_theta` `NaN`).
    ///
    /// `sin_theta` follows from the Pythagorean identity
    /// `sin²θ + cos²θ = 1`. Only the positive root is needed since θ is a
    /// geometric angle in `[0, π/2]`, where sine is non-negative.
    ///
    /// The `cannot_refract` test comes straight from Snell's law,
    /// `n₁ sin θ = n₂ sin θ'`, rearranged to `sin θ' = ri · sin θ`. Since
    /// `sin θ'` cannot exceed 1, any `ri · sin θ > 1` means no transmitted
    /// direction exists — total internal reflection.
    ///
    /// The `reflectance(...) > rand::random()` branch is Monte Carlo
    /// sampling of the Fresnel term. A physically faithful tracer would
    /// spawn *both* a reflected and a refracted ray at every dielectric
    /// hit, doubling work at each bounce. Instead we pick one path at
    /// random, weighted by the reflectance probability: if reflectance is
    /// 0.3, we reflect 30% of the time and refract the other 70%. Averaged
    /// over many samples per pixel, this converges to the correct partial
    /// reflection/refraction without the exponential blow-up.
    fn scatter(&self, ray: &Ray, hit: &HitRecord) -> Option<ScatterResult> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if hit.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = ray.direction.unit_vector();
        let cos_theta = -1.0 * unit_direction.dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = ri * sin_theta > 1.0;
        let direction: Vec3;
        if cannot_refract || Dielectric::reflectance(cos_theta, ri) > rand::random::<f64>() {
            direction = unit_direction.reflect(&hit.normal);
        } else {
            let refracted = unit_direction.refract(&hit.normal, ri);
            direction = refracted;
        }

        let scattered = Ray::new(hit.point, direction);
        Some(ScatterResult {
            attenuation,
            ray: scattered,
        })
    }
}
