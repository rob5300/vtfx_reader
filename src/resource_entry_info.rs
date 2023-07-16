#[repr(C)]
#[derive(Debug, Default)]
#[allow(non_snake_case)] 
pub struct ResourceEntryInfo
{
	pub chTypeBytes: [u8; 4],
	pub resData: u32	// Resource data or offset from the beginning of the file
}