use learn_open_gl_notan::utils::{Camera, CameraMovement};
use notan::math::{vec3, Mat4, Vec3};
use notan::prelude::*;

const IS_WASM: bool = cfg!(target_arch = "wasm32");

// language=glsl
const MATERIAL_VERTEX_SHADER: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aNormal;
    layout (location = 2) in vec2 aTextCoords;

    layout (location = 0) out vec3 FragPos;
    layout (location = 1) out vec3 Normal;
    layout (location = 2) out vec2 TexCoords;

    layout(set = 0, binding = 0) uniform Transform {
        mat4 model;
        mat4 view;
        mat4 projection;
    };

    void main()
    {
        FragPos = vec3(model * vec4(aPos, 1.0));
        Normal = mat3(transpose(inverse(model))) * aNormal;
        TexCoords = aTextCoords;

        gl_Position = projection * view * vec4(FragPos, 1.0);
    }
  "#
};

// language=glsl
const MATERIAL_FRAGMENT_SHADER: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450

    struct Material {
        float shininess;
    };

    struct Light {
        vec3 position;

        vec3 ambient;
        vec3 diffuse;
        vec3 specular;

        float constant;
        float linear;
        float quadratic;
    };

    layout(location = 0) in vec3 FragPos;
    layout(location = 1) in vec3 Normal;
    layout(location = 2) in vec2 TexCoords;

    layout(location = 0) out vec4 color;

    layout(binding = 0) uniform sampler2D diffuse_texture;
    layout(binding = 1) uniform sampler2D specular_texture;

    layout(set = 0, binding = 1) uniform MaterialData {
        vec3 viewPos;
        Material material;
        Light light;
    };

    void main()
    {
        // ambient
        vec3 ambient = light.ambient * texture(diffuse_texture, TexCoords).rgb;
         
        // diffuse 
        vec3 norm = normalize(Normal);
        vec3 lightDir = normalize(light.position - FragPos);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = light.diffuse * diff * texture(diffuse_texture, TexCoords).rgb;  
        
        // specular
        vec3 viewDir = normalize(viewPos - FragPos);
        vec3 reflectDir = reflect(-lightDir, norm);  
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);
        vec3 specular = light.specular * spec * texture(specular_texture, TexCoords).rgb;  
            
        // attenuation
        float distance    = length(light.position - FragPos);
        float attenuation = 1.0 / (light.constant + light.linear * distance + light.quadratic * (distance * distance));

        ambient  *= attenuation;
        diffuse   *= attenuation;
        specular *= attenuation;

        vec3 result = ambient + diffuse + specular;
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
struct Transform {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

#[derive(Copy, Clone, Default)]
#[uniform]
struct Light {
    position: Vec3,

    ambient: Vec3,
    diffuse: Vec3,
    specular: Vec3,

