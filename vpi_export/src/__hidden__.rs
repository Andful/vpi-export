use core::sync::atomic::{AtomicPtr, Ordering};

pub use ctor::ctor;

pub struct VpiFunctionCollection {
    head: AtomicPtr<VpiFunctionNode>,
}

pub struct VpiFunctionNode {
    f: fn(),
    next: *const VpiFunctionNode,
}

impl VpiFunctionNode {
    pub const fn new(f: fn()) -> Self {
        Self {
            f,
            next: core::ptr::null(),
        }
    }
}

impl VpiFunctionCollection {
    const fn new() -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    //SAFETY: unique access of vpi_function_node is required
    pub fn push(&self, vpi_function_node: &mut VpiFunctionNode) {
        let next = self.head.swap(vpi_function_node, Ordering::Relaxed);
        vpi_function_node.next = next;
    }
}

pub static VPI_FUNCTION_COLLECTION: VpiFunctionCollection = VpiFunctionCollection::new();

pub extern "C" fn register_vpi_functions() {
    let mut head = VPI_FUNCTION_COLLECTION.head.load(Ordering::Relaxed) as *const VpiFunctionNode;
    //SAFETY: No mutable references of the vpi_function_node exists
    while let Some(vpi_function_node) = unsafe { head.as_ref() } {
        (vpi_function_node.f)();
        head = vpi_function_node.next;
    }
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static vlog_startup_routines: [Option<extern "C" fn()>; 2] =
    [Some(register_vpi_functions), None];
