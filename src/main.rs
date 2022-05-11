use orbtk::prelude::*;
use rand::{thread_rng, Rng};

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
            BoardColor::Blue => "#00AAFF",
            BoardColor::Green => "#00FF00",
            BoardColor::Yellow => "#FFFF00",
            BoardColor::Empty => "#000000",
        }
    }
    pub fn get_glyph(&self) -> char {
        match *self {
            BoardColor::Red => 'R',
            BoardColor::Blue => 'B',
            BoardColor::Green => 'G',
            BoardColor::Yellow => 'Y',
            BoardColor::Empty => 'E',
        }
    }
}
/// The length of a side of the board.
const BOARD_SIZE: usize = 10;

#[derive(PartialEq, Eq, Clone, Copy)]
struct DotAction {
    x: usize,
    y: usize,
}

/**
The implementation  of a board state.
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
    moves_left: usize,
    moused_over: Option<Entity>,
}
///The number of moves that are allowed to be played in a single game.
const MOVE_LIMIT: usize = 30;

impl BoardState {
    pub fn new() -> Self {
        let mut r: BoardState = BoardState {
            dots: [BoardColor::Green; BOARD_SIZE * BOARD_SIZE],
            trail: Vec::new(),
            board_widgets: Vec::new(),
            action: None,
            score_label: None,
            score: 0,
            moves_left: MOVE_LIMIT,
            moused_over: None,
        };
        for i in 0..BOARD_SIZE * BOARD_SIZE {
            r.dots[i] = BoardColor::random();
        }
        r
    }
    /**
    Calculate the effective index for the position on the board
    */
    pub fn index(x: usize, y: usize) -> usize {
        y * BOARD_SIZE + x
    }

    /**
    Check if the next position can be connected.
    */
    pub fn can_connect(&self, x: usize, y: usize) -> bool {
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
                color_new == color_old
            } else {
                false
            }
        } else {
            // It's always valid to start a trail :)
            true
        }
    }
    /// Detect if the trail has a loop in it(a good thing)
    pub fn has_loop(&self) -> bool {
        for (b, x) in self.trail.iter().enumerate() {
            if self.trail[b + 1..].contains(x) {
                return true;
            }
        }
        false
    }
    /**
     Fill a column which all empty slots have been moved upwards.

     This rerolls some number of times if the newly placed dot would create a trivial cycle

     Returns true if the column is able to be filled and is filled. Otherwise returns false
     if more sorting is needed.
    */
    fn fill_column(&mut self, x: usize) -> bool {
        let mut first_empty: usize = BOARD_SIZE - 1;
        let mut seen_non_empty = false;
        for y in (0..BOARD_SIZE).rev() {
            if self.dots[Self::index(x, y)] == BoardColor::Empty && !seen_non_empty {
                first_empty = y;
            } else if seen_non_empty && self.dots[Self::index(x, y)] == BoardColor::Empty {
                return false;
            } else {
                seen_non_empty = true;
            }
        }
        for y in first_empty..BOARD_SIZE {
            self.dots[Self::index(x, y)] = BoardColor::random();
            if y == 0 {
                continue;
            }
            /*
             * Check adjacent squares to see if we can prevent a cycle from being too easy to make
             * 1S5
             * 234
             *
             * Roll again if S==1==2==3 or S==5==4==3
             */

            //This is the 'some number of times'
            let total_rolls = 3;
            for roll in 0..total_rolls {
                let mut roll_again = false;
                let i3 = Self::index(x, y - 1);
                if x > 1 {
                    let i1 = Self::index(x - 1, y);
                    let i2 = Self::index(x - 1, y - 1);
                    roll_again |= self.dots[i1] == self.dots[Self::index(x, y)]
                        && self.dots[i1] == self.dots[i2]
                        && self.dots[i1] == self.dots[i3];
                }
                if x < BOARD_SIZE - 1 {
                    let i4 = Self::index(x + 1, y - 1);
                    let i5 = Self::index(x + 1, y);
                    roll_again |= self.dots[i3] == self.dots[Self::index(x, y)]
                        && self.dots[i3] == self.dots[i4]
                        && self.dots[i3] == self.dots[i5];
                }
                if roll_again {
                    println!("Rerolling! {}", roll);
                    self.dots[Self::index(x, y)] = BoardColor::random();
                }
            }
        }

        true
    }
    /**
     * Remove empty cells by moving them up the board and then
     * replacing them
     */
    pub fn drop_remaining(&mut self) {
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
                if self.fill_column(x) {
                    break;
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
    pub fn finish_trail(&mut self) -> usize {
        if self.trail.len() < 2 {
            return 0;
        }
        let mut count: usize = 0;
        let trail_color = self.dots[Self::index(self.trail[0].0, self.trail[0].1)];
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
    pub fn handle_click(&mut self, x: usize, y: usize) {
        if self.moves_left == 0 {
            return;
        }
        if !self.trail.is_empty() {
            let pos = self.trail.last().expect("This shouldn't happen");
            if self.trail.len() >= 2 && pos.0 == x && pos.1 == y {
                self.score += self.finish_trail();
                self.trail.clear();
                self.moves_left -= 1;
            } else if self.can_connect(x, y) {
                self.trail.push((x, y));
            } else {
                self.trail.clear();
                println!("Cannot connect");
            }
        } else {
            self.trail.push((x, y))
        }
    }

    pub fn reset(&mut self) {
        self.dots.iter_mut().for_each(|b| {
            *b = BoardColor::random();
        });
        self.score = 0;
        self.trail.clear();
        self.moves_left = MOVE_LIMIT;
    }
}
impl Default for BoardState {
    fn default() -> Self {
        BoardState::new()
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
    fn init(&mut self, _reg: &mut Registry, ctx: &mut Context) {
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
        self.update(_reg, ctx);
    }
    fn update(&mut self, _reg: &mut Registry, ctx: &mut Context) {
        if let Some(ac) = self.action {
            self.handle_click(ac.x, ac.y);
            self.action = None;
        }
        let mut score_label = ctx.get_widget(self.score_label.expect("Failed to find label"));
        let text = score_label.get_mut::<String>("text");
        *text = format!("Score: {}, moves left: {}", self.score, self.moves_left);
        for x in 0..BOARD_SIZE {
            for y in 0..BOARD_SIZE {
                let idx = Self::index(x, y);
                let en = self.board_widgets[idx];

                let bc = self.dots[idx];
                ctx.get_widget(en).set("background", bc.get_brush());
                let mut text: String = format!("{}", bc.get_glyph());
                let id = ctx.get_widget(en).get::<String>("id").clone();
                if self
                    .trail
                    .iter()
                    .any(|&(x, y)| format!("{}x{}", x, y) == *id)
                {
                    if *self.trail.last().expect("") == (x, y) {
                        text = format!("{}(H)", text);
                        // ctx.get_widget(en).set("text", "HERE".to_string());
                        ctx.get_widget(en).set("border_width", Thickness::from(2.0));
                        ctx.get_widget(en)
                            .set("border_brush", Brush::from("#ffffff"));
                    } else {
                        text = format!("{}(T)", text);
                        // ctx.get_widget(en).set("text", "trail".to_string());
                        ctx.get_widget(en).set("border_width", Thickness::from(1.0));
                        ctx.get_widget(en)
                            .set("border_brush", Brush::from("#ff00ff"));
                    }
                } else if self.can_connect(x, y) {
                    text = format!("{}(A)", text);
                    // ctx.get_widget(en).set("text", "sure".to_string());
                    // The orange outline should only be shown when the trail isn't trivial
                    ctx.get_widget(en).set(
                        "border_width",
                        Thickness::from(if !self.trail.is_empty() { 2.0 } else { 0.0 }),
                    );
                    ctx.get_widget(en)
                        .set("border_brush", Brush::from("#ffa500"));
                } else {
                    text = format!("{}(X)", text);
                    // ctx.get_widget(en).set("text", "no".to_string());
                    ctx.get_widget(en).set("border_width", Thickness::from(0.0));
                }

                ctx.get_widget(en).set("text", text);
            }
        }
        if let Some(button) = self.moused_over {
            ctx.get_widget(button)
                .set("border_brush", Brush::from("#00FAFA"));
        }
    }
}
/**
 * Generate a button for the board, with minimal styling, the size, etc.
 */
fn generate_board_button(x: usize, y: usize, id: Entity, _ctx: &mut BuildContext) -> Button {
    let button = Button::new()
        .id(format!("{}x{}", x, y))
        .text(format!("{} {}", x, y))
        .style("")
        .border_radius(15)
        .min_size(10, 30)
        .max_size(40, 40)
        .foreground(Brush::from("#000000"))
        .on_enter(move |ctx, _point| {
            let button_id = ctx.get_mut::<BoardState>(id).board_widgets[BoardState::index(x, y)];
            ctx.get_mut::<BoardState>(id).moused_over = Some(button_id);
        })
        .on_leave(move |ctx, _point| {
            ctx.get_mut::<BoardState>(id).moused_over = None;
        })
        .on_click(move |ctx, _a| {
            ctx.get_mut::<BoardState>(id).action = Some(DotAction { x, y });
            true
        });
    button
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
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                let button = generate_board_button(x, y, id, ctx);
                grid = grid.place(ctx, button, x, BOARD_SIZE - 1 - y);
            }
        }
        // Without another grid the grid of buttons is sized for the label and button,
        // which doesn't happen to look especially square
        let new_grid = Grid::new()
            .columns(Blocks::create().push(Block::create().size(BlockSize::Auto).build()))
            .rows(Blocks::create().repeat(Block::create().size(BlockSize::Auto).build(), 3))
            .place(ctx, grid, 0, 0)
            .place(ctx, TextBlock::new().id("score_label"), 0, 1)
            .place(
                ctx,
                Button::new()
                    .text("Reset")
                    .on_click(move |a, _b| {
                        a.get_mut::<BoardState>(id).reset();
                        true
                    })
                    .attach(Grid::column_span(1))
                    .h_align(Alignment::Center),
                0,
                2,
            );
        self.name("DotBoard").child(
            Container::new()
                .child(new_grid.v_align("stretch").h_align("stretch").build(ctx))
                .build(ctx),
        )
    }

    fn render_object(&self) -> Box<dyn RenderObject> {
        DefaultRenderObject.into()
    }

    fn layout(&self) -> Box<dyn Layout> {
        GridLayout::new().into()
    }
}
fn main() {
    Application::new()
        .window(|ctx| {
            Window::new()
                .title("Terrible dot game")
                .position((100.0, 100.0))
                .size(600.0, 600.0)
                .child(DotBoard::new().width(600).height(600).build(ctx))
                .build(ctx)
        })
        .run();
}
