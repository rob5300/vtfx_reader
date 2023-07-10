/*

typedef struct tagVTFXHEADER  
{
	char fileTypeString[4];         // VTFX.
	int version[2]; 		// version[0].version[1].
	int headerSize;
	unsigned int	flags;
	unsigned short	width;					// actual width of data in file.
	unsigned short	height;					// actual height of data in file.
	unsigned short	depth;					// actual depth of data in file.
	unsigned short	numFrames;
	unsigned short	preloadDataSize;		// exact size of preload data (may extend into image!).
	unsigned char	mipSkipCount;			// used to resconstruct mapping dimensions.
	unsigned char	numResources;
	Vector			reflectivity;			// Resides on 16 byte boundary!.
	float			bumpScale;
	ImageFormat		imageFormat;
	unsigned char	lowResImageSample[4];
	unsigned int	compressedSize;

	// *** followed by *** ResourceEntryInfo resources[0];
} VTFXHEADER;
*/

const VTF_X360_MAJOR_VERSION: i32 = 0x0360;
const VTF_X360_MINOR_VERSION: i32 = 8;

#[repr(C)]
#[derive(Debug)]
pub struct VTFXHEADER {
    pub file_type_string: [u8; 4],               // VTFX.
    pub version: [i32; 2],                     // version[0].version[1].
    pub header_size: i32,
    pub flags: u32,
    pub width: u16,                    // actual width of data in file.
    pub height: u16,                   // actual height of data in file.
    pub depth: u16,                    // actual depth of data in file.
    pub num_frames: u16,
    pub preload_data_size: u16,         // exact size of preload data (may extend into image!).
    pub mip_skip_count: u8,             // used to resconstruct mapping dimensions.
    pub num_resources: u8,
    pub reflectivity: Vector,           // Resides on 16 byte boundary!.
    pub bump_scale: f32,
    pub image_format: ImageFormat,
    pub low_res_image_sample: [u8; 4],
    pub compressed_size: u32,
    // *** followed by *** ResourceEntryInfo resources[0];
}

#[derive(Debug)]
pub struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug)]
pub enum ImageFormat 
{
	IMAGE_FORMAT_X360_DST16,
	IMAGE_FORMAT_X360_DST24,
	IMAGE_FORMAT_X360_DST24F,
	// supporting these specific formats as non-tiled for procedural cpu access
	IMAGE_FORMAT_LINEAR_BGRX8888,
	IMAGE_FORMAT_LINEAR_RGBA8888,
	IMAGE_FORMAT_LINEAR_ABGR8888,
	IMAGE_FORMAT_LINEAR_ARGB8888,
	IMAGE_FORMAT_LINEAR_BGRA8888,
	IMAGE_FORMAT_LINEAR_RGB888,
	IMAGE_FORMAT_LINEAR_BGR888,
	IMAGE_FORMAT_LINEAR_BGRX5551,
	IMAGE_FORMAT_LINEAR_I8,
	IMAGE_FORMAT_LINEAR_RGBA16161616,

	IMAGE_FORMAT_LE_BGRX8888,
	IMAGE_FORMAT_LE_BGRA8888,
}