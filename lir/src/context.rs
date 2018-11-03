use std::{ptr, mem};
use types::{ComplexType, ComplexTypeId};

// Pointer to the static memory. Note, this is not a u8 slices because this
// contains an heterogeneous list of references, symbols and values which might
// be of different type and sizes.
type StaticStorage = *const ();

/// A context is a structure which centralize all the data necessary for the
/// execution of any Unit. It holds the collection of complex types, and any
/// counter related to having unique identifiers.
pub struct Context {
    /// This counter is used for both Rehash instructions and Newhash
    /// instructions. It holds the next value to be allocated if any of these
    /// instruction should be added to the graph.
    wrapper_seed: usize,

    /// This vector holds the list of types references by all Unit associated to
    /// this context. Any ComplexTypeId is an index in this Vector.
    types: Vec<ComplexType>,

    /// If any, this is the pointer to the memory which contains static
    /// information filled by the static compiler with all the symbol references
    /// or values. This fields should be set with the function
    /// `set_statics_refs` on the constructed or deserialized Context. Once set,
    /// it is not allowed to change. Attempting to build any unit without
    /// setting this value will cause a compilation error if the Unit uses an
    /// StaticAddress-es.
    refs_ptr: StaticStorage,

    /// When a Context is created with the ContextBuilder, this field is mutated
    /// to account for the expected size of type which contains all the
    /// references to symbols compiled by the static compiler (LLVM backend). As
    /// the refs_ptr value cannot be deserialized, it has to be initialized as
    /// runtime, and as such this fields is used to ensure that we are not going
    /// to do any out-of-bounds memory read when reading StaticAddress-es.
    // TODO: Remove unnecessary public fields by moving the Context builder to this file.
    pub expected_refs_size: usize,
}

impl Context {
    /// Create a new Context. This function should not be used externally,
    /// instead use a ContextBuilder to build a context for you.
    pub fn new() -> Context {
        Context {
            wrapper_seed: 0,
            types: vec![],
            refs_ptr: ptr::null(),
            expected_refs_size: 0,
        }
    }

    /// Create a new hash seed, such that we can avoid aliasing of hash values.
    /// This function is used by the ContextBuilder for creating new Rehash /
    /// Newhash instructions.
    pub fn get_hash_seed(&mut self) -> usize {
        let value = self.wrapper_seed;
        self.wrapper_seed += 1;
        value
    }

    /// Add a new complex type in the list of known types. This function is used
    /// by the ContextBuilder to register types seen while generating Units.
    pub fn add_type(&mut self, ty: ComplexType) -> ComplexTypeId {
        self.types.push(ty);
        ComplexTypeId(self.types.len() - 1)
    }

    /// Given a ComplexTypeId, returns the associated type.
    pub fn get_type(&self, id: ComplexTypeId) -> &ComplexType {
        let ComplexTypeId(index) = id;
        &self.types[index]
    }

    /// Register the constant tuple of references which are used by all units
    /// which are built with this context. This function is only allowed to be
    /// called once per Context, calling it more than once would cause this
    /// function to panic.
    pub fn set_static_refs<T>(&mut self, refs: &'static T) {
        let refs = refs as *const _ as *const();
        if self.refs_ptr != ptr::null() {
            // TODO: Panic with a documented error code, or an explicit message
            // explaining how to fix this issue.
            panic!("set_static_refs can only be called once per context.")
        }
        if self.expected_refs_size != mem::size_of::<T>() {
            // TODO: Panic with a documented error code, or an explicit message
            // explaining how to fix this issue.
            panic!("set_static_refs called with a tuple of unexpected size.")
        }
        self.refs_ptr = refs;
    }

    /// Return the pointer to the list of symbol references or values as an
    /// unsigned value which should be used for converting StaticAddress-es
    /// opcodes.
    ///
    /// This functions panics if `set_static_refs` has not been called before
    /// calling this function.
    pub fn get_static_refs_address(&self) -> usize {
        if self.refs_ptr == ptr::null() {
            // TODO: Panic with a documented error code, or an explicit message
            // explaining how to fix this issue.
            panic!("set_static_refs was not called when initializing the Context.")
        }
        self.refs_ptr as usize
    }
}
