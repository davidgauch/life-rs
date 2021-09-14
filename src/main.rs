#![feature(box_syntax, array_methods)]

use ggez::graphics::DrawParam;
use ggez::graphics::Drawable;
use ggez::graphics::FilterMode::Nearest;
use ggez::{conf, event, graphics, timer, Context, GameResult};
use rand::{self, Rng};
use rayon::prelude::*;
use std::time::{Duration, Instant};

const SCALE: usize = 2;
// Here we define the size of our game board in terms of how many grid cells.
const GRID_WIDTH: usize = 1920 / SCALE;
const GRID_HEIGHT: usize = 1080 / SCALE;
const GRID_CELLS: usize = GRID_WIDTH * GRID_HEIGHT;
const COLOR_BYTES: usize = 4;
const GRID_BYTES: usize = GRID_CELLS * COLOR_BYTES;

const WHITE: [u8; COLOR_BYTES] = [u8::MAX, u8::MAX, u8::MAX, u8::MAX];
const BLACK: [u8; COLOR_BYTES] = [u8::MIN, u8::MIN, u8::MIN, u8::MAX];

// Now we define the pixel size of each tile.

// Here we're defining how often we want our game to update. This will be
// important later so that we don't have our snake fly across the screen because
// it's moving a full tile every frame.
const UPDATES_PER_SECOND: u64 = 24;
// And we get the milliseconds of delay that this update rate corresponds to.
const MILLIS_PER_UPDATE: u64 = 1000 / UPDATES_PER_SECOND;

struct GameState {
    curr: Box<[[[u8; 4]; GRID_WIDTH]; GRID_HEIGHT]>,
    next: Box<[[[u8; 4]; GRID_WIDTH]; GRID_HEIGHT]>,
    last_update: Instant,
    needs_redraw: bool,
}

impl GameState {
    pub fn new() -> Self {
        let mut board = box [[BLACK; GRID_WIDTH]; GRID_HEIGHT];
        let mut rng = rand::thread_rng();

        for cell in board.iter_mut().flatten() {
            if rng.gen() {
                *cell = WHITE
            }
        }

        Self {
            curr: board,
            next: box [[BLACK; GRID_WIDTH]; GRID_HEIGHT],
            last_update: Instant::now(),
            needs_redraw: true,
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // First we check to see if enough time has elapsed since our last update based
        // on the update rate we defined at the top.
        // if not, we do nothing and return early.
        if !(Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE)) {
            return Ok(());
        }

        let curr = &self.curr;

        self.next.par_iter_mut().enumerate().for_each(|(r, row)| {
            for (c, v) in row.iter_mut().enumerate() {
                let (left, right, above, below) = (
                    (c + GRID_WIDTH - 1) % GRID_WIDTH,
                    (c + 1) % GRID_WIDTH,
                    (r + GRID_HEIGHT - 1) % GRID_HEIGHT,
                    (r + 1) % GRID_HEIGHT,
                );

                let neighbors: usize = IntoIterator::into_iter([
                    curr[above][left],
                    curr[above][c],
                    curr[above][right],
                    curr[r][left],
                    curr[r][right],
                    curr[below][left],
                    curr[below][c],
                    curr[below][right],
                ])
                .filter(|v| v[0] > 0)
                .count();

                if neighbors == 3 || (neighbors == 2 && curr[r][c][0] > 0) {
                    *v = WHITE;
                } else {
                    *v = BLACK;
                }
            }
        });

        std::mem::swap(&mut self.curr, &mut self.next);
        self.last_update = Instant::now();
        self.needs_redraw = true;

        Ok(())
    }
    /// draw is where we should actually render the game's current state.
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if self.needs_redraw {
            let curr: &[[[u8; COLOR_BYTES]; GRID_WIDTH]; GRID_HEIGHT] = self.curr.as_ref();
            let image_bytes: &[u8; GRID_BYTES] = unsafe { std::mem::transmute(curr) };

            let mut image = graphics::Image::from_rgba8(
                ctx,
                GRID_WIDTH as u16,
                GRID_HEIGHT as u16,
                image_bytes.as_slice(),
            )?;

            image.set_filter(Nearest);

            self.needs_redraw = false;
            image.draw(ctx, DrawParam::new())?;

            graphics::present(ctx)?;
        }
        // We yield the current thread until the next update
        timer::yield_now();
        // And return success.
        Ok(())
    }
}

fn main() -> GameResult {
    // Here we use a ContextBuilder to setup metadata about our game. First the title and author
    let (mut ctx, mut events_loop) = ggez::ContextBuilder::new("life", "Gray Olson")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(conf::WindowSetup::default().title("Life!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(
            conf::WindowMode::default()
                .dimensions(GRID_WIDTH as f32, GRID_HEIGHT as f32)
                .fullscreen_type(conf::FullscreenType::True),
        )
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    // And finally we actually run our game, passing in our context and state.
    event::run(&mut ctx, &mut events_loop, &mut GameState::new())
}
