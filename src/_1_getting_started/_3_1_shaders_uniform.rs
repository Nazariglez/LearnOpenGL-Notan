use notan::prelude::*;

// language=glsl
const VERTEX_SHADER_SOURCE: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout(location = 0) in vec3 aPos;
    void main() {
        gl_Position = vec4(aPos, 1.0);
    }
  "#
};

// language=glsl
const FRAGMENT_SHADER_SOURCE: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450
    layout(location = 0) out vec4 color;
    layout(set = 0, binding = 0) uniform Locals {
        vec4 ourColor;
    };
    void main() {
        color = ourColor;
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
    let vertex_info = VertexInfo::new().attr(0, VertexFormat::Float32x3);

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
        0.5, -0.5, 0.0,   // bottom right
        -0.5, -0.5, 0.0,  // bottom left
        0.0,  0.5, 0.0    // top
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
        .with_data(&[0.0, 0.0, 0.0, 1.0])
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

fn draw(app: &mut App, gfx: &mut Graphics, state: &mut State) {
    // calculate the green color
    let time = app.timer.time_since_init();
    let green_value = time.sin() / 2.0 + 0.5;

    // update the uniform block data
    gfx.set_buffer_data(&state.ubo, &[0.0, green_value, 0.0, 1.0]);

    // create a renderer object
    let mut renderer = gfx.create_renderer();

    let clear = ClearOptions::color(Color::from_rgb(0.2, 0.3, 0.3));
    renderer.begin(Some(&clear));
    renderer.set_pipeline(&state.pipeline);
    renderer.bind_buffer(&state.vbo);
    renderer.draw(0, 3);
    renderer.end();

    // render to the screen
    gfx.render(&renderer);
}
