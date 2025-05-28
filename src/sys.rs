
const TITLE: &str = "Sudoku Solver";
pub const TILE_SIZE: u32 = 30;
pub const LOGICAL_WIDTH: u32 = TILE_SIZE * 9;
pub const LOGICAL_HEIGHT: u32 = TILE_SIZE * 9;
pub const SCALE: u32 = 3;
pub const INT_SCALE: bool = true;
pub const BLEND_MODE: sdl2::render::BlendMode = sdl2::render::BlendMode::None;

const FONT_PATH: &str = "assets/minecraft.otf";
const FONT_SIZE: u16 = 20;

/// Contains the various components of SDL2 used by the game.
pub struct SdlContext {
    /// The SDL2 event loop for retrieving user inputs.
    pub events: sdl2::EventPump,
    /// The SDL2 canvas for rendering.
    pub canvas: sdl2::render::WindowCanvas,
    /// The SDL2 texture creator for rendering text and copying images.
    pub texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
}

/// A convenience function for initializing SDL2 with reasonable default settings.
pub fn init_sdl_systems(
    sdl: &sdl2::Sdl,
    video: &sdl2::VideoSubsystem,
) -> Result<SdlContext, Box<dyn std::error::Error>> {
    let events = sdl.event_pump()?;
    let window = video
        .window(TITLE, LOGICAL_WIDTH * SCALE, LOGICAL_HEIGHT * SCALE)
        .build()?;
    let mut canvas = window.into_canvas().build()?;
    canvas.set_logical_size(LOGICAL_WIDTH, LOGICAL_HEIGHT)?;
    canvas.set_integer_scale(INT_SCALE)?;
    canvas.set_blend_mode(BLEND_MODE);

    let texture_creator = canvas.texture_creator();

    Ok(SdlContext {
        events,
        canvas,
        texture_creator,
    })
}

/// Load the default font from the asset folder.
pub fn load_font(ttf: &sdl2::ttf::Sdl2TtfContext) -> Result<sdl2::ttf::Font<'_, 'static>, String> {
    ttf.load_font(FONT_PATH, FONT_SIZE)
}
