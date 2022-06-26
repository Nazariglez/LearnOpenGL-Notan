use learn_open_gl_notan::utils::{Camera, CameraMovement};
use notan::math::{vec3, Mat4, Vec3};
use notan::prelude::*;

const IS_WASM: bool = cfg!(target_arch = "wasm32");

// language=glsl
const COLOR_VERTEX_SHADER: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout (location = 0) in vec3 aPos;

    layout(set = 0, binding = 0) uniform Transform {
        mat4 model;
        mat4 view;
        mat4 projection;
    };

    void main()
    {
        gl_Position = projection * view * model * vec4(aPos, 1.0);
    }
  "#
};

// language=glsl
const COLOR_FRAGMENT_SHADER: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450
    layout(location = 0) out vec4 color;

    layout(set = 0, binding = 1) uniform Light {
        vec3 objectColor;
        vec3 lightColor;
    };

    void main()
    {
        color = vec4(lightColor * objectColor, 1.0);
    }
  "#
};

// language=glsl
const LIGHT_CUBE_FRAGMENT_SHADER: ShaderSource = notan::fragment_shader! {
    r#"
        #version 450
        layout(location = 0) out vec4 color;

        void main() {
            color = vec4(1.0); // white color
        }
    "#
};

// Represent our transform data
#[derive(Copy, Clone, Default)]
#[repr(C)]
struct Transform {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

unsafe impl bytemuck::Zeroable for Transform {}
unsafe impl bytemuck::Pod for Transform {}

// Create a struct to store the app's state
#[derive(AppState)]
struct State {
    pipeline_1: Pipeline,
    pipeline_2: Pipeline,
    vbo: Buffer,
    transform_ubo: Buffer,
    light_ubo: Buffer,
    camera: Camera,
    last_x: f32,
    last_y: f32,
    first_mouse: bool,
}

#[notan_main]
fn main() -> Result<(), String> {
    // init notan using setup as initialization callback
    notan::init_with(setup)
        // pass the update function
        .update(update)
        // pass the draw function
        .draw(draw)
        .build()
}

// initialize the state and return it to be used by notan
fn setup(app: &mut App, gfx: &mut Graphics) -> State {
    // capture the cursor
    app.window().set_capture_cursor(true);

    // Declare the vertex attributes
    let vertex_info = VertexInfo::new().attr(0, VertexFormat::Float32x3); // positions

    // Enable depth test
    let depth_test = DepthStencil {
        write: true,
        compare: CompareMode::Less,
    };

    // build the pipeline
    let pipeline_1 = gfx
        .create_pipeline()
        .from(&COLOR_VERTEX_SHADER, &COLOR_FRAGMENT_SHADER)
        .with_vertex_info(&vertex_info)
        .with_depth_stencil(depth_test)
        .build()
        .unwrap();

    // build the pipeline
    let pipeline_2 = gfx
        .create_pipeline()
        .from(&COLOR_VERTEX_SHADER, &LIGHT_CUBE_FRAGMENT_SHADER)
        .with_vertex_info(&vertex_info)
        .with_depth_stencil(depth_test)
        .build()
        .unwrap();

    // define vertex data
    #[rustfmt::skip]
    let vertices = [
        -0.5, -0.5, -0.5,
        0.5, -0.5, -0.5,
        0.5,  0.5, -0.5,
        0.5,  0.5, -0.5,
        -0.5,  0.5, -0.5,
        -0.5, -0.5, -0.5,

        -0.5, -0.5,  0.5,
        0.5, -0.5,  0.5,
        0.5,  0.5,  0.5,
        0.5,  0.5,  0.5,
        -0.5,  0.5,  0.5,
        -0.5, -0.5,  0.5,

        -0.5,  0.5,  0.5,
        -0.5,  0.5, -0.5,
        -0.5, -0.5, -0.5,
        -0.5, -0.5, -0.5,
        -0.5, -0.5,  0.5,
        -0.5,  0.5,  0.5,

        0.5,  0.5,  0.5,
        0.5,  0.5, -0.5,
        0.5, -0.5, -0.5,
        0.5, -0.5, -0.5,
        0.5, -0.5,  0.5,
        0.5,  0.5,  0.5,

        -0.5, -0.5, -0.5,
        0.5, -0.5, -0.5,
        0.5, -0.5,  0.5,
        0.5, -0.5,  0.5,
        -0.5, -0.5,  0.5,
        -0.5, -0.5, -0.5,

        -0.5,  0.5, -0.5,
        0.5,  0.5, -0.5,
        0.5,  0.5,  0.5,
        0.5,  0.5,  0.5,
        -0.5,  0.5,  0.5,
        -0.5,  0.5, -0.5,
    ];

    // create the vertex buffer object
    let vbo = gfx
        .create_vertex_buffer()
        .with_data(&vertices)
        .with_info(&vertex_info)
        .build()
        .unwrap();

    // create the uniform buffer object
    let transform_ubo = gfx
        .create_uniform_buffer(0, "Transform")
        .build()
        .unwrap();

    let object_color:[f32; 3] = [1.0, 0.5, 0.31];
    let light_color:[f32; 3] = [1.0, 1.0, 1.0];

    let light_ubo = gfx
        .create_uniform_buffer(1, "Light")
        .with_data(bytemuck::cast_slice(&[object_color, light_color]))
        .build()
        .unwrap();

    let camera = Camera::new(vec3(0.0, 0.0, 3.0));

    State {
        pipeline_1,
        pipeline_2,
        vbo,
        transform_ubo,
        light_ubo,
        camera,
        last_x: 0.0,
        last_y: 0.0,
        first_mouse: true
    }
}

fn update(app: &mut App, state: &mut State) {
    if !IS_WASM {
        // if esc is pressed close the app
        if app.keyboard.was_pressed(KeyCode::Escape) {
            app.exit();
        }
    }

    // capture the cursor (wasm32 allow escape the cursor using ESC)
    if app.mouse.was_pressed(MouseButton::Left) && !app.window().capture_cursor() {
        app.window().set_capture_cursor(true);
    }

    // Process all inputs to move the camera
    let delta = app.timer.delta_f32();
    if app.keyboard.is_down(KeyCode::W) {
        state.camera.process_keyboard(CameraMovement::Forward, delta);
    }
    if app.keyboard.is_down(KeyCode::S) {
        state.camera.process_keyboard(CameraMovement::Backward, delta);
    }
    if app.keyboard.is_down(KeyCode::A) {
        state.camera.process_keyboard(CameraMovement::Left, delta);
    }
    if app.keyboard.is_down(KeyCode::D) {
        state.camera.process_keyboard(CameraMovement::Right, delta);
    }

    // process mouse move
    let x = app.mouse.x;
    let y = app.mouse.y;

    if state.first_mouse {
        state.first_mouse = false;
        state.last_x = x;
        state.last_y = y;
    }

    let xoffset = x - state.last_x;
    let yoffset = state.last_y - y;
    state.last_x = x;
    state.last_y = y;

    // state.camera.process_mouse_movement(xoffset, yoffset, false);

    // process zoom
    // state.camera.process_mouse_scroll(app.mouse.wheel_delta.y);
}

fn draw(app: &mut App, gfx: &mut Graphics, state: &mut State) {
    // view/projection transformations
    let size = gfx.size();
    let (width, height) = (size.0 as f32, size.1 as f32);
    let aspect_ratio = width / height;
    let projection = Mat4::IDENTITY
        * Mat4::perspective_rh_gl(state.camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let view = state.camera.get_view_matrix();
    println!("{:?}", state.camera.front);

    // world transformation
    let model = Mat4::IDENTITY;
    //
    // let transform = &[Transform {
    //     projection,
    //     view: Mat4::IDENTITY,
    //     model: Mat4::IDENTITY,
    // }];
    // lighting transform
    let transform = &[Transform {
        model,
        view,
        projection
    }];
    let data: &[f32] = bytemuck::cast_slice(transform);
    gfx.set_buffer_data(&state.transform_ubo, data);

    let mut renderer = gfx.create_renderer();

    let clear = ClearOptions {
        color: Some(Color::from_rgb(0.1, 0.1, 0.1)),
        depth: Some(1.0),
        stencil: None,
    };

    renderer.begin(Some(&clear));

    renderer.set_pipeline(&state.pipeline_1);
    renderer.bind_buffers(&[&state.vbo, &state.transform_ubo, &state.light_ubo]);
    renderer.draw(0, 36);
    //
    // renderer.set_pipeline(&state.pipeline_2);
    // renderer.bind_buffers(&[&state.vbo, &state.transform_ubo]);
    // renderer.draw(0, 36);
    renderer.end();

    // render to the screen
    gfx.render(&renderer);
}
