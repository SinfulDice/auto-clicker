//! Logique pure de l'auto-clicker : états, compte à rebours, rythme des clics.
//! Rien ici ne lit l'horloge : on lui passe `now`, ce qui la rend testable.

use crate::clicker::Clicker;
use crate::config::{Config, MouseButton};
use std::time::{Duration, Instant};

pub const COUNTDOWN: Duration = Duration::from_secs(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartSource {
    Button,
    Hotkey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Stopped,
    CountingDown { until: Instant },
    Running,
}

/// R7 : intervalle entre deux clics = 1 / CPS.
pub fn interval(cps: u32) -> Duration {
    Duration::from_secs(1) / cps.max(1)
}

/// R8 : heure du clic suivant. Normalement `prévu + intervalle` (pas de dérive) ;
/// si on est déjà en retard, on repart de maintenant au lieu d'envoyer une rafale.
pub fn next_click_time(scheduled: Instant, interval: Duration, now: Instant) -> Instant {
    let next = scheduled + interval;
    if next <= now { now + interval } else { next }
}

/// R17 : texte de l'état affiché dans la fenêtre.
pub fn status_label(status: Status, now: Instant) -> String {
    match status {
        Status::Stopped => "Arrêté".to_string(),
        Status::CountingDown { until } => {
            let remaining = until.saturating_duration_since(now);
            let secs = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
            format!("Démarrage dans {} s", secs.max(1))
        }
        Status::Running => "En cours".to_string(),
    }
}

#[derive(Debug)]
pub struct Session {
    status: Status,
    button: MouseButton,
    interval: Duration,
    next_click: Instant,
    clicks: u64,
}

impl Session {
    pub fn new(now: Instant) -> Self {
        Self {
            status: Status::Stopped,
            button: MouseButton::Left,
            interval: interval(1),
            next_click: now,
            clicks: 0,
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    /// R18 : clics envoyés depuis le dernier démarrage.
    pub fn clicks(&self) -> u64 {
        self.clicks
    }

    /// R12 : bouton → compte à rebours de 3 s ; F6 → départ immédiat. R18 : compteur remis à 0.
    pub fn start(&mut self, source: StartSource, config: Config, now: Instant) {
        self.button = config.button;
        self.interval = interval(config.cps);
        self.clicks = 0;
        match source {
            StartSource::Button => {
                self.status = Status::CountingDown {
                    until: now + COUNTDOWN,
                }
            }
            StartSource::Hotkey => self.begin_clicking(now),
        }
    }

    /// R13/R14 : arrête tout, y compris un compte à rebours en cours.
    pub fn stop(&mut self) {
        self.status = Status::Stopped;
    }

    /// R10/R11 : Start si arrêté, Stop sinon.
    pub fn toggle(&mut self, source: StartSource, config: Config, now: Instant) {
        match self.status {
            Status::Stopped => self.start(source, config, now),
            _ => self.stop(),
        }
    }

    fn begin_clicking(&mut self, now: Instant) {
        self.status = Status::Running;
        self.next_click = now;
    }

    /// Fait avancer la session jusqu'à `now` : termine le compte à rebours, envoie
    /// le clic s'il est dû. Renvoie quand rappeler `poll` (`None` = rien à faire).
    pub fn poll(&mut self, now: Instant, clicker: &mut dyn Clicker) -> Option<Instant> {
        if let Status::CountingDown { until } = self.status {
            if now < until {
                return Some(until);
            }
            self.begin_clicking(until);
        }
        if self.status != Status::Running {
            return None;
        }
        if now >= self.next_click {
            clicker.click(self.button);
            self.clicks += 1;
            self.next_click = next_click_time(self.next_click, self.interval, now);
        }
        Some(self.next_click)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clicker::FakeClicker;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn cfg(button: MouseButton, cps: u32) -> Config {
        Config { button, cps }
    }

    /// Simule le fil d'exécution : appelle `poll` à chaque réveil demandé, jusqu'à `end`.
    fn run_until(session: &mut Session, clicker: &mut FakeClicker, mut now: Instant, end: Instant) {
        while let Some(wake) = session.poll(now, clicker) {
            if wake > end {
                break;
            }
            now = wake.max(now);
        }
    }

    #[test]
    fn r7_interval_is_one_over_cps() {
        assert_eq!(interval(1), ms(1000));
        assert_eq!(interval(10), ms(100));
        assert_eq!(interval(1000), ms(1));
        assert_eq!(interval(3), Duration::from_nanos(333_333_333));
    }

    #[test]
    fn r8_on_time_next_click_is_scheduled_plus_interval() {
        let t0 = Instant::now();
        // Le clic prévu à t0 part avec 0,3 ms de retard : le suivant reste à t0 + 1 ms.
        assert_eq!(
            next_click_time(t0, ms(1), t0 + Duration::from_micros(300)),
            t0 + ms(1)
        );
    }

    #[test]
    fn r8_when_late_no_burst_restart_from_now() {
        let t0 = Instant::now();
        let late = t0 + ms(50);
        assert_eq!(next_click_time(t0, ms(10), late), late + ms(10));
    }

    #[test]
    fn r8_rhythm_does_not_drift_over_one_second() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        let mut clicker = FakeClicker::default();
        session.start(StartSource::Hotkey, cfg(MouseButton::Left, 1000), t0);
        // Réveil toujours 200 µs en retard : le rythme ne doit pas glisser pour autant.
        let mut now = t0;
        while let Some(wake) = session.poll(now, &mut clicker) {
            if wake >= t0 + ms(1000) {
                break;
            }
            now = wake + Duration::from_micros(200);
        }
        // Clics prévus à 0, 1, 2 … 999 ms : exactement 1000 clics sur une seconde.
        assert_eq!(session.clicks(), 1000);
    }

    #[test]
    fn r5_r1_clicks_use_chosen_button() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        let mut clicker = FakeClicker::default();
        session.start(StartSource::Hotkey, cfg(MouseButton::Right, 10), t0);
        run_until(&mut session, &mut clicker, t0, t0 + ms(250));
        assert_eq!(clicker.clicks, vec![MouseButton::Right; 3]); // 0, 100, 200 ms
    }

    #[test]
    fn r9_keeps_clicking_until_stopped() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        let mut clicker = FakeClicker::default();
        session.start(StartSource::Hotkey, cfg(MouseButton::Left, 100), t0);
        run_until(&mut session, &mut clicker, t0, t0 + Duration::from_secs(60));
        assert_eq!(session.status(), Status::Running);
        assert_eq!(session.clicks(), 6001);
    }

    #[test]
    fn r12_button_start_waits_3s_before_first_click() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        let mut clicker = FakeClicker::default();
        session.start(StartSource::Button, cfg(MouseButton::Left, 10), t0);
        assert_eq!(session.status(), Status::CountingDown { until: t0 + ms(3000) });

        assert_eq!(session.poll(t0 + ms(2999), &mut clicker), Some(t0 + ms(3000)));
        assert!(clicker.clicks.is_empty());

        session.poll(t0 + ms(3000), &mut clicker);
        assert_eq!(session.status(), Status::Running);
        assert_eq!(clicker.clicks.len(), 1);
    }

    #[test]
    fn r12_hotkey_start_clicks_immediately() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        let mut clicker = FakeClicker::default();
        session.start(StartSource::Hotkey, cfg(MouseButton::Left, 10), t0);
        assert_eq!(session.status(), Status::Running);
        session.poll(t0, &mut clicker);
        assert_eq!(clicker.clicks.len(), 1);
    }

    #[test]
    fn r13_stop_during_countdown_sends_no_click() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        let mut clicker = FakeClicker::default();
        session.toggle(StartSource::Button, cfg(MouseButton::Left, 10), t0);
        session.toggle(StartSource::Hotkey, cfg(MouseButton::Left, 10), t0 + ms(1000));
        assert_eq!(session.status(), Status::Stopped);
        assert_eq!(session.poll(t0 + ms(10_000), &mut clicker), None);
        assert!(clicker.clicks.is_empty());
    }

    #[test]
    fn r14_no_click_after_stop() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        let mut clicker = FakeClicker::default();
        session.start(StartSource::Hotkey, cfg(MouseButton::Left, 10), t0);
        run_until(&mut session, &mut clicker, t0, t0 + ms(450));
        let before = clicker.clicks.len();
        session.stop();
        assert_eq!(session.poll(t0 + ms(5000), &mut clicker), None);
        assert_eq!(clicker.clicks.len(), before);
    }

    #[test]
    fn r10_r11_toggle_starts_then_stops() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        session.toggle(StartSource::Hotkey, Config::default(), t0);
        assert_eq!(session.status(), Status::Running);
        session.toggle(StartSource::Button, Config::default(), t0);
        assert_eq!(session.status(), Status::Stopped);
    }

    #[test]
    fn r18_counter_counts_and_resets_on_start() {
        let t0 = Instant::now();
        let mut session = Session::new(t0);
        let mut clicker = FakeClicker::default();
        session.start(StartSource::Hotkey, cfg(MouseButton::Left, 10), t0);
        run_until(&mut session, &mut clicker, t0, t0 + ms(950));
        assert_eq!(session.clicks(), 10);
        session.stop();
        assert_eq!(session.clicks(), 10, "le compteur reste visible après l'arrêt");
        session.start(StartSource::Button, cfg(MouseButton::Left, 10), t0 + ms(2000));
        assert_eq!(session.clicks(), 0);
    }

    #[test]
    fn r17_status_labels() {
        let t0 = Instant::now();
        assert_eq!(status_label(Status::Stopped, t0), "Arrêté");
        assert_eq!(status_label(Status::Running, t0), "En cours");
        let until = t0 + ms(3000);
        assert_eq!(
            status_label(Status::CountingDown { until }, t0),
            "Démarrage dans 3 s"
        );
        assert_eq!(
            status_label(Status::CountingDown { until }, t0 + ms(1),),
            "Démarrage dans 3 s"
        );
        assert_eq!(
            status_label(Status::CountingDown { until }, t0 + ms(1500)),
            "Démarrage dans 2 s"
        );
        assert_eq!(
            status_label(Status::CountingDown { until }, t0 + ms(2999)),
            "Démarrage dans 1 s"
        );
        assert_eq!(
            status_label(Status::CountingDown { until }, t0 + ms(3000)),
            "Démarrage dans 1 s"
        );
    }
}
