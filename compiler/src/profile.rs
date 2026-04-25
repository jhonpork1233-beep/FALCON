use std::str::FromStr;
use std::collections::HashSet;

/// Capabilities that a module might require
/// Used for import validation - profiles restrict which capabilities are allowed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Requires heap allocator
    Heap,
    /// Requires Falcon runtime
    Runtime,
    /// Uses panic/unwrap
    Panic,
    /// Contains unsafe code
    Unsafe,
    /// Requires threading support
    Threads,
    /// Requires OS syscalls
    Os,
}

/// Compilation profile determines safety guarantees and runtime behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Userland profile (default)
    /// - Bounds checking
    /// - Heap allocations allowed
    /// - Panic unwinding
    /// - Memory sanitizers available
    Userland,
    
    /// Kernel profile
    /// - No implicit heap allocations
    /// - No panics (must use Result)
    /// - Explicit lifetimes at boundaries
    /// - Strict aliasing rules
    Kernel,
    
    /// Baremetal profile
    /// - Zero runtime overhead
    /// - No safety checks
    /// - Direct hardware access
    /// - Programmer trusted completely
    Baremetal,
}

impl Default for Profile {
    fn default() -> Self {
        Profile::Userland
    }
}

impl FromStr for Profile {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "userland" => Ok(Profile::Userland),
            "kernel" => Ok(Profile::Kernel),
            "baremetal" => Ok(Profile::Baremetal),
            _ => Err(format!("Unknown profile: {}. Must be: userland, kernel, or baremetal", s)),
        }
    }
}

impl Profile {
    /// Get the set of capabilities allowed by this profile
    pub fn allowed_capabilities(&self) -> HashSet<Capability> {
        match self {
            Profile::Userland => {
                // Userland allows all capabilities
                [
                    Capability::Heap,
                    Capability::Runtime,
                    Capability::Panic,
                    Capability::Unsafe,
                    Capability::Threads,
                    Capability::Os,
                ].into_iter().collect()
            }
            Profile::Kernel => {
                // Kernel only allows unsafe (for direct hardware/memory access)
                // No heap, no runtime, no panics
                [Capability::Unsafe].into_iter().collect()
            }
            Profile::Baremetal => {
                // Baremetal only allows unsafe
                // Zero runtime, direct hardware access only
                [Capability::Unsafe].into_iter().collect()
            }
        }
    }
}
