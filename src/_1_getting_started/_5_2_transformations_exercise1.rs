use notan::math::{vec3, Mat4};
use notan::prelude::*;

// language=glsl
const VERTEX_SHADER_SOURCE: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aColor;
    layout (location = 2) in vec2 aTexCoord;

    layout(location = 1) out vec2 TexCoord;

    layout(set = 0, binding = 0) uniform Locals {
        mat4 transform;
    };

    void main()
    {
        gl_Position = transform * vec4(aPos.x, -aPos.y, aPos.z, 1.0);
        TexCoord = vec2(aTexCoord.x, aTexCoord.y);
    }
  "#
};

// language=glsl
const FRAGMENT_SHADER_SOURCE: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450
    layout(location = 0) out vec4 color;

    layout(location = 1) in vec2 TexCoord;

    // texture sampler
    layout(location = 0) uniform sampler2D texture1;
    layout(location = 1) uniform sampler2D texture2;

    void main()
    {
        color = mix(texture(texture1, TexCoord), texture(texture2, TexCoord), 0.2);
    }
  "#
};

// Create a struct to store the app's state
#[derive(AppState)]
struct State {
    pipeline: Pipeline,
    vbo: Buffer,
    ebo: Buffer,
    ubo: Buffer,
    texture1: Texture,
    texture2: Texture,
}

fn main() -> Result<(), String> {
    // init notan using setup as initialization callback
    notan::init_with(setup)
        // pass the update function
        .update(update)
        // pass the draw function
        .draw(draw)
        .build()
}

// create transformation
fn create_transform(time: f32) -> Mat4 {
    let mut transform = Mat4::IDENTITY; // make sure to initialize matrix to identity matrix first

    // switched the order
    transform = transform * Mat4::from_rotation_z(time);
    transform * Mat4::from_translation(vec3(0.5, -0.5, 0.0))
}

// initialize the state and return it to be used by notan
fn setup(app: &mut App, gfx: &mut Graphics) -> State {
    // Declare the vertex attributes
    let vertex_info = VertexInfo::new()
        .attr(0, VertexFormat::Float32x3) // positions
        .attr(2, VertexFormat::Float32x2); // texture coords

    // build the pipeline
    let pipeline = gfx
        .create_pipeline()
        .from(&VERTEX_SHADER_SOURCE, &FRAGMENT_SHADER_SOURCE)
        .with_vertex_info(&vertex_info)
        .with_texture_location(0, "texture1")
        .with_texture_location(1, "texture2")
        .build()
        .unwrap();

    // define vertex data
    #[rustfmt::skip]
    let vertices = [
        // positions        // texture coords
        0.5,  0.5, 0.0,     1.0, 1.0,   // top right
        0.5, -0.5, 0.0,     1.0, 0.0,   // bottom right
        -0.5, -0.5, 0.0,    0.0, 0.0,   // bottom left
        -0.5,  0.5, 0.0,    0.0, 1.0    // top left
    ];

    // create the vertex buffer object
    let vbo = gfx
        .create_vertex_buffer()
        .with_data(&vertices)
        .with_info(&vertex_info)
        .build()
        .unwrap();

    #[rustfmt::skip]
    let indices = [
        0, 1, 3, // first triangle
        1, 2, 3  // second triangle
    ];

    // create the elements buffer object
    let ebo = gfx
        .create_index_buffer()
        .with_data(&indices)
        .build()
        .unwrap();

    let transform = create_transform(app.timer.time_since_init());

    // create the uniform buffer object
    let ubo = gfx
        .create_uniform_buffer(0, "Locals")
        .with_data(&transform.to_cols_array())
        .build()
        .unwrap();

    // load and create the gpu texture1
    let texture1 = gfx
        .create_texture()
        .from_image(include_bytes!("../../resources/textures/container.jpg"))
        .build()
        .unwrap();

    // load and create the gpu texture2
    let texture2 = gfx
        .create_texture()
        .from_image(include_bytes!("../../resources/textures/awesomeface.png"))
        .build()
        .unwrap();

    State {
        pipeline,
        vbo,
        ebo,
        ubo,
        texture1,
        texture2,
    }
}

fn update(app: &mut App) {
    // if esc is pressed close the app
    if app.keyboard.was_pressed(KeyCode::Escape) {
        app.exit();
    }
}

fn draw(app: &mut App, gfx: &mut Graphics, state: &mut State) {
    // update the uniform block data
    let transform = create_transform(app.timer.time_since_init());
    gfx.set_buffer_data(&state.ubo, &transform.to_cols_array());

    let mut renderer = gfx.create_renderer();

    let clear = ClearOptions::color(Color::from_rgb(0.2, 0.3, 0.3));

    renderer.begin(Some(&clear));

    renderer.set_pipeline(&state.pipeline);
    renderer.bind_buffers(&[&state.vbo, &state.ebo]);
    renderer.bind_texture_slot(0, 0, &state.texture1);
    renderer.bind_texture_slot(1, 1, &state.texture2);
    renderer.draw(0, 6);
    renderer.end();

    // render to the screen
    gfx.render(&renderer);
}
