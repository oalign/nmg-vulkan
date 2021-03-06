extern crate fnv;

use alg;
use render;
use graphics;
use entity;
use components;

use components::transform;

/// Builder pattern for lights
pub struct LightBuilder<'a> {
    manager: &'a mut Manager,
    light: render::Light,
}

impl<'a> LightBuilder<'a> {
    pub fn new(manager: &'a mut Manager) -> LightBuilder<'a> {
        LightBuilder {
            manager,
            light: render::Light {
                vector: alg::Vec3::zero(),
                intensity: 1.0,
                color: graphics::Color::white(),
                radius: 0.0,
            },
        }
    }

    /// Create directional light with given vector \
    /// Usage with `point_with_radius(...)` or
    /// `hemisphere_with_lower_color(...)` results in undefined behavior
    pub fn directional(
        &mut self,
        direction: alg::Vec3,
    ) -> &mut LightBuilder<'a> {
        self.light.vector = -direction.norm();
        self.light.radius = -1.0; // Sentinel
        self
    }

    /// Create point light with given radius \
    /// Position is taken from the associated transform component \
    /// Usage with `directional(...)` or `hemisphere_with_lower_color(...)`
    /// results in undefined behavior
    pub fn point_with_radius(
        &mut self,
        radius: f32,
    ) -> &mut LightBuilder<'a> {
        self.light.radius = radius;
        self
    }

    /// Create hemisphere light with given lower color \
    /// Use `color(...)` to set the upper color field
    /// Usage with `directional(...)` or `point_with_radius(...)` results in
    /// undefined behavior
    pub fn hemisphere_with_lower_color(
        &mut self,
        lower_color: graphics::Color,
    ) -> &mut LightBuilder<'a> {
        self.light.vector = lower_color.into();
        self.light.radius = -2.0; // Sentinel
        self.light.intensity = 0.0; // Unused
        self
    }

    pub fn color(&mut self, color: graphics::Color) -> &mut LightBuilder<'a> {
        self.light.color = color;
        self
    }

    pub fn intensity(&mut self, intensity: f32) -> &mut LightBuilder<'a> {
        debug_assert!(intensity >= 0.0);
        self.light.intensity = intensity;
        self
    }

    /// Finalize
    pub fn for_entity(&mut self, entity: entity::Handle) {
        #[cfg(debug_assertions)] {
            if self.light.radius == 0.0 {
                eprintln!("Warning: Light created with radius of zero");
            }
        }

        #[cfg(debug_assertions)] {
            if self.light.radius == -1.0
                && self.light.vector == alg::Vec3::zero()
            {
                panic!("Directional light has no direction");
            }
        }

        #[cfg(debug_assertions)] {
            if self.light.radius == -2.0 && self.light.intensity > 0.0 {
                panic!("Hemisphere lights do not support intensity");
            }
        }

        self.manager.set(entity, self.light);
    }
}

pub struct Manager {
    instances: fnv::FnvHashMap<entity::Handle, render::Light>,
}

impl components::Component for Manager {
    fn register(&mut self, entity: entity::Handle) {
        self.instances.insert(
            entity,
            render::Light::default(),
        );
    }

    fn registered(&self, entity: entity::Handle) -> bool {
        self.instances.contains_key(&entity)
    }

    fn count(&self) -> usize {
        self.instances.len()
    }

    #[cfg(debug_assertions)] fn debug_name(&self) -> &str { "Light" }
}

impl Manager {
    pub fn new(hint: usize) -> Manager {
        Manager {
            instances: fnv::FnvHashMap::with_capacity_and_hasher(
                hint,
                Default::default(),
            ),
        }
    }

    /// Get light builder that can be used to initialize the light instance
    /// for this entity
    pub fn build(&mut self) -> LightBuilder {
        LightBuilder::new(self)
    }

    fn set(&mut self, entity: entity::Handle, light: render::Light) {
        debug_validate_entity!(self, entity);
        *self.instances.get_mut(&entity).unwrap() = light;
    }

    pub fn set_color(
        &mut self,
        entity: entity::Handle,
        color: graphics::Color,
    ) {
        debug_validate_entity!(self, entity);
        let instance = self.instances.get_mut(&entity).unwrap();
        instance.color = color;
    }

    /// Sets direction vector of directional light.
    /// Normalizes input.
    pub fn set_direction(
        &mut self,
        entity: entity::Handle,
        direction: alg::Vec3,
    ) {
        debug_validate_entity!(self, entity);
        let instance = self.instances.get_mut(&entity).unwrap();
        instance.vector = direction.norm();
    }

    /// Sets `vector` field of light instance directly
    pub fn set_vector_raw(
        &mut self,
        entity: entity::Handle,
        vector: alg::Vec3,
    ) {
        debug_validate_entity!(self, entity);
        let instance = self.instances.get_mut(&entity).unwrap();
        instance.vector = vector;
    }

    /// Update point light positions from transform component
    pub(crate) fn update(&mut self, transforms: &transform::Manager) {
        for (entity, light) in &mut self.instances {
            if light.radius > 0.0 {
                debug_validate_entity!(transforms, *entity);
                light.vector = transforms.get_position(*entity);
            }
        }
    }

    /// Given a position, return the set of lights affecting it
    pub(super) fn cull(
        &self,
        position: alg::Vec3,
    ) -> [render::Light; render::MAX_INSTANCE_LIGHTS] {
        let mut instance_lights = [
            render::Light::default();
            render::MAX_INSTANCE_LIGHTS
        ];

        let mut i = 0;

        for light in self.instances.values() {
            // Directional (or hemisphere)
            if light.radius < 0.0 {
                instance_lights[i] = *light; // Set light
                i += 1;
            }

            // Dummy light
            else if light.radius == 0.0 {
                continue;
            }

            // Point light--check radius for containment
            else if light.radius > position.dist(light.vector) {
                instance_lights[i] = *light; // Set light
                i += 1;
            }

            // Exit after the number of lights per instance is exceeded
            if i == render::MAX_INSTANCE_LIGHTS {
                break;
            }
        }

        instance_lights
    }
}
