use std::cell::RefMut;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::mem;
use std::path::Path;
use std::rc::Rc;
use std::str;
use image::DynamicImage;
use image::GenericImage;
use image::GenericImageView;
use vtfx::VTFXHEADER;
use std::convert::TryInto;
use num_enum::TryFromPrimitive;

use crate::image_format::ImageFormat;
use crate::resource_entry_info::ResourceEntryInfo;
use crate::vtfx::VTF_LEGACY_RSRC_IMAGE;
use crate::vtfx::Vector;

mod vtfx;
mod image_format;
mod resource_entry_info;

fn main() {
    let mut args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("VTFX Reader");
        println!("Enter path of file:");
        let mut buffer = String::new();
        let stdin = io::stdin();
        match stdin.read_line(&mut buffer) {
            Err(e) => println!("Input not valid"),
            Ok(_) => (),
        }
        //Remove trailing new line chars
        if buffer.ends_with('\n') {
            buffer.pop();
            if buffer.ends_with('\r') {
                buffer.pop();
            }
        }
        buffer = buffer.replace("\"", "");
        //Check path is file
        let path = Path::new(&buffer);

        if path.is_file() {
            args.push(buffer);
        } else {
            println!("Provided path '{}' is not a file!", buffer);
            return;
        }
    }

    match read_vtfx(Path::new(&args[1])) {
        Ok(_) => {println!("Success")},
        Err(e) => {println!("Failed to open file: {e}")},
    };

}

fn read_vtfx(path: &Path) -> Result<VTFXHEADER, Box<dyn Error>> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    let type_str_range = &buffer[0..4];
    let type_str = str::from_utf8(&type_str_range)?;
    if type_str != "VTFX"
    {
        let err = io::Error::new(io::ErrorKind::Other, "File is not VTFX file!");
        return Err(Box::new(err));
    }

    let mut vtfx: VTFXHEADER = { Default::default() };

    let mut i = 0;
    vtfx.file_type_string = String::from(type_str);
    i += 4;
    println!("Type: {}", vtfx.file_type_string);
    
    let mut version: [i32; 2] = [0; 2];
    for j in 0..2 {
        let start = i + (j * 4);
        let end = start + 4;
        let slice: &[u8] = &buffer[start..end];
        let value: i32 = i32::from_be_bytes(slice.try_into().unwrap());
        version[j] = value;
    }
    i += 4 * 2;
    vtfx.version = version;

    vtfx.header_size = i32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
    i += 4;

    vtfx.flags = u32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
    i += 4;

    let has_alpha = vtfx.has_alpha();
    let has_alpha_onebit = vtfx.has_onebit_alpha();
    let dxt_hint = vtfx.hint_dx5();
    if cfg!(debug_assertions) && dxt_hint
    {
        println!("[Debug] Has dxt5 hint flag");
    }  

    println!("Version: {}.{}, Header Size: {}, Flags: {}. Has Alpha: {}, One Bit Alpha: {}", vtfx.version[0], vtfx.version[1], vtfx.header_size, vtfx.flags, has_alpha, has_alpha_onebit);

    vtfx.width = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
    i += 2;

    vtfx.height = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
    i += 2;

    vtfx.depth = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
    i += 2;

    vtfx.num_frames = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
    i += 2;
    println!("Num Frames: {}", vtfx.num_frames);

    vtfx.preload_data_size = u16::from_be_bytes(buffer[i..i+2].try_into().unwrap());
    i += 2;
    println!("Preload Data Size: {}", vtfx.preload_data_size);

    vtfx.mip_skip_count = u8::from_be_bytes(buffer[i..i+1].try_into().unwrap());
    i += 1;
    println!("Mip Skip Count: {}", vtfx.mip_skip_count);

    vtfx.num_resources = u8::from_be_bytes(buffer[i..i+1].try_into().unwrap());
    i += 1;
    println!("Num Resources: {}", vtfx.num_resources);

    //vtfx.reflectivity
    i += mem::size_of::<Vector>();

    vtfx.bump_scale = f32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
    i += 4;

    let image_format_i32 = i32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
    println!("Raw image format: {}", image_format_i32);
    vtfx.image_format = ImageFormat::try_from_primitive(image_format_i32).unwrap();
    i += 4;

    //vtfx.low_res_image_sample
    i += 4;

    vtfx.compressed_size = u32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
    i += 4;

    println!("Width: {}, Height: {}, Depth: {}, Image Format: {:?}, Compressed Size: {}", vtfx.width, vtfx.height, vtfx.depth, vtfx.image_format, vtfx.compressed_size);

    let mut resource_entry_infos: Vec<ResourceEntryInfo> = Vec::new();
    for _res_num in 0..vtfx.num_resources
    {
        let mut resource_entry_info: ResourceEntryInfo = { Default::default() };
        for x in 0..3
        {
            resource_entry_info.chTypeBytes[x] = buffer[i+x];
        }
        i += 4;
        resource_entry_info.resData = u32::from_be_bytes(buffer[i..i+4].try_into().unwrap());
        i += 4;
        resource_entry_infos.push(resource_entry_info);
    }

    if cfg!(debug_assertions){ println!("[Debug] READ END: Current read position: {}, Data left: {} bytes.\n", i, buffer.len() - i); }

    let mut res_num = 0;
    for resource in resource_entry_infos
    {
        println!("Reading resource #{}. Type: {:?}, Start: {}", res_num, resource.chTypeBytes, resource.resData);

        //Is this resource a high res image?
        if resource.chTypeBytes == VTF_LEGACY_RSRC_IMAGE
        {
            println!("    Type is VTF_LEGACY_RSRC_IMAGE");

            match resource_to_image(&buffer, &resource, &vtfx) {
                Ok(image) => {
                    let new_image_name = format!("output_{res_num}.png");
                    println!("    Saved resource image data to '{}'", new_image_name);
                    image.save_with_format(new_image_name, image::ImageFormat::Png)?;
                },
                Err(error) => {println!("    {}", error)},
            }
            
            res_num += 1;
        }
        else
        {
            println!("Error: Unknown type, skipping...");
        }
    }

    Ok(vtfx)
}

