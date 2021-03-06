This is a JPEG encoding library written in Rust. It is a direct port of
[serge-rgb/TinyJPEG](https://github.com/serge-rgb/TinyJPEG).

I mainly wrote it to better understand the Rust language. It could be adapted
for production, but isn't ready for that out of the box.

The public interface of this library is as follows:

```rust
pub enum Quality {
    Medium,
    High,
    Highest,
}


/// Takes bitmap data and writes a JPEG-encoded image to disk at the highest
/// quality.
pub fn encode_to_file(dest: &Path, w: i32, h: i32, num_components: i32,
                      data: &[u8])
                      -> Result<(), io::Error>


/// Takes bitmap data and writes a JPEG-encoded image to disk at the specified
/// quality.
pub fn encode_to_file_at_quality(dest: &Path, quality: Quality, w: i32, h: i32,
                                 num_components: i32, data: &[u8])
                                 -> Result<(), io::Error>


/// Returns a JPEG-encoded buffer, given bitmap data
pub fn encode_to_buffer(quality: Quality, w: i32, h: i32, num_components: i32,
                        data: &[u8])
                        -> Vec<u8>
```

The following things should be added before using it in production:

1. *Testing* - Currently there is only one test, in which we encode a white
   texture and write it to file. We manually write a small C program and encode
   this same texture to another JPEG file using the original TinyJPEG. We
   manually take a binary diff of the two JPEG files and ensure that they are
   the same. Clearly, better testing can be devised.
2. *Optimization* - This version is marginally slower than the original. There
   may be some low-hanging fruit, such as pre-reserving memory for the output
   `Vec<u8>`. It may be possible to run faster than the original TinyJPEG by
    exploring concurrent processing, which the original does not do.
3. *Error-handling* - Currently we only return a potential `io::Error`. It will
   be useful to implement a custom Error type, and also handle other cases, such
   as invalid parameters.

All the code is in the public domain, just like the original TinyJPEG. I would
encourage you to fork this library and improve it for production.
