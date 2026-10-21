use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ptr;

#[derive(Clone, Copy)]
enum State {
    Inactive,
    Refusing(usize),
}

thread_local! {
    static STATE: Cell<State> = const { Cell::new(State::Inactive) };
}

fn refuses() -> bool {
    STATE
        .try_with(|state| match state.get() {
            State::Inactive => false,
            State::Refusing(count) => {
                state.set(State::Refusing(count.saturating_add(1)));
                true
            }
        })
        .unwrap_or(false)
}

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if refuses() {
            ptr::null_mut()
        } else {
            unsafe { System.alloc(layout) }
        }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, length: usize) -> *mut u8 {
        if refuses() {
            ptr::null_mut()
        } else {
            unsafe { System.realloc(pointer, layout, length) }
        }
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        STATE.with(|state| state.set(State::Inactive));
    }
}

pub fn refusing<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    STATE.with(|state| {
        assert!(matches!(state.get(), State::Inactive));
        state.set(State::Refusing(0));
    });
    let restore = Restore;
    let result = operation();
    let requests = STATE.with(|state| match state.get() {
        State::Refusing(count) => count,
        State::Inactive => 0,
    });
    drop(restore);
    (result, requests)
}
