use orbtk::prelude::*;
use rand::{thread_rng, Rng};
use std::{
    borrow::{Borrow, BorrowMut},
    cell::Cell,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum BoardColor {
    Red,
    Blue,
    Green,
    Yellow,
    Empty,
}
impl BoardColor {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let n: i32 = rng.gen_range(0..4);
        match n {
            0 => BoardColor::Red,
            1 => BoardColor::Blue,
            2 => BoardColor::Green,
            3 => BoardColor::Yellow,
            _ => panic!("Got an invalid random number??"),
        }
    }
    pub fn get_brush(&self) -> Brush {
        Brush::from(self.get_color())
    }
    pub fn get_color(&self) -> &str {
        match *self {
            BoardColor::Red => "#FF0000",
            BoardColor::Blue => "#0000FF",
            BoardColor::Green => "#00FF00",
            BoardColor::Yellow => "#FFFF00",
            BoardColor::Empty => "#000000",
        }
    }
}
/// The length of a side of the board.
const BOARD_SIZE: usize = 10;

//Keys
static ID_CANVAS_DOTS: &str = "dot_canvas";

#[derive(PartialEq, Eq, Clone, Copy)]
struct DotAction {
    x: usize,
    y: usize,
}

/**
 * The implementation  of a board state.
 */
#[derive(PartialEq, Eq, AsAny)]
struct BoardState {
    /// Stores the board in a linear array
    dots: [BoardColor; BOARD_SIZE * BOARD_SIZE],
    /// The trail of 'dots' selected.
    trail: Vec<(usize, usize)>,
    board_widgets: Vec<Entity>,
    score_label: Option<Entity>,
    action: Option<DotAction>,
    score: usize,
}

impl BoardState {
    pub fn new() -> Self {
        let mut r: BoardState = BoardState {
            dots: [BoardColor::Green; BOARD_SIZE * BOARD_SIZE],
            trail: Vec::new(),
            board_widgets: Vec::new(),
            action: None,
            score_label: None,
            score: 0,
        };
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            r.dots[i] = BoardColor::random();
        }
        r
    }
    /**
     * Calculate the effective index for the position on the board
     */
    pub fn index(x: usize, y: usize) -> usize {
        y * BOARD_SIZE + x
    }
    /**
     * Set the dot's color at a given position
     */
    pub fn set_position(self: &mut Self, x: usize, y: usize, color: BoardColor) {
        self.dots[Self::index(x, y)] = color;
    }
    /**
     * Check if the next position can be connected.
     */
    pub fn can_connect(self: &Self, x: usize, y: usize) -> bool {
        //Check if the trail is empty
        if let Some((old_x, old_y)) = self.trail.last() {
            //If it isn't, check if the next coordinates are adjacent to the last position
            let dx = (x as i32) - (*old_x as i32);
            let dy = (y as i32) - (*old_y as i32);
            if dx.abs() + dy.abs() == 1 {
                // The second to last position in the list must not be the same as x,y
                if self.trail.len() >= 2 && self.trail[self.trail.len() - 2] == (x, y) {
                    return false;
                }
                // If the position is valid, the colors at each position must be the same
                let color_old = self.dots[Self::index(*old_x, *old_y)];
                let color_new = self.dots[Self::index(x, y)];
                // println!("{:?} == {:?}", color_old, color_new);
                color_new == color_old
            } else {
                // println!("Can't connect because of distance");
                false
            }
        } else {
            // It's always valid to start a trail :)
            true
        }
    }
    /// Detect if the trail has a loop in it(a good thing)
    pub fn has_loop(self: &Self) -> bool {
        for (b, x) in self.trail.iter().enumerate() {
            if self.trail[b+1..].contains(x){
                return true;
            }
        }
        false
    }
    /**
     * Remove empty cells by moving them up the board and then
     * replacing them
     */
    pub fn drop_remaining(self: &mut Self) {
        for x in 0..BOARD_SIZE {
            let mut found_empty = true;
            while found_empty {
                found_empty = false;
                for y in 0..BOARD_SIZE - 1 {
                    let col = self.dots[Self::index(x, y)];
                    if col == BoardColor::Empty {
                        self.dots.swap(Self::index(x, y), Self::index(x, y + 1));
                        found_empty = true;
                    }
                }
                if self.dots[Self::index(x, BOARD_SIZE - 1)] == BoardColor::Empty {
                    self.dots[Self::index(x, BOARD_SIZE - 1)] = BoardColor::random();
                }
            }
        }
    }
    /**
     * Clear the dots that have been matched, resets the trail, and whatever.
     *
     * Returns the number of dots cleared.
     *
     */
    pub fn finish_trail(self: &mut Self) -> usize {
        if self.trail.len() < 2 {
            return 0;
        }
        let mut count: usize = 0;
        let trail_color = self.dots[Self::index(self.trail[0].0, self.trail[0].1)].clone();
        //If the trail has a loop, clear the board of the color of the loop :)
        if self.has_loop() {
            self.dots
                .iter_mut()
                .filter(|f| **f == trail_color)
                .for_each(|f| {
                    count += 1;
                    *f = BoardColor::Empty;
                });
        } else {
            //Otherwise just remove the stuff in the trail.
            self.trail
                .iter()
                .map(|x| Self::index(x.0, x.1)) //Convert each position into an index.
                .for_each(|x| {
                    //Set each position to be empty
                    self.dots[x] = BoardColor::Empty;
                    count += 1;
                });
        }
        self.trail.clear();
        self.drop_remaining();
        count
    }

    pub fn print(self: &Self) {
        self.dots
            .chunks(BOARD_SIZE) //Operate on rows
            .rev() //Work with the topmost row being the last row
            .map(|x| {
                x.iter().map(|x| match *x {
                    //It's probably not a good idea to do the escapes this way,
                    // but like, this is for debugging right now
                    BoardColor::Red => "\x1b[31mR",
                    BoardColor::Blue => "\x1b[34mB",
                    BoardColor::Green => "\x1b[32mG",
                    BoardColor::Yellow => "\x1b[33mY",
                    BoardColor::Empty => "\x1b[30mX",
                })
            })
            .for_each(|x| {
                let z = x.collect::<String>();
                println!("{}", z);
            });
    }
    pub fn handle_click(self: &mut Self, x: usize, y: usize) {
        if self.trail.len() > 0 {
            let pos = self.trail.last().expect("This shouldn't happen");
            if self.trail.len() >= 2 && pos.0 == x && pos.1 == y {
                self.score += self.finish_trail();
                self.trail.clear();
            } else {
                if self.can_connect(x, y) {
                    self.trail.push((x, y));
                } else {
                    self.trail.clear();
                    println!("Cannot connect");
                }
            }
        } else {
            self.trail.push((x, y))
        }
    }

    pub fn reset(self: &mut Self) {
        self.dots.iter_mut().for_each(|b| {
            *b = BoardColor::random();
        });
        self.score = 0;
        self.trail.clear();
    }
}
impl Default for BoardState {
    fn default() -> Self {
        BoardState::new()
    }
}

