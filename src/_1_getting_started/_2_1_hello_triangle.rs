use notan::prelude::*;

// language=glsl
const VERTEX_SHADER_SOURCE: ShaderSource = notan::vertex_shader! {
  r#"
    #version 450
    layout(location = 0) in vec3 aPos;
    void main() {
        gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    }
  "#
};

// language=glsl
const FRAGMENT_SHADER_SOURCE: ShaderSource = notan::fragment_shader! {
  r#"
    #version 450
    layout(location = 0) out vec4 color;
    void main() {
        color = vec4(1.0, 0.5, 0.2, 1.0);
    }
  "#
};

// Create a struct to store the app's state
#[derive(AppState)]
struct State {
    pipeline: Pipeline,
    vbo: Buffer,
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
        -0.5, -0.5, 0.0, // left
        0.5, -0.5, 0.0,  // right
        0.0, 0.5, 0.0    // top
    ];

    // create the vertex buffer object
    let vbo = gfx
        .create_vertex_buffer()
        .with_data(&vertices)
        .with_info(&vertex_info)
        .build()
        .unwrap();

    State { pipeline, vbo }
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

    // define a color to use as clear
    let clear = ClearOptions::color(Color::from_rgb(0.2, 0.3, 0.3));

    // begin the pass
    renderer.begin(Some(&clear));

    // draw our triangle using our pipeline and vbo
    renderer.set_pipeline(&state.pipeline);
    renderer.bind_buffer(&state.vbo);
    renderer.draw(0, 3);

    // end the pass (we only want clear)
    renderer.end();

    // render to the screen
    gfx.render(&renderer);
}
