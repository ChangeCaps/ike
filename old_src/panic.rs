use std::panic::UnwindSafe;

#[derive(Clone, Copy, Debug, Default)]
pub struct Panic;

impl std::fmt::Display for Panic {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "an operation panicked")
    }
}

pub type Result<T> = std::result::Result<T, Panic>;

pub fn catch<T>(f: impl FnOnce() -> T + UnwindSafe) -> Result<T> {
    let res = std::panic::catch_unwind(f);

    match res {
        Ok(t) => Ok(t),
        Err(e) => {
            std::thread::spawn(|| {
                std::panic::resume_unwind(e);
            });

            Err(Panic)
        }
    }
}
