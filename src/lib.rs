#![feature(lazy_cell)]

use std::sync::Mutex;

#[macro_export]
macro_rules! cvars {
    ($($name:ident: $type:ty = $value:expr),+ $(,)?) => {
        use std::sync::LazyLock;

        $(
            #[allow(non_upper_case_globals)]
            static $name: LazyLock<CVar<$type>> = LazyLock::new(|| CVar::init($value));
        )+
    }
}

// enum CVarState<T: 'static, F: Copy> {
//     Init(&'static Mutex<T>),
//     Uninit(ManuallyDrop<F>),
// }

// impl<T: 'static, F: Copy> Clone for CVarState<T, F> {
//     fn clone(&self) -> Self {
//         match self {
//             Self::Init(arg0) => Self::Init(arg0.clone()),
//             Self::Uninit(arg0) => Self::Uninit(arg0.clone()),
//         }
//     }
// }

// impl<T, F: Copy> Copy for CVarState<T, F> {}

pub struct CVar<T>
where
    T: 'static,
{
    v: &'static Mutex<T>,
}

impl<T> CVar<T>
where
    T: 'static,
{
    pub fn init(v: T) -> Self {
        Self {
            v: Box::leak(Box::new(Mutex::new(v))),
        }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, T> {
        self.v.lock().unwrap()
    }
}

// impl<'b, T, F: Copy> Deref for CVar<T, F> {
//     type Target = std::sync::MutexGuard<'b, T>;

//     fn deref(&'b self) -> &'b Self::Target
//     {
//         // self.value.lock().unwrap()
//     }
// }

impl<T> Clone for CVar<T> {
    fn clone(&self) -> Self {
        Self {
            v: self.v,
        }
    }
}

impl<T> Copy for CVar<T> {}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Condvar, Mutex};

    use super::CVar;

    cvars!(
        NAME: String = "Default Name".to_string(),
        COMPLEX: Complex = Complex {
            _valid: true,
            _stuff: String::from("test"),
            _bytes: vec![0, 0, 1, 1],
        },
    );

    #[derive(Debug)]
    struct Complex {
        _valid: bool,
        _stuff: String,
        _bytes: Vec<u8>,
    }

    struct Test {
        name: CVar<String>,
        complex: CVar<Complex>,
    }

    impl Test {
        pub fn new() -> Self {
            Self {
                name: *NAME,
                complex: *COMPLEX,
            }
        }

        pub fn use_name(&self) {
            println!("Name: {}", self.name.lock());
        }

        pub fn set_name(&self, name: &str) {
            *self.name.lock() = name.into();
        }

        pub fn use_complex(&self) {
            println!("{:#?}", self.complex.lock());
        }
    }

    #[test]
    fn test() {
        let mut handles = Vec::new();

        let mutex = Mutex::new(false);
        let condvar = Condvar::new();

        let p = Arc::new((mutex, condvar));
        for i in 0..16 {
            let p = p.clone();
            let handle = std::thread::spawn(move || {
                let lock = p.0.lock().unwrap();
                std::mem::drop(p.1.wait(lock).unwrap());

                let test = Test::new();

                test.use_name();
                test.set_name(&format!("{}", i));
                test.use_name();

                test.use_complex();
            });

            handles.push(handle);
        }

        std::thread::sleep(std::time::Duration::from_secs(1));

        *p.0.lock().unwrap() = true;
        p.1.notify_all();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
