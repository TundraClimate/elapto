use elapto::*;
use std::io;

fn main() {
    let alloc = CanvasAllocator::new();

    let canvas = alloc
        .allocate(
            0,
            Shape::rect((0, 0), (63, 15)),
            Source {
                cell: HashCell::new(12),
            },
        )
        .unwrap();

    crossterm::execute!(
        io::stdout(),
        TuiInitialize::new()
            .enter_alternate()
            .enable_raw_mode()
            .hide_cursor()
    )
    .ok();

    let draws = (0..16)
        .map(|_| DrawCommand::Line(".".repeat(63)))
        .collect::<Vec<_>>();

    canvas.draw(&draws).ok();

    let draws = (0..16)
        .map(|_| DrawCommand::Line("H".repeat(63)))
        .chain(vec![DrawCommand::Clear {
            rows: 8,
            begin: 2,
            end: 61,
        }])
        .collect::<Vec<_>>();

    canvas
        .draw_with_excludes(
            &draws,
            &[Shape::rect((5, 5), (18, 10)), Shape::rect((7, 7), (33, 12))],
        )
        .ok();

    std::thread::sleep(std::time::Duration::from_secs(5));

    crossterm::execute!(io::stdout(), Restore::all()).ok();
}
