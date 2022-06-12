use notan::math::{vec3, Mat4, DEG_TO_RAD};
use notan::prelude::*;

// language=glsl
const VERTEX_SHADER_SOURCE: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout (location = 0) in vec3 aPos;
    layout (location = 2) in vec2 aTexCoord;

    layout(location = 0) out vec2 TexCoord;

    layout(set = 0, binding = 0) uniform Transform {
        mat4 model;
        mat4 view;
        mat4 projection;
    };

    void main()
    {
        gl_Position = projection * view * model * vec4(aPos.x, -aPos.y, aPos.z, 1.0);
        TexCoord = vec2(aTexCoord.x, aTexCoord.y);
    }
  "#
};

// language=glsl
const FRAGMENT_SHADER_SOURCE: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450
    layout(location = 0) out vec4 color;
    layout(location = 0) in vec2 TexCoord;

    // texture sampler
    layout(location = 0) uniform sampler2D texture1;
    layout(location = 1) uniform sampler2D texture2;

    void main()
    {
        color = mix(texture(texture1, TexCoord), texture(texture2, TexCoord), 0.2);
    }
  "#
};

// Represent our transform data
#[derive(Copy, Clone)]
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
    pipeline: Pipeline,
    vbo: Buffer,
    ebo: Buffer,
    ubo: Buffer,
    texture1: Texture,
    texture2: Texture,
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
fn setup(gfx: &mut Graphics) -> State {
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
        0.5,  0.5, 0.0,     1.0, 1.0, // top right
        0.5, -0.5, 0.0,     1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0,    0.0, 0.0, // bottom left
        -0.5,  0.5, 0.0,    0.0, 1.0  // top left
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

    // create transformation
    let size = gfx.size();
    let (width, height) = (size.0 as f32, size.1 as f32);
    let aspect_ratio = width / height;

    let transform = Transform {
        model: Mat4::IDENTITY * Mat4::from_rotation_x(-55.0_f32.to_radians()),
        view: Mat4::IDENTITY * Mat4::from_translation(vec3(0.0, 0.0, -3.0)),
        projection: Mat4::IDENTITY
            * Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect_ratio, 0.1, 100.0),
    };

    // create the uniform buffer object
    let ubo = gfx
        .create_uniform_buffer(0, "Transform")
        .with_data(bytemuck::cast_slice(&[transform]))
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

fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut renderer = gfx.create_renderer();

    let clear = ClearOptions::color(Color::from_rgb(0.2, 0.3, 0.3));

    renderer.begin(Some(&clear));

    renderer.set_pipeline(&state.pipeline);
    renderer.bind_buffers(&[&state.vbo, &state.ebo, &state.ubo]);
    renderer.bind_texture_slot(0, 0, &state.texture1);
    renderer.bind_texture_slot(1, 1, &state.texture2);
    renderer.draw(0, 6);
    renderer.end();

    // render to the screen
    gfx.render(&renderer);
}
