#[macro_use] extern crate nmg_vulkan as nmg;

use nmg::alg;
use nmg::render;
use nmg::graphics;
use nmg::entity;
use nmg::components;
use nmg::components::Component;
use nmg::input;
use nmg::debug;

struct Demo {
    last_angle: alg::Vec2,
    cube: Option<entity::Handle>,
    camera: Option<entity::Handle>,
}

default_traits!(Demo, [nmg::FixedUpdate, components::softbody::Iterate]);

impl nmg::Start for Demo {
    fn start(
        &mut self,
        entities:   &mut entity::Manager,
        components: &mut components::Container,
    ) {
        let cube = entities.add();
        components.transforms.register(cube);
        components.draws.register(cube);
        components.draws.bind_model_index(cube, 0);
        self.cube = Some(cube);

        let light = entities.add();
        components.lights.register(light);
        components.lights.build()
            .directional(-alg::Vec3::one())
            .for_entity(light);

        // Add camera
        let camera = entities.add();
        components.transforms.register(camera);
        components.cameras.register(camera);
        self.camera = Some(camera);
    }
}

impl nmg::Update for Demo {
    #[allow(unused_variables)]
    fn update(
        &mut self,
        time:  f64,
        delta: f64,
        metadata: nmg::Metadata,
        screen: nmg::ScreenData,
        parameters: &mut render::Parameters,
        entities:   &mut entity::Manager,
        components: &mut components::Container,
        input: &input::Manager,
        debug: &mut debug::Handler,
    ) {
        parameters.clear_color = graphics::Color::lerp(
            graphics::Color::white(),
            graphics::Color::blue(),
            0.3,
        );

        components.transforms.set(
            self.cube.unwrap(),
            alg::Vec3::zero(),
            alg::Quat::id(),
            alg::Vec3::one(),
        );

        // Compute rotation angle using mouse
        let angle = self.last_angle + input.mouse_delta * 0.005;
        self.last_angle = angle;

        // Orbit camera
        let camera_position =
              alg::Mat3::rotation_y(angle.x as f32)
            * alg::Mat3::rotation_x(angle.y as f32)
            * alg::Mat4::translation(0.0, 0.0, -2.0)
            * alg::Vec3::zero();

        let camera_orientation = alg::Quat::look_at(
            camera_position,
            alg::Vec3::zero(),
            alg::Vec3::up(),
        );

        components.transforms.set(
            self.camera.unwrap(),
            camera_position,
            camera_orientation,
            alg::Vec3::one(),
        );
    }
}

fn main() {
    let demo = Demo {
        last_angle: alg::Vec2::zero(),
        cube: None,
        camera: None,
    };

    let model_data = get_models();
    nmg::go(model_data, demo)
}

// "Load" model(s)
fn get_models() -> Vec<render::ModelData> {
    let cube = render::ModelData::new_with_normals(
        "cube",
        vec![
            // Front Face
            render::Vertex::new_position_color(-0.5,  0.5, -0.5, 1., 1., 1.),
            render::Vertex::new_position_color( 0.5,  0.5, -0.5, 1., 1., 1.),
            render::Vertex::new_position_color( 0.5, -0.5, -0.5, 1., 1., 1.),
            render::Vertex::new_position_color(-0.5, -0.5, -0.5, 1., 1., 1.),

            // Back Face
            render::Vertex::new_position_color(-0.5,  0.5, 0.5, 1., 1., 1.),
            render::Vertex::new_position_color(-0.5, -0.5, 0.5, 1., 1., 1.),
            render::Vertex::new_position_color( 0.5, -0.5, 0.5, 1., 1., 1.),
            render::Vertex::new_position_color( 0.5,  0.5, 0.5, 1., 1., 1.),

            // Top Face
            render::Vertex::new_position_color(-0.5, 0.5, -0.5, 1., 0., 1.),
            render::Vertex::new_position_color(-0.5, 0.5,  0.5, 1., 0., 1.),
            render::Vertex::new_position_color( 0.5, 0.5,  0.5, 1., 0., 1.),
            render::Vertex::new_position_color( 0.5, 0.5, -0.5, 1., 0., 1.),

            // Bottom Face
            render::Vertex::new_position_color(-0.5, -0.5, -0.5, 1., 1., 0.),
            render::Vertex::new_position_color( 0.5, -0.5, -0.5, 1., 1., 0.),
            render::Vertex::new_position_color( 0.5, -0.5,  0.5, 1., 1., 0.),
            render::Vertex::new_position_color(-0.5, -0.5,  0.5, 1., 1., 0.),

            // Left Face
            render::Vertex::new_position_color(-0.5,  0.5, -0.5, 1., 0., 0.),
            render::Vertex::new_position_color(-0.5, -0.5, -0.5, 1., 0., 0.),
            render::Vertex::new_position_color(-0.5, -0.5,  0.5, 1., 0., 0.),
            render::Vertex::new_position_color(-0.5,  0.5,  0.5, 1., 0., 0.),

            // Right Face
            render::Vertex::new_position_color(0.5,  0.5, -0.5, 1., 0., 0.),
            render::Vertex::new_position_color(0.5,  0.5,  0.5, 1., 0., 0.),
            render::Vertex::new_position_color(0.5, -0.5,  0.5, 1., 0., 0.),
            render::Vertex::new_position_color(0.5, -0.5, -0.5, 1., 0., 0.),
        ], vec![
             0,  1,  2,
             2,  3,  0,
             4,  5,  6,
             6,  7,  4,
             8,  9, 10,
            10, 11,  8,
            12, 13, 14,
            14, 15, 12,
            16, 17, 18,
            18, 19, 16,
            20, 21, 22,
            22, 23, 20,
        ],
        render::NormalMode::Flat,
    );

    vec![cube]
}
