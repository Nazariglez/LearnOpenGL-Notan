use notan::prelude::*;

// language=glsl
const VERTEX_SHADER_SOURCE: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec3 aColor;
    layout (location = 2) in vec2 aTexCoord;

    layout(location = 0) out vec3 ourColor;
    layout(location = 1) out vec2 TexCoord;

    void main()
    {
        gl_Position = vec4(aPos.x, -aPos.y, aPos.z, 1.0);
        ourColor = aColor;
        TexCoord = vec2(aTexCoord.x, aTexCoord.y);
    }
  "#
};

// language=glsl
const FRAGMENT_SHADER_SOURCE: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450
    layout(location = 0) out vec4 color;

    layout(location = 0) in vec3 ourColor;
    layout(location = 1) in vec2 TexCoord;

    layout(set = 0, binding = 0) uniform Locals {
        float mixValue;
    };

    // texture sampler
    layout(location = 0) uniform sampler2D texture1;
    layout(location = 1) uniform sampler2D texture2;

    void main()
    {
        color = mix(texture(texture1, TexCoord), texture(texture2, TexCoord), mixValue);
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
    mix_value: f32,
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
        .attr(1, VertexFormat::Float32x3) // colors
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
        // positions        // colors        // texture coords
        0.5,  0.5, 0.0,     1.0, 0.0, 0.0,   1.0, 1.0, // top right
        0.5, -0.5, 0.0,     0.0, 1.0, 0.0,   1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0,    0.0, 0.0, 1.0,   0.0, 0.0, // bottom left
        -0.5,  0.5, 0.0,    1.0, 1.0, 0.0,   0.0, 1.0  // top left
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

    let mix_value = 0.2;

    // create the uniform buffer object
    let ubo = gfx
        .create_uniform_buffer(0, "Locals")
        .with_data(&[mix_value])
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
        mix_value,
    }
}

fn update(app: &mut App, state: &mut State) {
    // if esc is pressed close the app
    if app.keyboard.was_pressed(KeyCode::Escape) {
        app.exit();
    }

    let delta = app.timer.delta_f32();
    if app.keyboard.is_down(KeyCode::Down) && state.mix_value >= delta {
        state.mix_value -= app.timer.delta_f32();
    } else if app.keyboard.is_down(KeyCode::Up) && state.mix_value <= 1.0 - delta {
        state.mix_value += app.timer.delta_f32();
    }
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    // update the uniform block data
    gfx.set_buffer_data(&state.ubo, &[state.mix_value]);

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
