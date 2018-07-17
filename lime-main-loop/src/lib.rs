extern crate failure;
#[macro_use]
extern crate log;
extern crate winit;

mod action;
mod ticker;

use std::collections::BinaryHeap;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use winit::{ControlFlow, Event, EventsLoop, EventsLoopProxy};

use self::action::{Action, QueuedAction};
use self::ticker::Ticker;

const SECOND: Duration = Duration::from_secs(1);

pub trait App {
    const UPDATES_PER_SECOND: u32;
    const RENDERS_PER_SECOND: u32;

    fn update(&mut self, dt: Duration);
    fn render(&mut self, dt: Duration);
    fn event(&mut self, ev: Event) -> ControlFlow;
}

pub fn run<A, F>(build: F)
where
    A: App,
    F: FnOnce(&EventsLoop) -> A,
{
    let mut events_loop = EventsLoop::new();
    let mut app = build(&events_loop);

    let (tx, rx) = mpsc::channel();
    let proxy = events_loop.create_proxy();
    thread::spawn(move || wakeup::<A>(proxy, tx));

    let mut update_ticker = Ticker::new();
    let mut render_ticker = Ticker::new();
    events_loop.run_forever(|event| match event {
        Event::Awakened => {
            while let Ok(QueuedAction(action, deadline)) = rx.try_recv() {
                if deadline < Instant::now() {
                    trace!("Skipping action {:?}.", action);
                } else {
                    match action {
                        Action::Update => app.update(update_ticker.tick()),
                        Action::Render => app.render(render_ticker.tick()),
                        Action::Log => {
                            info!("Updates per second: {}.", update_ticker.split());
                            info!("Renders per second: {}.", render_ticker.split());
                        }
                    }
                }
            }
            ControlFlow::Continue
        }
        event => app.event(event),
    })
}

fn wakeup<A: App>(proxy: EventsLoopProxy, tx: mpsc::Sender<QueuedAction>) {
    let intervals: [Duration; Action::COUNT] = [
        SECOND / A::UPDATES_PER_SECOND,
        SECOND / A::RENDERS_PER_SECOND,
        SECOND,
    ];

    let mut heap: BinaryHeap<_> = Action::values().map(QueuedAction::new).collect();

    loop {
        let QueuedAction(action, time) = heap.pop().unwrap();
        let next = QueuedAction(action, time + intervals[action as usize]);
        heap.push(next);

        let now = Instant::now();
        if now < time {
            thread::sleep(time - now);
        }

        if tx.send(next).is_err() || proxy.wakeup().is_err() {
            return;
        }
    }
}
