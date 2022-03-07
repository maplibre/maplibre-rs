# Developer Resources

## Next Major Goals

* Improve buffer_pool eviction rules
* Use MPI: https://doc.rust-lang.org/book/ch16-02-message-passing.html
* Input-handling via events and functional pipelines
* Show old tiles until new tile is ready / Show mixed tiles, based on availability
* Use a simple style definition
  * type: background/fill/line
  * minzoom/maxzoom
  * source
  * source-layer
  * paint (fill-color)
* Map feeling:
  * Wrap world around in x direction
  * Limit panning in y direction
  * Nicer default map style

## Intermediate Goals
* Support multiple projections? PoC such that we are sure the renderer is acceptable

## Future Goals
* Very simple text rendering
* Cache tessellation results
    * We have three "caches": downloaded tiles, tessellated tiles, gpu tiles
* Handle missing tiles
* Support different tile raster addressing

