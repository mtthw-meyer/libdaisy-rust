const BLOCK_SIZE_MAX: usize = 48;
const BUFFER_SIZE: usize = BLOCK_SIZE_MAX * 2;

#[no_mangle]
#[link_section = ".sdram1_bss"]
static mut buf_in: [i32; BUFFER_SIZE] = [0; BUFFER_SIZE];
#[no_mangle]
#[link_section = ".sdram1_bss"]
static mut buf_out: [i32; BUFFER_SIZE] = [0; BUFFER_SIZE];

struct Audio {}

impl Audio {}

// Setup SAI
