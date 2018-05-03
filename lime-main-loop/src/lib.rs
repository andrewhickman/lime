extern crate failure;
#[macro_use]
extern crate log;
extern crate winit;

mod action;
mod ticker;

use std::collections::BinaryHeap;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{cmp, thread};

use winit::{ControlFlow, DeviceEvent, Event, EventsLoop, EventsLoopProxy, WindowEvent};

use self::action::Action;
use self::ticker::Ticker;

pub trait App: Sized {
    const UPDATES_PER_SECOND: u32;
    const RENDERS_PER_SECOND: u32;

    fn update(&mut self, dt: Duration);
    fn render(&mut self, dt: Duration);
    fn window_event(&mut self, event: WindowEvent) -> ControlFlow;
    fn device_event(&mut self, event: DeviceEvent) -> ControlFlow;
}

pub fn run<A, F>(build: F)
where
    A: App,
    F: FnOnce(&EventsLoop) -> A,
{
    let mut events_loop = EventsLoop::new();
    let mut app = build(&events_loop);

    const SECOND: Duration = Duration::from_secs(1);
    let intervals: [Duration; Action::COUNT] = [
        SECOND / A::UPDATES_PER_SECOND,
        SECOND / A::RENDERS_PER_SECOND,
        SECOND,
    ];

    let (tx, rx) = mpsc::channel();
    let proxy = events_loop.create_proxy();
    thread::spawn(move || wakeup(proxy, tx, &intervals));

    let mut update_ticker = Ticker::new();
    let mut render_ticker = Ticker::new();
    events_loop.run_forever(|event| match event {
        Event::WindowEvent { event, .. } => app.window_event(event),
        Event::DeviceEvent { event, .. } => app.device_event(event),
        Event::Awakened => {
            let (deadline, action) = rx.recv().unwrap();
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
            ControlFlow::Continue
        }
        Event::Suspended(_) => ControlFlow::Continue,
    })
}

fn wakeup(
    proxy: EventsLoopProxy,
    tx: mpsc::Sender<(Instant, Action)>,
    intervals: &[Duration; Action::COUNT],
) {
    if intervals.len() > 0 {
        let mut heap: BinaryHeap<_> = Action::values()
            .map(|action| (Instant::now(), action))
            .map(cmp::Reverse)
            .collect();

        loop {
            let cmp::Reverse((time, action)) = heap.pop().unwrap();
            let next = time + intervals[action as usize];
            heap.push(cmp::Reverse((next, action)));

            let now = Instant::now();
            if now < time {
                thread::sleep(time - now);
            }

            if let Err(_) = tx.send((next, action)) {
                return;
            }
            if let Err(_) = proxy.wakeup() {
                return;
            }
        }
    }
}
