

#[macro_export]
macro_rules! create_hooks {
    ($($orig_fn:ident ($($param:ident: $ptype:ty),*) -> $ret:ty),*) => {
        $(
            create_hook!($orig_fn($($param: $ptype),*) -> $ret);
        )*
    };
}

#[macro_export]
macro_rules! create_hook {
    ($orig_fn:ident ($($param:ident: $ptype:ty),*) -> $ret:ty) => {
        #[allow(dead_code)]
        unsafe extern "C" fn $orig_fn($($param: $ptype),*) -> $ret {
            $orig_fn::Chain::new().call($($param),*)
        }

        #[allow(dead_code)]
        mod $orig_fn {
            use super::*;
            use std::sync::{Mutex, atomic::AtomicPtr};
            

            pub static HOOKS: Mutex<Vec<HookFn>> = Mutex::new(vec![]);
            #[derive(Clone)]
            pub struct HookFn {
                pub f: fn($($ptype),*, &mut Chain) -> $ret,
            }

            pub struct Chain {
                index: usize,
            }
            impl Chain {
                pub fn new() -> Self {
                    Chain { index: 0 }
                }
                pub fn call(&mut self, $($param: $ptype),*) -> $ret {
                    match HOOKS.lock().unwrap().get(self.index) {
                        Some(hook) => {
                            let result = (hook.f)($($param),*, self);
                            self.index += 1;
                            result
                        }
                        None => {
                            call_orig($($param),*, self)
                        }
                    }
                }
            }
            pub fn add_hook(hook: fn($($ptype),*, &mut Chain) -> $ret) {
                let mut hooks = HOOKS.lock().unwrap();
                hooks.push(HookFn { f: hook });
            }

            pub fn call_orig($($param: $ptype),*, _: &mut Chain) -> $ret {
                use std::sync::LazyLock;

                static REAL: LazyLock<AtomicPtr<c_void>> = LazyLock::new(|| {
                    AtomicPtr::new( unsafe {
                            libc::dlsym(
                                libc::RTLD_NEXT,
                                concat!(stringify!($orig_fn), "\0").as_ptr() as *const c_char,
                            )
                        }
                    )
                });

                unsafe {
                    (::std::mem::transmute::<*const c_void, unsafe extern "C" fn ( $($param: $ptype),* ) -> $ret>(
                        REAL.load(std::sync::atomic::Ordering::SeqCst)
                    ))($($param),*)
                }
            }
        }

    };
}

