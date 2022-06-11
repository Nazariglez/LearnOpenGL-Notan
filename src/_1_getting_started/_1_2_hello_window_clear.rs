use notan::prelude::*;

#[notan_main]
fn main() -> Result<(), String> {
    notan::init()
        // pass the update function
        .update(update)
        // pass the draw function
        .draw(draw)
        .build()
}

fn update(app: &mut App) {
    // if esc is pressed close the app
    if app.keyboard.was_pressed(KeyCode::Escape) {
        app.exit();
    }
}

fn draw(gfx: &mut Graphics) {
    // create a renderer object
    let mut renderer = gfx.create_renderer();

    // define a color to use as clear
    let clear = ClearOptions::color(Color::from_rgb(0.2, 0.3, 0.3));

    // begin the pass
    renderer.begin(Some(&clear));

    // end the pass (we only want clear)
    renderer.end();

    // render to the screen
    gfx.render(&renderer);
}
