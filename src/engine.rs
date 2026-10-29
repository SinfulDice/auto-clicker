//! Fait tourner une `Session` dans un fil d'exécution (thread) dédié aux clics,
//! pour que l'interface reste fluide même à 1000 CPS.

use crate::clicker::Clicker;
use crate::config::Config;
use crate::session::{Session, StartSource, Status};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

struct Shared {
    session: Session,
    config: Config,
    error: Option<String>,
    shutdown: bool,
}

/// Poignée partagée entre l'interface, la touche F6 et le fil de clics.
#[derive(Clone)]
pub struct EngineHandle {
    inner: Arc<(Mutex<Shared>, Condvar)>,
}

impl EngineHandle {
    fn read<R>(&self, f: impl FnOnce(&Shared) -> R) -> R {
        f(&self.inner.0.lock().unwrap())
    }

    fn write<R>(&self, f: impl FnOnce(&mut Shared) -> R) -> R {
        let (lock, wake) = &*self.inner;
        let result = f(&mut lock.lock().unwrap());
        // Réveille le fil de clics : son prochain rendez-vous a peut-être changé.
        wake.notify_all();
        result
    }

    /// Réglages utilisés au prochain démarrage (par le bouton comme par F6).
    pub fn set_config(&self, config: Config) {
        self.write(|s| s.config = config);
    }

    pub fn toggle(&self, source: StartSource) {
        self.write(|s| s.session.toggle(source, s.config, Instant::now()));
    }

    pub fn status(&self) -> Status {
        self.read(|s| s.session.status())
    }

    pub fn clicks(&self) -> u64 {
        self.read(|s| s.session.clicks())
    }

    pub fn error(&self) -> Option<String> {
        self.read(|s| s.error.clone())
    }
}

/// Possède le fil de clics : le détruire l'arrête proprement (R15).
pub struct Engine {
    handle: EngineHandle,
    thread: Option<JoinHandle<()>>,
}

impl Engine {
    /// `make_clicker` est appelé dans le nouveau fil, car le cliqueur X11 y vit.
    pub fn spawn<F>(config: Config, make_clicker: F) -> Self
    where
        F: FnOnce() -> Result<Box<dyn Clicker>, String> + Send + 'static,
    {
        let handle = EngineHandle {
            inner: Arc::new((
                Mutex::new(Shared {
                    session: Session::new(Instant::now()),
                    config,
                    error: None,
                    shutdown: false,
                }),
                Condvar::new(),
            )),
        };
        let inner = handle.inner.clone();
        let thread = std::thread::spawn(move || run(&inner, make_clicker));
        Self {
            handle,
            thread: Some(thread),
        }
    }

    pub fn handle(&self) -> EngineHandle {
        self.handle.clone()
    }
}

fn run<F>(inner: &(Mutex<Shared>, Condvar), make_clicker: F)
where
    F: FnOnce() -> Result<Box<dyn Clicker>, String>,
{
    let (lock, wake) = inner;
    let mut clicker = match make_clicker() {
        Ok(clicker) => clicker,
        Err(e) => {
            lock.lock().unwrap().error = Some(e);
            return;
        }
    };

    let mut shared = lock.lock().unwrap();
    while !shared.shutdown {
        // Le clic est envoyé verrou tenu : après un `stop()`, aucun clic ne peut partir (R14).
        let next = shared.session.poll(Instant::now(), clicker.as_mut());
        shared = match next {
            None => wake.wait(shared).unwrap(),
            Some(at) => {
                let delay = at.saturating_duration_since(Instant::now());
                wake.wait_timeout(shared, delay).unwrap().0
            }
        };
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.handle.write(|s| {
            s.session.stop();
            s.shutdown = true;
        });
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MouseButton;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    struct CountingClicker(Arc<AtomicU64>);

    impl Clicker for CountingClicker {
        fn click(&mut self, _: MouseButton) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn engine_with_counter(cps: u32) -> (Engine, Arc<AtomicU64>) {
        let count = Arc::new(AtomicU64::new(0));
        let c = count.clone();
        let config = Config {
            button: MouseButton::Left,
            cps,
        };
        (
            Engine::spawn(config, move || {
                Ok(Box::new(CountingClicker(c)) as Box<dyn Clicker>)
            }),
            count,
        )
    }

    #[test]
    fn r11_r14_real_thread_clicks_then_stops() {
        let (engine, count) = engine_with_counter(100);
        let handle = engine.handle();
        handle.toggle(StartSource::Hotkey);
        std::thread::sleep(Duration::from_millis(300));
        handle.toggle(StartSource::Hotkey);
        let after_stop = count.load(Ordering::SeqCst);
        assert!(
            (20..=40).contains(&after_stop),
            "≈30 clics attendus, obtenu {after_stop}"
        );
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(
            count.load(Ordering::SeqCst),
            after_stop,
            "R14 : aucun clic après l'arrêt"
        );
        assert_eq!(handle.clicks(), after_stop);
    }

    #[test]
    fn r15_dropping_engine_stops_thread() {
        let (engine, count) = engine_with_counter(1000);
        engine.handle().toggle(StartSource::Hotkey);
        std::thread::sleep(Duration::from_millis(50));
        drop(engine); // attend la fin du fil : bloquerait si le fil ne s'arrêtait pas
        let after_drop = count.load(Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(count.load(Ordering::SeqCst), after_drop);
    }

    #[test]
    fn clicker_creation_error_is_reported() {
        let engine = Engine::spawn(Config::default(), || Err("pas de X11".to_string()));
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(engine.handle().error().as_deref(), Some("pas de X11"));
    }
}
