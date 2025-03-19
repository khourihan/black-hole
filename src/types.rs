use bytemuck::{Pod, Zeroable};
use glam::Vec2;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct View {
    pub camera: [f32; 16],
    pub position: [f32; 3],
    pub focal_length: f32,
    pub resolution: [u32; 2],
    pub frame_count: u32,
    pub flags: u32,
}

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct BlackHole {
    /// Radius of the sphere of influence in which gravitational lensing occurs
    pub cdist: f32,
    /// Black hole spin factor (J/M^2)
    pub a: f32,
    /// Black hole mass
    pub m: f32,
    /// Black hole charge
    pub q: f32,

    /// Radius of the accretion disc
    pub disc_radius: f32,
    /// Height of the accretion disc
    pub disc_height: f32,
    /// Falloff of the volumetric accretion disc (radial, vertical)
    pub disc_falloff: Vec2,
    /// Falloff of the emission of the volumetric accretion disc (radial, vertical)
    pub disc_emission_falloff: Vec2,
    /// Disc temperature variance
    pub disc_temperature_scale: f32,
    /// Disc temperature base value
    pub disc_temperature_offset: f32,
    /// Scale of noise on the accretion disc
    pub disc_radial_scale: f32,

    /// Minimum timestep for spacetime pathtracer
    pub dt_min: f32,
    /// Maximum timestep for spacetime pathtracer
    pub dt_max: f32,
    /// Number of timesteps for spacetime pathtracer
    pub steps: u32,
}

impl Default for BlackHole {
    fn default() -> Self {
        Self {
            cdist: 120.0,
            a: 0.3,
            m: 1.0,
            q: 0.2,
            disc_radius: 10.0,
            disc_height: 0.8,
            disc_falloff: Vec2::new(0.1, 0.5),
            disc_emission_falloff: Vec2::new(0.06, 0.6),
            disc_temperature_scale: 4000.0,
            disc_temperature_offset: 2000.0,
            disc_radial_scale: 8.0,
            dt_min: 0.03,
            dt_max: 10.0, // 1.0 for accretion disc
            steps: 128,   // 256 for accretion disc
        }
    }
}
