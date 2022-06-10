use notan::prelude::*;

// language=glsl
const VERTEX_SHADER_SOURCE: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout(location = 0) in vec3 aPos;
    layout(location = 1) in vec3 aColor;

    layout(location = 0) out vec3 ourColor;

    layout(set = 0, binding = 0) uniform Locals {
        float xOffset;
    };

    void main() {
        gl_Position = vec4(aPos.x + xOffset, aPos.y, aPos.z, 1.0);
        ourColor = aColor;
    }
  "#
};

// language=glsl
const FRAGMENT_SHADER_SOURCE: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450
    layout(location = 0) in vec3 ourColor;
    layout(location = 0) out vec4 color;

    void main() {
        color = vec4(ourColor, 1.0);
    }
  "#
};

// Create a struct to store the app's state
#[derive(AppState)]
struct State {
    pipeline: Pipeline,
    vbo: Buffer,
    ubo: Buffer,
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

// initialize the state and return it to be used by notan
fn setup(gfx: &mut Graphics) -> State {
    // Declare the vertex attributes
    let vertex_info = VertexInfo::new()
        .attr(0, VertexFormat::Float32x3)
        .attr(1, VertexFormat::Float32x3);

    // build the pipeline
    let pipeline = gfx
        .create_pipeline()
        .from(&VERTEX_SHADER_SOURCE, &FRAGMENT_SHADER_SOURCE)
        .with_vertex_info(&vertex_info)
        .build()
        .unwrap();

    // define vertex data
    #[rustfmt::skip]
    let vertices = [
        // positions         // colors
        0.5, -0.5, 0.0,     1.0, 0.0, 0.0,      // bottom right
        -0.5, -0.5, 0.0,    0.0, 1.0, 0.0,      // bottom left
        0.0,  0.5, 0.0,     0.0, 0.0, 1.0       // top
    ];

    // create the vertex buffer object
    let vbo = gfx
        .create_vertex_buffer()
        .with_data(&vertices)
        .with_info(&vertex_info)
        .build()
        .unwrap();

    // create the uniform buffer object
    let ubo = gfx
        .create_uniform_buffer(0, "Locals")
        .with_data(&[0.5])
        .build()
        .unwrap();

    State { pipeline, vbo, ubo }
}

fn update(app: &mut App) {
    // if esc is pressed close the app
    if app.keyboard.was_pressed(KeyCode::Escape) {
        app.exit();
    }
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    // create a renderer object
    let mut renderer = gfx.create_renderer();

    let clear = ClearOptions::color(Color::from_rgb(0.2, 0.3, 0.3));
    renderer.begin(Some(&clear));
    renderer.set_pipeline(&state.pipeline);
    renderer.bind_buffers(&[&state.vbo, &state.ubo]);
    renderer.draw(0, 3);
    renderer.end();

    // render to the screen
    gfx.render(&renderer);
}
