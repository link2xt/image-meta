
# image-meta

Image meta data inspector for rust


# Supported formats

- [APNG](https://en.wikipedia.org/wiki/APNG)
- [BMP](https://en.wikipedia.org/wiki/BMP_file_format)
- [GIF](https://en.wikipedia.org/wiki/GIF)
- [JPEG](https://en.wikipedia.org/wiki/JPEG)
- [PNG](https://en.wikipedia.org/wiki/Portable_Network_Graphics)
- [WebP](https://en.wikipedia.org/wiki/WebP)


# Usage

```rust,no_run
use image_meta;

fn main() {
  let meta = image_meta::load_from_file("test-files/paw.png").unwrap();
  println!("dims: {}x{}", meta.dimensions.width, meta.dimensions.width);
  println!("animation: {:?}", meta.is_animation());
  println!("format: {:?}", meta.format);
}
```
