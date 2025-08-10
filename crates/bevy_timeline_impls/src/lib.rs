
pub mod transform;
pub mod dir_light;
pub mod point_light;
pub mod spot_light;
pub mod std_material;

pub mod prelude {
    pub use super::transform::{TransformSet, Scale, RotaTrans};
    pub use super::dir_light::{DirLightSet, DirLightColor, DirLightLumen};
    pub use super::point_light::{PointLightSet, PointLightColor, PointLightIntensity, PointLightRadius};
    pub use super::spot_light::{SpotLightSet, SpotLightColor, SpotLightIntensity, SpotLightRange, SpotLightRadius};
    pub use super::std_material::{StdMaterialSet, MaterialController, StdMaterialColor, std_material_control};
}