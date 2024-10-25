use {
    crossbeam::channel::{
        Receiver,
        bounded,
    },
    std::{
        sync::{
            Arc,
            Condvar,
            Mutex,
        },
        thread,
        time::Duration,
    },
};

/// The timer isn't really precise but should be immune to
/// changes on system's time.
pub struct Timer {
    pair: Arc<(Mutex<Option<TimerCommand>>, Condvar)>,
}

#[derive(Debug, Clone, Copy)]
enum TimerCommand {
    #[allow(dead_code)]
    RingNow,
    Reset,
    #[allow(dead_code)]
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerResult {
    CommandedRing,
    TimeoutRing,
    Stopped,
    Crash,
}

impl Timer {
    pub fn new(delay: Duration) -> (Self, Receiver<TimerResult>) {
        let cmd: Option<TimerCommand> = None;
        let pair = Arc::new((Mutex::new(cmd), Condvar::new()));
        let timer_pair = Arc::clone(&pair);
        let (tx_ring, rx_ring) = bounded(1);
        thread::spawn(move || {
            let (cmd, cvar) = &*timer_pair;
            let mut cmd = cmd.lock().unwrap();
            // we use the ring channel to notifiy the outside
            // that the thread is ready
            tx_ring.send(TimerResult::CommandedRing).unwrap();
            loop {
                match cvar.wait_timeout_while(cmd, delay, |cmd| cmd.is_none()) {
                    Ok((wcmd, wait_timeout_result)) => {
                        cmd = wcmd;
                        if wait_timeout_result.timed_out() {
                            tx_ring.send(TimerResult::TimeoutRing).unwrap();
                            break;
                        }
                        match *cmd {
                            Some(TimerCommand::RingNow) => {
                                tx_ring.send(TimerResult::CommandedRing).unwrap();
                                break;
                            }
                            Some(TimerCommand::Reset) => {
                                *cmd = None;
                            }
                            Some(TimerCommand::Stop) => {
                                tx_ring.send(TimerResult::Stopped).unwrap();
                                break;
                            }
                            None => {
                                warn!("unexpected lack of command in timer");
                                tx_ring.send(TimerResult::Crash).unwrap();
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("crash in timer: {}", e);
                        tx_ring.send(TimerResult::Crash).unwrap();
                        break;
                    }
                }
            }
        });
        rx_ring.recv().unwrap(); // we wait for the thread to be started
        (Self { pair }, rx_ring)
    }
    fn send(
        &self,
        timer_command: TimerCommand,
    ) {
        let _ = self.pair.0.lock().unwrap().insert(timer_command);
        self.pair.1.notify_all();
    }
    #[allow(dead_code)]
    pub fn stop(&self) {
        self.send(TimerCommand::Stop);
    }
    pub fn reset(&self) {
        self.send(TimerCommand::Reset);
    }
    #[allow(dead_code)]
    pub fn ring_now(&self) {
        self.send(TimerCommand::RingNow);
    }
}

#[cfg(test)]
mod timer_tests {

    use {
        super::*,
        std::time::{
            Duration,
            Instant,
        },
    };

    const MARGIN: Duration = Duration::from_millis(100);

    /// check that the uninterrupted timer rings after the required delay
    #[test]
    fn test_timer_timeout() {
        let delay = Duration::from_millis(10);
        let start = Instant::now();
        let (_, timer_rx) = Timer::new(delay);
        let res = timer_rx.recv();
        assert_eq!(res, Ok(TimerResult::TimeoutRing));
        assert!(start.elapsed() < delay + MARGIN); // we don't want it to be too long
        assert!(start.elapsed() >= delay);
    }

    /// check that the timer has been immediately stopped
    #[test]
    fn test_timer_stop() {
        let delay = Duration::from_secs(1);
        let start = Instant::now();
        let (timer, timer_rx) = Timer::new(delay);
        timer.stop();
        let res = timer_rx.recv();
        assert_eq!(res, Ok(TimerResult::Stopped));
        assert!(start.elapsed() < MARGIN);
    }

    /// check that the timer immediately rings
    #[test]
    fn test_timer_ring_now() {
        let delay = Duration::from_secs(1);
        let start = Instant::now();
        let (timer, timer_rx) = Timer::new(delay);
        timer.ring_now();
        let res = timer_rx.recv();
        assert_eq!(res, Ok(TimerResult::CommandedRing));
        assert!(start.elapsed() < MARGIN);
    }

    /// check that the timer has been reset before the timeout
    #[test]
    fn test_timer_reset() {
        let delay = Duration::from_millis(100);
        let start = Instant::now();
        let (timer, timer_rx) = Timer::new(delay);
        thread::spawn(move || {
            for _ in 0..5 {
                thread::sleep(delay / 2);
                timer.reset();
            }
        });
        let res = timer_rx.recv();
        assert_eq!(res, Ok(TimerResult::TimeoutRing));
        // we've reset 5 times after having waited for half the
        // delay, so the total duration should be more than
        // twice the delay
        assert!(start.elapsed() > 2 * delay);
        // but not too long
        assert!(start.elapsed() < 5 * delay);
    }
}