struct DotBoardRenderPipeline {
    data: Cell<[BoardColor; BOARD_SIZE * BOARD_SIZE]>,
}
impl RenderPipeline for DotBoardRenderPipeline {
    fn draw(self: &Self, target: &mut RenderTarget) {
        let thing = self.data.get();
        let width = target.width() / (BOARD_SIZE as f64);
        let height = target.height() / (BOARD_SIZE as f64);
        let mut ctx = RenderContext2D::new(target.width(), target.height());

        thing
            .iter()
            .enumerate()
            .map(|(idx, color)| ((idx % BOARD_SIZE, idx / BOARD_SIZE), color))
            .map(|((x, y), color)| ((x as f64 * width, y as f64 * height), color))
            .map(|(position, color)| {
                let col = match *color {
                    BoardColor::Red => "#FF0000",
                    BoardColor::Blue => "#0000FF",
                    BoardColor::Yellow => "#FFFF00",
                    BoardColor::Green => "#00FF00",
                    BoardColor::Empty => "#252525",
                };
                (position, col)
            })
            .for_each(|(position, color)| {
                ctx.set_fill_style(Brush::from(color));
                ctx.fill_rect(position.0, position.1, width, height);
            })
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    #[test]
    pub fn test_drop() {
        let mut board = BoardState::new();
        board.trail.push((0, 0));
        board.trail.push((0, 1));
        board.trail.push((1, 1));
        board.trail.push((1, 0));
        board.trail.push((0, 0));
        assert!(board.has_loop());
        assert_ne!(board.finish_trail(), 0);
    }
}

widget!(DotBoard<BoardState>: MouseHandler {
    score: u32,
    pipeline:DefaultRenderPipeline
});

impl State for BoardState {
    fn init(self: &mut Self, _reg: &mut Registry, ctx: &mut Context) {
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                let thing = format!("{}x{}", x, y);
                self.board_widgets.push(
                    ctx.entity_of_child(thing.as_str())
                        .expect("Button not found!?!?!"),
                );
            }
        }
        self.score_label = Some(
            ctx.entity_of_child("score_label")
                .expect("Couldn't find score label"),
        );
        self.update(_reg,ctx);
    }
    fn update(self: &mut Self, _reg: &mut Registry, ctx: &mut Context) {
        if let Some(ac) = self.action {
            self.handle_click(ac.x, ac.y);
            self.action = None;
        }
        let mut score_label = ctx.get_widget(self.score_label.expect("Failed to find label"));
        let text = score_label.get_mut::<String>("text");
        *text = format!("Score: {}", self.score);
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let idx = Self::index(x, y);
                let en = self.board_widgets[idx];
                let bc = self.dots[idx];
                ctx.get_widget(en).set("background", bc.get_brush());
                let id = ctx.get_widget(en).get::<String>("id").clone();
                if self
                    .trail
                    .iter()
                    .any(|&(x, y)| format!("{}x{}", x, y) == *id)
                {
                    if *self.trail.last().expect("") == (x, y) {
                        ctx.get_widget(en).set("text", "HERE".to_string());
                    } else {
                        ctx.get_widget(en).set("text", "trail".to_string());
                    }
                } else {
                    if self.can_connect(x, y) {
                        ctx.get_widget(en).set("text", "sure".to_string());
                    } else {
                        ctx.get_widget(en).set("text", "no".to_string());
                    }
                }
            }
        }
    }
}
fn generate_rectangle(x: usize, y: usize, id: Entity, ctx: &mut BuildContext) -> Button {
    println!("Adding button at {}, {}", x, y);
    let button = Button::new()
        .id(format!("{}x{}", x, y))
        .text(format!("{} {}", x, y))
        .min_size(0, 0)
        .padding(1)
        .style("")
        .on_click(move |ctx, a| {
            ctx.get_mut::<BoardState>(id).action = Some(DotAction { x, y });
            true
        });
    button
    // button.build(ctx)
}
impl Template for DotBoard {
    fn template(self, id: Entity, ctx: &mut BuildContext<'_>) -> Self {
        let mut grid = Grid::new();
        // Is this actually how I have to do this?
        let columns =
            Blocks::create().repeat(Block::create().size(BlockSize::Auto).build(), BOARD_SIZE);
        let rows = Blocks::create().repeat(
            Block::create().size(BlockSize::Auto).build(),
            BOARD_SIZE + 1,
        );
        grid = grid.columns(columns).rows(rows);
        // for i in 0..BOARD_SIZE * BOARD_SIZE {
        //     let x = i % BOARD_SIZE;
        //     let y = i / BOARD_SIZE;
        //     let button = generate_rectangle(x, y, id, ctx);
        //     grid = grid.place(ctx, button, x, y);
        // }
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                let button = generate_rectangle(x, y, id, ctx);
                // grid = grid.place(ctx, button, x, y);
                grid = grid.place(ctx, button, x, BOARD_SIZE - 1 - y);
            }
        }
        let id2 = id;
        grid = grid
            .place(ctx, TextBlock::new().id("score_label"), 0, BOARD_SIZE)
            .place(
                ctx,
                Button::new()
                    .text("Reset")
                    .on_click(move |a, b| {
                        a.get_mut::<BoardState>(id2).reset();
                        true
                    })
                    .attach(Grid::column_span(3)),
                1,
                BOARD_SIZE,
            );
        self.name("DotBoard")
            // .v_align("stretch")
            // .h_align("stretch")
            .child(
                Container::new()
                    .child(grid.v_align("stretch").h_align("stretch").build(ctx))
                    .build(ctx),
            )
    }
    // fn layout(&self) -> Box<dyn Layout> {
    //     Box::new(PaddingLayout::new())
    // }
}
impl RenderPipeline for DotBoard {
    fn draw(self: &Self, image: &mut RenderTarget) {
        let mut ctx = RenderContext2D::new(self.bounds.width(), self.bounds.height());
        ctx.draw_render_target(image, 0.0, 0.0);
    }
}
fn main() {
    println!("Hello, world!");
    Application::new()
        .window(|ctx| {
            Window::new()
                .title("Henlo")
                .position((100.0, 100.0))
                .size(600.0, 600.0)
                .child(DotBoard::new().width(600).height(600).build(ctx))
                .build(ctx)
        })
        .run();
}
