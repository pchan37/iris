const KILOBYTE: usize = 1024;
const MEGABYTE: usize = 1024 * KILOBYTE;

/// Keep the chunk size reasonably small to balance between overhead and excessive
/// memory usage
pub const CHUNK_SIZE: usize = 64 * MEGABYTE;
