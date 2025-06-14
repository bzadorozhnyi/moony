# Cartoonify CLI

Developed a command-line tool that applies a cartoon-like effect to images via k-means color clustering. Used `rayon` for parallelism, `palette` for color space conversion, `image` crate for manipulation, `clap` for CLI.

# Usage

```
cargo run --release -- \
  --input input.jpg \
  --output output.jpg \
  --clusters 10 \
  --seed 0 \
  --max-iters 10 \
  --converge 255.0 \
  --max-threads 4
```

# Examples

[Owl's original image source](https://www.pexels.com/photo/brown-white-and-grey-owl-perching-on-grey-log-59996/)

![owl](https://github.com/user-attachments/assets/70f6338c-deaf-4486-8b06-77457500661a)

[Sakura's original image source](https://www.pexels.com/photo/brown-white-and-grey-owl-perching-on-grey-log-59996/)

![sakura-resized](https://github.com/user-attachments/assets/5adbe0f2-a5d7-4f23-89eb-ce79d416967e)
