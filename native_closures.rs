#![feature(libc)]
extern crate libc;

#[repr(C)]
struct GMainLoop;

#[link(name = "glib-2.0")]
extern {
    fn g_main_loop_new(ctx: *const libc::c_void, is_running: libc::c_int) -> *mut GMainLoop;
    fn g_main_loop_run(l: *mut GMainLoop);
    fn g_main_loop_quit(l: *mut GMainLoop);
    fn g_main_loop_ref(l: *mut GMainLoop) -> *mut GMainLoop;
    fn g_main_loop_unref(l: *mut GMainLoop);

    fn g_idle_add_full(prio: libc::c_int, f: extern fn (*mut libc::c_void) -> libc::c_int, user_data: *mut libc::c_void, destroy: extern fn (*mut libc::c_void));
}

struct MainLoop {
    raw: *mut GMainLoop
}

impl MainLoop {
    fn new() -> MainLoop {
        unsafe {
            let raw = g_main_loop_new(std::ptr::null(), 0);
            return MainLoop {raw: raw};
        };
    }

    fn run(&self) {
        unsafe {
            g_main_loop_run(self.raw);
        }
    }

    fn quit(&self) {
        unsafe {
            g_main_loop_quit(self.raw);
        }
    }
}

impl Drop for MainLoop {
    fn drop(&mut self) {
        unsafe {
            g_main_loop_unref(self.raw);
        }
    }
}

impl Clone for MainLoop {
    fn clone(&self) -> Self {
        unsafe {
            return MainLoop{raw: g_main_loop_ref(self.raw)};
        }
    }
}

enum SourceReturn {
    SourceContinue,
    SourceRemove
}

fn idle_add<F>(f: F)
  where F: FnMut() -> SourceReturn + 'static {
    let closure = Box::new(f);

    unsafe {
        g_idle_add_full(200, dispatch::<F>, std::mem::transmute(closure), destroy::<F>);
    }

    extern fn dispatch<F>(user_data: *mut libc::c_void) -> libc::c_int
      where F: FnMut() -> SourceReturn + 'static {
        unsafe {
            let mut closure: Box<F> = std::mem::transmute(user_data);

            let res = match (*closure)() {
                SourceReturn::SourceRemove => 0,
                _                          => 1
            };

            std::mem::forget(closure);
            return res;
        }
    }

    extern fn destroy<F>(user_data: *mut libc::c_void)
      where F: FnMut() -> SourceReturn + 'static{
        unsafe {
            let _: Box<F> = std::mem::transmute(user_data);
        }
    }
}

struct Foo {
    b : u32
}

impl Drop for Foo {
    fn drop(&mut self) {
        print!("drop!\n");
    }
}

fn foo(l: &mut MainLoop) {
    let l = l.clone();
    let mut x = 0;
    let y = Foo{b: 1};

    idle_add(move || {
        x += 1;
        print!("bar {} {}\n", x, y.b);

        if x >= 100 {
            l.quit();
            return SourceReturn::SourceRemove;
        }

        return SourceReturn::SourceContinue;
    });
}

fn main() {
    let mut l = MainLoop::new();

    foo(&mut l);

    l.run();

    print!("done\n");
}
