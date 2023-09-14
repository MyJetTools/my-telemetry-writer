use std::sync::atomic::AtomicI64;

#[derive(Debug, Clone)]
pub enum WriteMode {
    Unknown,
    Grpc,
    Http,
}

impl WriteMode {
    pub fn is_unknown(&self) -> bool {
        match self {
            WriteMode::Unknown => true,
            _ => false,
        }
    }
}
pub struct WriteModeKeeper {
    mode: AtomicI64,
}

impl WriteModeKeeper {
    pub fn new() -> Self {
        Self {
            mode: AtomicI64::new(0),
        }
    }

    pub fn get_write_mode(&self) -> WriteMode {
        match self.mode.load(std::sync::atomic::Ordering::Relaxed) {
            1 => WriteMode::Grpc,
            2 => WriteMode::Http,
            _ => WriteMode::Unknown,
        }
    }

    pub fn set_write_mode(&self, mode: WriteMode) {
        println!("Telemetry mode is switched to {:?}", mode);
        match mode {
            WriteMode::Grpc => self.mode.store(1, std::sync::atomic::Ordering::Relaxed),
            WriteMode::Http => self.mode.store(2, std::sync::atomic::Ordering::Relaxed),
            _ => self.mode.store(0, std::sync::atomic::Ordering::Relaxed),
        }
    }
}
