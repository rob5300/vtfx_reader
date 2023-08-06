# VTFX Reader + Image Export
A tool to read the header + output image resources from a VTFX file. Written in rust.

Tested with xbox 360 vtf files.

Working texture export formats:
- DXT1
- DXT5
- RGBA16161616

Compressed (LZMA) and non compressed images are supported.

## Arguments
    -i, --input <INPUT>
            Input .vtf file path

    -o, --output <OUTPUT>
            Output folder

        --mip0-only
            Try to output only mip 0 (EXPERIMENTAL)

        --no-dxt-fix
            Do not use big to little endian fix on DXT images

        --no-resource-export
            Do not export any resources

        --open
            Auto open exported images

    -h, --help
            Print help (see a summary with '-h')

    -V, --version
            Print version