use notan::prelude::*;

#[notan_main]
fn main() -> Result<(), String> {
    // initialize notan passing the update function
    notan::init().update(update).build()
}

fn update(app: &mut App) {
    // if esc is pressed close the app
    if app.keyboard.was_pressed(KeyCode::Escape) {
        app.exit();
    }
}
