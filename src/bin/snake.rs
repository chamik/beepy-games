use beepy_display::{BeepyDisplay, bind_console, unbind_console};
use embedded_graphics::{
    draw_target::DrawTarget, geometry::{Point, Size}, image::Image, mono_font::{ascii::FONT_10X20, MonoTextStyle}, pixelcolor::BinaryColor, primitives::{Circle, Line, Primitive, PrimitiveStyle, Rectangle}, text::Text, Drawable
};
use evdev::{Device, FetchEventsSynced, InputEventKind, Key};
use rand::{thread_rng, Rng};
use tinybmp::Bmp;
use std::{cmp::max, collections::VecDeque, convert::Infallible, error, fs::File, io::Write, thread, time::Duration};
use std::sync::mpsc::{self, TryRecvError};

const WIDTH: i32 = 20;
const HEIGHT: i32 = 11;
const TICK: u64 = 600;

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct SnakeGame {
    snake_buffer: VecDeque<Point>,
    direction: Direction,
    apple: Point,
    score: usize,
    tick: u64,
}

impl SnakeGame {
    fn new() -> Self {
        let head = Point::new(WIDTH / 2, HEIGHT / 2);

        let mut snake_buffer = VecDeque::new();
        snake_buffer.push_front(head - Point::new(2, 0));
        snake_buffer.push_front(head - Point::new(1, 0));
        snake_buffer.push_front(head);

        Self {
            snake_buffer,
            direction: Direction::Right,
            apple: random_point(),
            score: 0,
            tick: TICK,
        }
    }

    fn turn(&mut self, dir: Direction) {
        match dir {
            Direction::Left if self.direction == Direction::Left => self.direction = Direction::Down,
            Direction::Left if self.direction == Direction::Right => self.direction = Direction::Up,
            Direction::Left if self.direction == Direction::Up => self.direction = Direction::Left,
            Direction::Left if self.direction == Direction::Down => self.direction = Direction::Right,
            Direction::Right if self.direction == Direction::Left => self.direction = Direction::Up,
            Direction::Right if self.direction == Direction::Right => self.direction = Direction::Down,
            Direction::Right if self.direction == Direction::Up => self.direction = Direction::Right,
            Direction::Right if self.direction == Direction::Down => self.direction = Direction::Left,
            _ => {}
        }
    }

    fn tick(&mut self) -> bool {

        let current_head = *self.snake_buffer.front().unwrap();
        let mut new_head = match self.direction {
            Direction::Up => current_head + Point::new(0, -1),
            Direction::Down => current_head + Point::new(0, 1),
            Direction::Left => current_head + Point::new(-1, 0),
            Direction::Right => current_head + Point::new(1, 0),
        };

        if new_head.x >= WIDTH {
            new_head.x = 0;
        } else if new_head.x < 0 {
            new_head.x = WIDTH - 1;
        }

        if new_head.y > HEIGHT {
            new_head.y = 1;
        } else if new_head.y < 1 {
            new_head.y = HEIGHT;
        }

        if self.snake_buffer.contains(&new_head) {
            return false;
        }

        if current_head == self.apple {
            self.apple = random_point();
            self.score += 1;
            self.tick = max(self.tick - 25, 90);
        } else {
            self.snake_buffer.pop_back();
        }

        self.snake_buffer.push_front(new_head);

        return true;
    }

    fn set_dir(&mut self, dir: Direction) {
        self.direction = dir;
    }
}

fn random_point() -> Point {
    Point::new(
        thread_rng().gen_range(0..WIDTH),
        thread_rng().gen_range(0..HEIGHT),
    ) + Point::new(0, 1)
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut device = Device::open("/dev/input/event0").expect("Unable to open the keyboard.");
    let mut display =
        BeepyDisplay::new("/dev/fb1".into()).expect("Unable to open the frame buffer.");
    let mut snake_game = SnakeGame::new();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            let events = device.fetch_events().unwrap();
            for k in handle_input(events) {
                tx.send(k).unwrap();
            }
            // Sleep for a short period to prevent busy waiting
            thread::sleep(Duration::from_millis(10));
        }
    });

    unbind_console()?;

    // todo: load/save best score?

    'game: loop {
        // handle input
        match rx.try_recv() {
            Ok(k) => match k {
                // todo: guard against going into yourself?
                Key::KEY_D | Key::KEY_LEFT  => snake_game.turn(Direction::Left),
                Key::KEY_J | Key::KEY_RIGHT => snake_game.turn(Direction::Right),
                Key::KEY_ESC => break 'game,
                _ => {},
            }
            Err(TryRecvError::Disconnected) => {
                println!("disconnected");
                break 'game;
            },
            Err(TryRecvError::Empty) => {},
        }
        

        // advance game
        if !snake_game.tick() {
            break 'game;
        };

        // draw the game
        draw(&mut display, &snake_game)?;

        // sleep for a while
        thread::sleep(Duration::from_millis(snake_game.tick));
    }

    bind_console()?;

    let score = snake_game.score;
    println!("\n\nGAME OVER\nScore: {score}");

    Ok(())
}

fn draw(display: &mut BeepyDisplay, game: &SnakeGame) -> Result<(), Infallible> {
    display.clear(BinaryColor::Off)?;

    let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    let fill = PrimitiveStyle::with_fill(BinaryColor::On);
    let character_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

    let score = game.score;
    let score_text = format!("SCORE: {score}");

    // status bar
    Line::new(Point::new(0, 19), Point::new(399, 19))
        .into_styled(thin_stroke)
        .draw(display)?;
    Text::new(&score_text, Point::new(10, 15), character_style)
        .draw(display)?;

    // apple
    let bmp_data = include_bytes!("../media/apple.bmp");
    let bmp_apple = Bmp::from_slice(bmp_data).unwrap();
    Image::new(&bmp_apple, game.apple * 20).draw(display)?;

    for c in &game.snake_buffer {
        Circle::new(*c * 20, 20)
            .into_styled(fill)
            .draw(display)?;
    }

    // for i in 1..game.snake_buffer.len() - 1 {
    //     Rectangle::new(game.snake_buffer[i] * 20, Size::new(20, 20))
    //         .into_styled(fill)
    //         .draw(display)?;
    // }

    // let tick = game.apple;
    // let debug = format!("APPL: {tick}");
    // Text::new(&debug, Point::new(200, 15), character_style)
    //     .draw(display)?;

    display.flush();

    Ok(())
}

fn handle_input<'a>(events: FetchEventsSynced<'a>) -> impl Iterator<Item = evdev::Key> + 'a {
    events.filter_map(|e| match e.kind() {
        InputEventKind::Key(key) if e.value() == 1 => Some(key),
        _ => None,
    })
}
