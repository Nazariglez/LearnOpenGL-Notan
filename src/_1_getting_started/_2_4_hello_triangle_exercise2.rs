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
    vbo1: Buffer,
    vbo2: Buffer,
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
    let first_triangle = [
        -0.9, -0.5, 0.0,  // let
        -0.0, -0.5, 0.0,  // right
        -0.45, 0.5, 0.0,  // top
    ];

    #[rustfmt::skip]
    let second_triangle = [
        0.0, -0.5, 0.0,  // let
        0.9, -0.5, 0.0,  // right
        0.45, 0.5, 0.0   // top 
    ];

    // create the vertex buffer object
    let vbo1 = gfx
        .create_vertex_buffer()
        .with_data(&first_triangle)
        .with_info(&vertex_info)
        .build()
        .unwrap();

    // create the vertex buffer object
    let vbo2 = gfx
        .create_vertex_buffer()
        .with_data(&second_triangle)
        .with_info(&vertex_info)
        .build()
        .unwrap();

    State {
        pipeline,
        vbo1,
        vbo2,
    }
}

fn update(app: &mut App) {
    // if esc is pressed close the app
    if app.keyboard.was_pressed(KeyCode::Escape) {
        app.exit();
    }
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    println!("------being");
    // create a renderer object
    let mut renderer = gfx.create_renderer();

    // define a color to use as clear
    let clear = ClearOptions::color(Color::from_rgb(0.2, 0.3, 0.3));

    // draw triangles using the same pipeline but different vertex buffer object
    renderer.begin(Some(&clear));
    renderer.set_pipeline(&state.pipeline);

    // draw first triangle using the first vbo
    renderer.bind_buffer(&state.vbo1);
    renderer.draw(0, 3);

    // draw second triangle using the second vbo
    renderer.bind_buffer(&state.vbo2);
    renderer.draw(0, 3);

    renderer.end();

    // render to the screen
    gfx.render(&renderer);

    println!("------end");
}
