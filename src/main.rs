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
}
/// The length of a side of the board.
const BOARD_SIZE: usize = 10;
/**
 * The implementation  of a board state.
 */
#[derive(PartialEq, Eq)]
struct BoardState {
    /// Stores the board in a linear array
    dots: [BoardColor; BOARD_SIZE * BOARD_SIZE],
    /// The trail of stuff.
    trail: Vec<(usize, usize)>,
}

impl BoardState {
    pub fn new() -> Self {
        let mut r: BoardState = BoardState {
            dots: [BoardColor::Green; BOARD_SIZE * BOARD_SIZE],
            trail: Vec::new(),
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
     * Check if the next position can be connected.
     */
    pub fn can_connect(self: &Self, x: usize, y: usize) -> bool {
        //Check if the trail is empty
        if let Some((old_x, old_y)) = self.trail.last() {
            //If it isn't, check if the next coordinates are adjacent to the last position
            if (x as i32 - *old_x as i32).abs() + (y as i32 - *old_y as i32).abs() != 1 {
                return false;
            }
            // The second to last position in the list must not be the same as x,y
            if self.trail.len() >= 2 && self.trail[self.trail.len() - 2] == (x, y) {
                return false;
            }
            // If the position is valid, the colors at each position must be the same
            let color_old = self.dots[Self::index(*old_x, *old_y)];
            let color_new = self.dots[Self::index(x, y)];
            color_new == color_old
        } else {
            // It's always valid to start a trail :)
            true
        }
    }
    /// Detect if the trail has a loop in it(a good thing)
    pub fn has_loop(self: &Self) -> bool {
        for (b, x) in self.trail.iter().enumerate() {
            for (a, y) in self.trail[b..].iter().enumerate() {
                if a == b {
                    continue;
                }
                if *x == *y {
                    return true;
                }
            }
        }
        false
    }
    pub fn drop_remaining(self: &mut Self) {
        for x in 0..BOARD_SIZE {
            let mut found_empty = true;
            while (found_empty) {
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
        if self.has_loop() {
            self.dots
                .iter_mut()
                .filter(|f| **f == trail_color)
                .for_each(|f| {
                    count += 1;
                    *f = BoardColor::Empty;
                });
        } else {
            self.trail
                .iter()
                .map(|x| Self::index(x.0, x.1))
                .for_each(|x| {
                    self.dots[x] = BoardColor::Empty;
                    count += 1;
                });
        }
        self.trail.clear();
        self.drop_remaining();
        count
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

fn main() {
    println!("Hello, world!");
    // orbtk::initialize();
    Application::new()
        .window(|ctx| {
            Window::new()
                .title("Henlo")
                .position((100.0, 100.0))
                .size(420.0, 420.0)
                .child(
                    Button::new()
                        .text("heh")
                        .on_click(|a, b| {
                            println!("hi");
                            true
                        })
                        .build(ctx),
                )
                .child(Canvas::new().build(ctx))
                .build(ctx)
        })
        .run();
}
