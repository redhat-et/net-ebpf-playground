
#![no_std]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PacketLog {
    pub ipv4_address: u32,
    pub action: u32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PacketFiveTuple { 
    pub src_address: u32, 
    pub dst_address: u32,
    pub src_port: u16,
    pub dst_port: u16, 
    pub protocol: u8,
    pub _pad: [u8; 3],
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for PacketLog {}

#[cfg(feature = "user")]
unsafe impl aya::Pod for PacketFiveTuple {}
