//! A snail creeps 2 metres up a wall during the daytime, then falls asleep. It wakes up the next
//! morning and discovers it slipped down 1 metre.
//!
//! If this fierce determination persists, how many days will it take to reach the top of a 10 metre wall?
//!
//! This simple problem can easily be solved analytically, provided one doesn't forget that once the snail
//! reaches the top, it will not slip again. This example uses a discrete-event simulation to solve the problem.
//! There are two event types: [`Climb`](climb::Climb) and [`Slip`](slip::Slip). `Climb` advances the snail's
//! progress by 2 units. If (and only if) it fails to complete the journey, it enqueues a subsequent `Slip`
//! event. Conversely, `Slip` pushes the snail back by one unit and enqueues a subsequent `Climb` event.
//! The simulation terminates when the event timeline is exhausted.
//!
//! The simulation [`State`] keeps track of the progress and the number of days the snail spent climbing. It is
//! bootstrapped with a single `Climb` event. We print the state after the conclusion of the simulation to
//! see how many days the poor bugger spent climbing.

use sequent::{Scenario, Simulation};
use crate::climb::Climb;

fn main() {
    let scenario = Scenario {
        initial: State::default(),
        timeline: vec![
            Box::new(Climb)
        ]
    };

    let mut simulation = Simulation::from(scenario);
    simulation.run().unwrap();

    println!("{:?}", simulation.current_state());
}

#[derive(Default, Clone, Debug)]
struct State {
    progress: u16,
    days: u16
}

const WALL_HEIGHT: u16 = 10;
const CLIMB_STEP: u16 = 2;
const SLIP_STEP: u16 = 1;

mod climb {
    use std::borrow::Cow;
    use std::fmt::Debug;
    use sequent::{Event, Named, Queue, TransitionError};
    use crate::{CLIMB_STEP, State, WALL_HEIGHT};
    use crate::slip::Slip;

    #[derive(Debug)]
    pub struct Climb;

    impl ToString for Climb {
        fn to_string(&self) -> String {
            "".into()
        }
    }

    impl Named for Climb {
        fn name(&self) -> Cow<'static, str> {
            "climb".into()
        }
    }

    impl Event<State> for Climb {
        fn apply(&self, state: &mut State, queue: &mut Queue<State>) -> Result<(), TransitionError> {
            state.progress += CLIMB_STEP;
            state.days += 1;
            if state.progress < WALL_HEIGHT {
                queue.push_later(Box::new(Slip));
            }
            Ok(())
        }
    }
}

mod slip {
    use std::borrow::Cow;
    use sequent::{Event, Named, Queue, TransitionError};
    use crate::{SLIP_STEP, State};
    use crate::climb::Climb;

    #[derive(Debug)]
    pub struct Slip;

    impl ToString for Slip {
        fn to_string(&self) -> String {
            "".into()
        }
    }

    impl Named for Slip {
        fn name(&self) -> Cow<'static, str> {
            "slip".into()
        }
    }

    impl Event<State> for Slip {
        fn apply(&self, state: &mut State, queue: &mut Queue<State>) -> Result<(), TransitionError> {
            state.progress -= SLIP_STEP;
            queue.push_later(Box::new(Climb));
            Ok(())
        }
    }
}