fn resource_to_image(buffer: &[u8], resource_entry_info: &ResourceEntryInfo, vtfx: &VTFXHEADER) -> Result<DynamicImage, Box<dyn Error>>
{
    let res_start: usize = resource_entry_info.resData.try_into()?;
    let image_slice = &buffer[res_start..buffer.len()];
    
    let mut output_image = DynamicImage::new_rgba8(vtfx.width.into(), vtfx.height.into());
    let image_format = &vtfx.image_format;
    let format_info = image_format.get_format_info();
    if format_info.is_some()
    {
        let format_info_u = format_info.unwrap();
        let channels = match vtfx.has_alpha() {
            true => 4,
            false => 3
        };
        let size: u32 = channels * vtfx.width as u32 * vtfx.height as u32;
        let mut image_vec: Vec<u8>;

        //If this format is BC encoded
        if format_info_u.bc_format.is_some()
        {
            let bc_format = format_info_u.bc_format.unwrap();
            image_vec = vec![0; size.try_into().unwrap()];
            let image_vec_slice = image_vec.as_mut_slice();

            if cfg!(debug_assertions){ println!("[Debug] In slice size: {}. Allocated {} bytes for decompression. Channels: {}", image_slice.len(), size, channels); }

            let expected_compressed_size = bc_format.compressed_size(vtfx.width.into(), vtfx.height.into());
            if expected_compressed_size != image_slice.len()
            {
                println!("WARN: Resource size is {} but expected length is {}. ({} % diff) Program may crash.", image_slice.len(), expected_compressed_size, (image_slice.len() as f32 / expected_compressed_size as f32) * 100f32);
            }

            //Decompress. More research needed to see if a custom version to handle big endian data is needed instead.
            bc_format.decompress( image_slice, vtfx.width.into(), vtfx.height.into(), image_vec_slice);
        }
        else
        {
            image_vec = Vec::from(image_slice);
        }

        //Take decompressed data and put into image
        for x in 0..vtfx.width
        {
            for y in 0..vtfx.height
            {
                let mut pixel = output_image.get_pixel(x.into(), y.into()).clone();
                let pixel_index: u16 = x + (y * vtfx.width);
                for i in 0..channels.try_into().unwrap()
                {
                    //Using format data, construct index and copy source image pixel colour data
                    let channel_offset = format_info_u.channel_order[i];
                    let from_index: usize = ((format_info_u.depth * pixel_index) + (channel_offset * format_info_u.depth)).try_into().unwrap();
                    pixel[i] = get_pixel_as_u8(&image_vec, from_index, &format_info_u.depth);
                }
                output_image.put_pixel(x.into(), y.into(), pixel);
            }
        }
    }
    else
    {
        let err = io::Error::new(io::ErrorKind::Other, format!("Unsupported image format: {:?}", image_format));
        return Err(Box::new(err));
    }

    Ok(output_image)
}

///Get pixel as u8. Convert larger sized pixels down
fn get_pixel_as_u8(in_buffer: &Vec<u8>, index: usize, depth: &u16) -> u8
{
    match depth
    {
        1 => in_buffer[index],
        2 => {
            let colour = u16::from_be_bytes(in_buffer[index..index+2].try_into().unwrap());
            return (colour / 2).try_into().unwrap();
        },
        4 => {
            let colour = u32::from_be_bytes(in_buffer[index..index+4].try_into().unwrap());
            return (colour / 4).try_into().unwrap();
        },
        _ => { 0 }
    }
}