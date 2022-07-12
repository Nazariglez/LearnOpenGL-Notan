use learn_open_gl_notan::utils::{Camera, CameraMovement};
use notan::math::{vec3, Mat4, Vec3};
use notan::prelude::*;

const IS_WASM: bool = cfg!(target_arch = "wasm32");

// language=glsl
const BASIC_LIGHTING_VERTEX_SHADER: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aNormal;

    layout (location = 0) out vec3 FragPos;
    layout (location = 1) out vec3 Normal;
    layout (location = 2) out vec3 LightPos;

    layout(set = 0, binding = 0) uniform Transform {
        vec3 lightPos; // light position is now on the vertex shader
        mat4 model;
        mat4 view;
        mat4 projection;
    };

    void main()
    {
        gl_Position = projection * view * model * vec4(aPos, 1.0);

        FragPos = vec3(view * model * vec4(aPos, 1.0));
        Normal = mat3(transpose(inverse(view * model))) * aNormal;
        // Transform world-space light position to view-space light position
        LightPos = vec3(view * vec4(lightPos, 1.0));
    }
  "#
};

// language=glsl
const BASIC_LIGHTING_FRAGMENT_SHADER: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450
    layout(location = 0) in vec3 Normal;
    layout(location = 1) in vec3 FragPos;
    layout(location = 2) in vec3 LightPos;

    layout(location = 0) out vec4 color;

    layout(set = 0, binding = 1) uniform Light {
        vec3 lightColor;
        vec3 objectColor;
    };

    void main()
    {
        // ambient
        float ambientStrength = 0.1;
        vec3 ambient = ambientStrength * lightColor;

         // diffuse
        vec3 norm = normalize(Normal);
        vec3 lightDir = normalize(LightPos - FragPos);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = diff * lightColor;

        // specular
        float specularStrength = 0.5;
        vec3 viewDir = normalize(-FragPos); // the viewer is always at (0,0,0) in view-space, so viewDir is (0,0,0) - Position => -Position
        vec3 reflectDir = reflect(-lightDir, norm);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
        vec3 specular = specularStrength * spec * lightColor;

        vec3 result = (ambient + diffuse + specular) * objectColor;
        color = vec4(result, 1.0);
    }
  "#
};

