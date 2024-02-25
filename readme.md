# VTFX Reader + Image Export
[![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads/rob5300/vtfx_reader/latest/total?sort=date)
](https://github.com/rob5300/vtfx_reader/releases/latest)

A tool to read the header + output image resources (as png) from a [VTFX file](https://developer.valvesoftware.com/wiki/VTFX_file_format). Written in rust.

- Supports 360 vtf files [\*.360.vtf]. 
- PS3 files are also supported [\*.vtf] but use the ``--no-dxt-fix`` argument to get a desired output.

*Note: X360 images usually have multiple mip levels packed into the texture, functionality to export only the max mip level is in progress and experimental.*

## Working texture export formats (Open issue to request):
- DXT1
- DXT5
- RGBA16161616
- 
Compressed (LZMA) and non compressed images are supported. By default alpha is not exported, but can be enabled with the ``--export-alpha`` argument.

## How to use
Download the latest release and run, using the arguments listed below to specify the input files and options.

## Arguments
    -i, --input <INPUT>
            Input .vtf file path

    -o, --output <OUTPUT>
            Output folder

        --export-alpha
            Export alpha channel

        --mip0-only
            Try to output only mip 0 (EXPERIMENTAL)

        --no-dxt-fix
            Do not use big to little endian fix on DXT images

        --no-resource-export
            Do not export any resources

        --open
            Auto open exported images

## Compiling
To compile from source, install the rust tooling [rustup](https://rustup.rs/), then use ``cargo to run`` and build the project.

[texpresso](https://crates.io/crates/texpresso) is used to decode dxt data, and [lzma-rs](https://crates.io/crates/lzma-rs) for lzma decompression.
