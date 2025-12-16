use std::*;

#[link(name = "ftd2xx64")]
unsafe extern "C" {
    fn FT_OpenEx(_: *const u8, _: u32, _: *mut usize) -> u32;
    fn FT_Close(_: usize) -> u32;
    fn FT_SetBitMode(_: usize, _: u8, _: u8) -> u32;
    fn FT_Write(_: usize, _: *const u8, _: u32, _: *mut u32) -> u32;
}

pub struct FtBitBang {
    handle: usize,
}

impl Drop for FtBitBang {
    fn drop(&mut self) {
        unsafe { FT_Close(self.handle) };
    }
}

impl FtBitBang {
    pub fn new(desc: &[u8]) -> io::Result<FtBitBang> {
        assert!(desc.last() == Some(&0));

        let mut handle = 0;
        // 2 == FT_OPEN_BY_DESCRIPTION
        if unsafe { FT_OpenEx(desc.as_ptr(), 2, &mut handle) } != 0 {
            return Err(io::Error::other("FT_OpenEx"));
        }
        // 1 == FT_BITMODE_ASYNC_BITBANG
        if unsafe { FT_SetBitMode(handle, 0xff, 1) } != 0 {
            return Err(io::Error::other("FT_SetBitMode"));
        }
        Ok(FtBitBang { handle: handle })
    }

    pub fn set_bits(&self, bits: u8) -> io::Result<()> {
        let mut len = 0;
        if unsafe { FT_Write(self.handle, &bits, 1, &mut len) } != 0 {
            return Err(io::Error::other("FT_Write"));
        }
        Ok(())
    }
}
