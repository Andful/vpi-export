use core::{
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

pub use ctor::ctor;

pub struct VpiFunctionCollections {
    head: AtomicPtr<VpiFunctionNode>,
}

pub struct VpiFunctionNode {
    f: fn(),
    next: AtomicPtr<VpiFunctionNode>,
}

impl VpiFunctionNode {
    pub const fn new(f: fn()) -> Self {
        Self {
            f,
            next: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

impl VpiFunctionCollections {
    const fn new() -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    pub fn push(&self, vpi_function_node: &VpiFunctionNode) {
        let next = self.head.swap(
            vpi_function_node as *const VpiFunctionNode as *mut VpiFunctionNode,
            Ordering::Relaxed,
        );
        vpi_function_node.next.store(next, Ordering::Relaxed);
    }
}

pub static VPI_FUNCTION_COLLECTIONS: VpiFunctionCollections = VpiFunctionCollections::new();

pub extern "C" fn register_vpi_functions() {
    let mut head = NonNull::new(
        VPI_FUNCTION_COLLECTIONS
            .head
            .swap(core::ptr::null_mut(), Ordering::Relaxed),
    );
    while let Some(ptr) = head {
        //SAFETY: No mutable references of the vpi_function_node exists
        let vpi_function_node = unsafe { ptr.as_ref() };
        (vpi_function_node.f)();
        head = NonNull::new(vpi_function_node.next.load(Ordering::Relaxed));
    }
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static vlog_startup_routines: [Option<extern "C" fn()>; 2] =
    [Some(register_vpi_functions), None];