    constant: f32,
    linear: f32,
    quadratic: f32,
}

#[derive(Copy, Clone)]
#[uniform]
struct Material {
    shininess: f32,
}

#[derive(Copy, Clone)]
#[uniform]
struct MaterialData {
    view_pos: Vec3,
    material: Material,
    light: Light,
}

const CUBE_POSITIONS: [Vec3; 10] = [
    vec3(0.0, 0.0, 0.0),
    vec3(2.0, 5.0, -15.0),
    vec3(-1.5, -2.2, -2.5),
    vec3(-3.8, -2.0, -12.3),
    vec3(2.4, -0.4, -3.5),
    vec3(-1.7, 3.0, -7.5),
    vec3(1.3, -2.0, -2.5),
    vec3(1.5, 2.0, -2.5),
    vec3(1.5, 0.2, -1.5),
    vec3(-1.3, 1.0, -1.5),
];

const LIGHT_POS: Vec3 = vec3(1.2, 1.0, 2.0);

// Create a struct to store the app's state
#[derive(AppState)]
struct State {
    material_pipeline: Pipeline,
    light_cube_pipeline: Pipeline,
    vbo: Buffer,
    transform_ubo: Buffer,
    material_ubo: Buffer,
    diffuse_texture: Texture,
    specular_texture: Texture,
    camera: Camera,
    last_x: f32,
    last_y: f32,
    first_mouse: bool,
}

#[notan_main]
fn main() -> Result<(), String> {
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
        .attr(1, VertexFormat::Float32x3) // normals
        .attr(2, VertexFormat::Float32x2); // normals

    // Enable depth test
    let depth_test = DepthStencil {
        write: true,
        compare: CompareMode::Less,
    };

    // build the pipeline
    let material_pipeline = gfx
        .create_pipeline()
        .from(&MATERIAL_VERTEX_SHADER, &MATERIAL_FRAGMENT_SHADER)
        .with_vertex_info(&vertex_info)
        .with_depth_stencil(depth_test)
        .with_texture_location(0, "diffuse_texture")
        .with_texture_location(1, "specular_texture")
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
        // pos                // normals          // tex coords
        -0.5, -0.5, -0.5,     0.0,  0.0, -1.0,    0.0,  0.0,
        0.5, -0.5, -0.5,      0.0,  0.0, -1.0,    1.0,  0.0,
        0.5,  0.5, -0.5,      0.0,  0.0, -1.0,    1.0,  1.0,
        0.5,  0.5, -0.5,      0.0,  0.0, -1.0,    1.0,  1.0,
        -0.5,  0.5, -0.5,     0.0,  0.0, -1.0,    0.0,  1.0,
        -0.5, -0.5, -0.5,     0.0,  0.0, -1.0,    0.0,  0.0,

        -0.5, -0.5,  0.5,     0.0,  0.0,  1.0,    0.0,  0.0,
        0.5, -0.5,  0.5,      0.0,  0.0,  1.0,    1.0,  0.0,
        0.5,  0.5,  0.5,      0.0,  0.0,  1.0,    1.0,  1.0,
        0.5,  0.5,  0.5,      0.0,  0.0,  1.0,    1.0,  1.0,
        -0.5,  0.5,  0.5,     0.0,  0.0,  1.0,    0.0,  1.0,
        -0.5, -0.5,  0.5,     0.0,  0.0,  1.0,    0.0,  0.0,

        -0.5,  0.5,  0.5,    -1.0,  0.0,  0.0,    1.0,  0.0,
        -0.5,  0.5, -0.5,    -1.0,  0.0,  0.0,    1.0,  1.0,
        -0.5, -0.5, -0.5,    -1.0,  0.0,  0.0,    0.0,  1.0,
        -0.5, -0.5, -0.5,    -1.0,  0.0,  0.0,    0.0,  1.0,
        -0.5, -0.5,  0.5,    -1.0,  0.0,  0.0,    0.0,  0.0,
        -0.5,  0.5,  0.5,    -1.0,  0.0,  0.0,    1.0,  0.0,

        0.5,  0.5,  0.5,      1.0,  0.0,  0.0,    1.0,  0.0,
        0.5,  0.5, -0.5,      1.0,  0.0,  0.0,    1.0,  1.0,
        0.5, -0.5, -0.5,      1.0,  0.0,  0.0,    0.0,  1.0,
        0.5, -0.5, -0.5,      1.0,  0.0,  0.0,    0.0,  1.0,
        0.5, -0.5,  0.5,      1.0,  0.0,  0.0,    0.0,  0.0,
        0.5,  0.5,  0.5,      1.0,  0.0,  0.0,    1.0,  0.0,

        -0.5, -0.5, -0.5,     0.0, -1.0,  0.0,    0.0,  1.0,
        0.5, -0.5, -0.5,      0.0, -1.0,  0.0,    1.0,  1.0,
        0.5, -0.5,  0.5,      0.0, -1.0,  0.0,    1.0,  0.0,
        0.5, -0.5,  0.5,      0.0, -1.0,  0.0,    1.0,  0.0,
        -0.5, -0.5,  0.5,     0.0, -1.0,  0.0,    0.0,  0.0,
        -0.5, -0.5, -0.5,     0.0, -1.0,  0.0,    0.0,  1.0,

        -0.5,  0.5, -0.5,     0.0,  1.0,  0.0,    0.0,  1.0,
        0.5,  0.5, -0.5,      0.0,  1.0,  0.0,    1.0,  1.0,
        0.5,  0.5,  0.5,      0.0,  1.0,  0.0,    1.0,  0.0,
        0.5,  0.5,  0.5,      0.0,  1.0,  0.0,    1.0,  0.0,
        -0.5,  0.5,  0.5,     0.0,  1.0,  0.0,    0.0,  0.0,
        -0.5,  0.5, -0.5,     0.0,  1.0,  0.0,    0.0,  1.0,
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

    let material_ubo = gfx
        .create_uniform_buffer(1, "MaterialData")
        .build()
        .unwrap();

    let diffuse_texture = gfx
        .create_texture()
        .from_image(include_bytes!("../../resources/textures/container2.png"))
        .build()
        .unwrap();

    let specular_texture = gfx
        .create_texture()
        .from_image(include_bytes!(
            "../../resources/textures/container2_specular.png"
        ))
        .build()
        .unwrap();

    State {
        material_pipeline,
        light_cube_pipeline,
        vbo,
        transform_ubo,
        material_ubo,
        camera,
        diffuse_texture,
        specular_texture,
        last_x: 0.0,
        last_y: 0.0,
        first_mouse: true,
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

fn draw(gfx: &mut Graphics, state: &mut State) {
    // view/projection transformations
    let size = gfx.size();
    let (width, height) = (size.0 as f32, size.1 as f32);
    let aspect_ratio = width / height;
    let projection =
        Mat4::perspective_rh_gl(state.camera.zoom.to_radians(), aspect_ratio, 0.1, 100.0);
    let view = state.camera.get_view_matrix();

    let light = Light {
        position: LIGHT_POS,
        ambient: Vec3::splat(0.2),
        diffuse: Vec3::splat(0.5),
        specular: Vec3::splat(1.0),
        constant: 1.0,
        linear: 0.09,
        quadratic: 0.032,
    };

    let material = Material { shininess: 32.0 };

    gfx.set_buffer_data(
        &state.material_ubo,
        &MaterialData {
            view_pos: state.camera.position,
            material,
            light,
        },
    );

    CUBE_POSITIONS.iter().enumerate().for_each(|(i, &pos)| {
        let angle = 20.0 * i as f32;
        let translation = Mat4::from_translation(pos);
        let rotation = Mat4::from_axis_angle(vec3(1.0, 0.3, 0.5).normalize(), angle.to_radians());
        let model = translation * rotation;

        // lighting transform
        gfx.set_buffer_data(
            &state.transform_ubo,
            &Transform {
                model,
                view,
                projection,
            },
        );

        let mut renderer = gfx.create_renderer();

        let clear = if i == 0 {
            Some(ClearOptions {
                color: Some(Color::from_rgb(0.1, 0.1, 0.1)),
                depth: Some(1.0),
                stencil: None,
            })
        } else {
            None
        };

        renderer.begin(clear.as_ref());

        renderer.set_pipeline(&state.material_pipeline);
        renderer.bind_buffers(&[&state.vbo, &state.transform_ubo, &state.material_ubo]);
        renderer.bind_texture_slot(0, 0, &state.diffuse_texture);
        renderer.bind_texture_slot(1, 1, &state.specular_texture);
        renderer.draw(0, 36);

        renderer.end();

        gfx.render(&renderer);
    });

    // light point
    let mut renderer = gfx.create_renderer();
    let model = Mat4::from_translation(LIGHT_POS);
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
