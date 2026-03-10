pub struct ByteBuffer {
    pub data: Vec<u8>,
    pub position: usize,
}

impl ByteBuffer {
    #[inline]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, position: 0 }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            position: 0,
        }
    }

    #[inline(always)]
    fn advance(&mut self, bytes: usize) -> usize {
        let start = self.position;
        let end = start.checked_add(bytes).expect("buffer position overflow");

        assert!(end <= self.data.len(), "buffer underflow");

        self.position = end;

        start
    }

    #[inline(always)]
    fn read_array<const N: usize>(&mut self) -> [u8; N] {
        let start = self.advance(N);
        // SAFETY: `advance` verified bounds and `u8` alignment is always valid.
        unsafe { *(self.data.as_ptr().add(start) as *const [u8; N]) }
    }

    #[inline(always)]
    pub fn write_boolean(&mut self, value: bool) {
        self.data.push(if value { 1 } else { 0 });
    }

    #[inline(always)]
    pub fn write_bytes(&mut self, value: &[u8]) {
        self.data.extend_from_slice(value);
    }

    #[inline(always)]
    pub fn write_short(&mut self, value: i16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    #[inline(always)]
    pub fn write_int(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    #[inline(always)]
    pub fn write_uint(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    #[inline(always)]
    pub fn write_float(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    #[inline(always)]
    pub fn write_double(&mut self, value: f64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    #[inline(always)]
    pub fn write_utf(&mut self, value: &str) {
        let bytes = value.as_bytes();

        self.write_short(bytes.len() as i16);
        self.data.extend_from_slice(bytes);
    }

    #[inline(always)]
    pub fn read_boolean(&mut self) -> bool {
        self.read_byte() != 0
    }

    #[inline(always)]
    pub fn read_byte(&mut self) -> u8 {
        self.read_array::<1>()[0]
    }

    #[inline(always)]
    pub fn read_short(&mut self) -> i16 {
        i16::from_be_bytes(self.read_array())
    }

    #[inline(always)]
    pub fn read_int(&mut self) -> i32 {
        i32::from_be_bytes(self.read_array())
    }

    #[inline(always)]
    pub fn read_uint(&mut self) -> u32 {
        u32::from_be_bytes(self.read_array())
    }

    #[inline(always)]
    pub fn read_float(&mut self) -> f32 {
        f32::from_be_bytes(self.read_array())
    }

    #[inline(always)]
    pub fn read_padding(&mut self) {
        self.position += 7;
    }

    #[inline(always)]
    pub fn read_double(&mut self) -> f64 {
        f64::from_be_bytes(self.read_array())
    }

    #[inline(always)]
    pub fn read_utf(&mut self) -> String {
        let length = self.read_short() as usize;
        let start = self.advance(length);
        let bytes = &self.data[start..self.position];

        std::str::from_utf8(bytes)
            .expect("Invalid UTF-8 sequence")
            .to_owned()
    }
}