// language=glsl
const LIGHT_CUBE_VERTEX_SHADER: ShaderSource = notan::vertex_shader! {
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
#[uniform]
struct TransformWithLight {
    light_pos: Vec3,
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

#[derive(Copy, Clone, Default)]
#[uniform]
struct Transform {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

#[derive(Copy, Clone, Default)]
#[uniform]
struct Light {
    light_color: Vec3,
    object_color: Vec3,
}

// Create a struct to store the app's state
#[derive(AppState)]
struct State {
    basic_lighting_pipeline: Pipeline,
    light_cube_pipeline: Pipeline,
    vbo: Buffer,
    transform_ubo: Buffer,
    light_ubo: Buffer,
    camera: Camera,
    last_x: f32,
    last_y: f32,
    first_mouse: bool,
    light: Light,
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
    let vertex_info = VertexInfo::new()
        .attr(0, VertexFormat::Float32x3) // positions
        .attr(1, VertexFormat::Float32x3); // normals

    // Enable depth test
    let depth_test = DepthStencil {
        write: true,
        compare: CompareMode::Less,
    };

    // build the pipeline
    let basic_lighting_pipeline = gfx
        .create_pipeline()
        .from(
            &BASIC_LIGHTING_VERTEX_SHADER,
            &BASIC_LIGHTING_FRAGMENT_SHADER,
        )
        .with_vertex_info(&vertex_info)
        .with_depth_stencil(depth_test)
        .build()
        .unwrap();

    // build the pipeline
    let light_cube_pipeline = gfx
        .create_pipeline()
        .from(&LIGHT_CUBE_VERTEX_SHADER, &LIGHT_CUBE_FRAGMENT_SHADER)
        .with_vertex_info(&vertex_info)
        .with_depth_stencil(depth_test)
        .build()
        .unwrap();

    // define vertex data
    #[rustfmt::skip]
    let vertices = [
        // pos              // normals
        -0.5, -0.5, -0.5,   0.0,  0.0, -1.0,
        0.5, -0.5, -0.5,    0.0,  0.0, -1.0,
        0.5,  0.5, -0.5,    0.0,  0.0, -1.0,
        0.5,  0.5, -0.5,    0.0,  0.0, -1.0,
        -0.5,  0.5, -0.5,   0.0,  0.0, -1.0,
        -0.5, -0.5, -0.5,   0.0,  0.0, -1.0,

        -0.5, -0.5,  0.5,   0.0,  0.0,  1.0,
        0.5, -0.5,  0.5,    0.0,  0.0,  1.0,
        0.5,  0.5,  0.5,    0.0,  0.0,  1.0,
        0.5,  0.5,  0.5,    0.0,  0.0,  1.0,
        -0.5,  0.5,  0.5,   0.0,  0.0,  1.0,
        -0.5, -0.5,  0.5,   0.0,  0.0,  1.0,

        -0.5,  0.5,  0.5,   -1.0,  0.0,  0.0,
        -0.5,  0.5, -0.5,   -1.0,  0.0,  0.0,
        -0.5, -0.5, -0.5,   -1.0,  0.0,  0.0,
        -0.5, -0.5, -0.5,   -1.0,  0.0,  0.0,
        -0.5, -0.5,  0.5,   -1.0,  0.0,  0.0,
        -0.5,  0.5,  0.5,   -1.0,  0.0,  0.0,

        0.5,  0.5,  0.5,    1.0,  0.0,  0.0,
        0.5,  0.5, -0.5,    1.0,  0.0,  0.0,
        0.5, -0.5, -0.5,    1.0,  0.0,  0.0,
        0.5, -0.5, -0.5,    1.0,  0.0,  0.0,
        0.5, -0.5,  0.5,    1.0,  0.0,  0.0,
        0.5,  0.5,  0.5,    1.0,  0.0,  0.0,

        -0.5, -0.5, -0.5,   0.0, -1.0,  0.0,
        0.5, -0.5, -0.5,    0.0, -1.0,  0.0,
        0.5, -0.5,  0.5,    0.0, -1.0,  0.0,
        0.5, -0.5,  0.5,    0.0, -1.0,  0.0,
        -0.5, -0.5,  0.5,   0.0, -1.0,  0.0,
        -0.5, -0.5, -0.5,   0.0, -1.0,  0.0,

        -0.5,  0.5, -0.5,   0.0,  1.0,  0.0,
        0.5,  0.5, -0.5,    0.0,  1.0,  0.0,
        0.5,  0.5,  0.5,    0.0,  1.0,  0.0,
        0.5,  0.5,  0.5,    0.0,  1.0,  0.0,
        -0.5,  0.5,  0.5,   0.0,  1.0,  0.0,
        -0.5,  0.5, -0.5,   0.0,  1.0,  0.0,
    ];

    // create the vertex buffer object
    let vbo = gfx
        .create_vertex_buffer()
        .with_data(&vertices)
        .with_info(&vertex_info)
        .build()
        .unwrap();

    // create the uniform buffer object
    let transform_ubo = gfx.create_uniform_buffer(0, "Transform").build().unwrap();

    let camera = Camera {
        position: vec3(0.0, 0.0, 3.0),
        ..Default::default()
    };

    let light = Light {
        light_color: vec3(1.0, 1.0, 1.0),
        object_color: vec3(1.0, 0.5, 0.31),
    };

    let light_ubo = gfx
        .create_uniform_buffer(1, "Light")
        .with_data(&light)
        .build()
        .unwrap();

    State {
        basic_lighting_pipeline,
        light_cube_pipeline,
        vbo,
        transform_ubo,
        light_ubo,
        camera,
        last_x: 0.0,
        last_y: 0.0,
        first_mouse: true,
        light,
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
        state
            .camera
            .process_keyboard(CameraMovement::Forward, delta);
    }
    if app.keyboard.is_down(KeyCode::S) {
        state
            .camera
            .process_keyboard(CameraMovement::Backward, delta);
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

    state.camera.process_mouse_movement(xoffset, yoffset, false);

    // process zoom
    state
        .camera
        .process_mouse_scroll(app.mouse.wheel_delta.y * delta);
}

fn draw(app: &mut App, gfx: &mut Graphics, state: &mut State) {
    // view/projection transformations
    let size = gfx.size();
    let (width, height) = (size.0 as f32, size.1 as f32);
    let aspect_ratio = width / height;
    let projection = Mat4::IDENTITY
        * Mat4::perspective_rh_gl(state.camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let view = state.camera.get_view_matrix();
    let light_pos = vec3(1.2, 1.0, 2.0);

    // world transformation
    let model = Mat4::IDENTITY;

    // lighting transform
    gfx.set_buffer_data(
        &state.transform_ubo,
        &TransformWithLight {
            light_pos,
            model,
            view,
            projection,
        },
    );

    let mut renderer = gfx.create_renderer();

    let clear = ClearOptions {
        color: Some(Color::from_rgb(0.1, 0.1, 0.1)),
        depth: Some(1.0),
        stencil: None,
    };

    renderer.begin(Some(&clear));

    renderer.set_pipeline(&state.basic_lighting_pipeline);
    renderer.bind_buffers(&[&state.vbo, &state.transform_ubo, &state.light_ubo]);
    renderer.draw(0, 36);

    renderer.end();

    gfx.render(&renderer);

    // --

    // lighting
    let mut renderer = gfx.create_renderer();
    let model = Mat4::from_translation(light_pos);
    let model = model * Mat4::from_scale(Vec3::splat(0.2));

    gfx.set_buffer_data(
        &state.transform_ubo,
        &Transform {
            model,
            view,
            projection,
        },
    );

    renderer.begin(None);
    renderer.set_pipeline(&state.light_cube_pipeline);
    renderer.bind_buffers(&[&state.vbo, &state.transform_ubo]);
    renderer.draw(0, 36);
    renderer.end();

    gfx.render(&renderer);
}
