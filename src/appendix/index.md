# Appendix


## Goals

### Next Major Goals

* ~~Improve buffer_pool eviction rules~~
* ~~Use MPI: https://doc.rust-lang.org/book/ch16-02-message-passing.html~~
* Input-handling via events and functional pipelines
* ~~Show old tiles until new tile is ready / Show mixed tiles, based on availability~~
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

### Intermediate Goals
* Support multiple projections? PoC such that we are sure the renderer is acceptable

### Future Goals
* Very simple text rendering
* Cache tessellation results
    * We have three "caches": downloaded tiles, tessellated tiles, gpu tiles
* Handle missing tiles
* Support different tile raster addressing

## Future Ideas

* Use [rust-gpu](https://github.com/EmbarkStudios/rust-gpu) as shading language
* Focus on accessibility of maps: https://www.w3.org/WAI/RD/wiki/Accessible_Maps
* Display in AR: https://developer.apple.com/documentation/arkit/displaying_an_ar_experience_with_metal
* Use tracing framework: [tracing](https://docs.rs/tracing/0.1.31/tracing)

## Challenges:

* Accuracy of floating point numbers is very bad for big world view
  coordinates ([Plot](https://en.wikipedia.org/wiki/IEEE_754#/media/File:IEEE754.svg))
* [Perils of World Space](https://paroj.github.io/gltut/Positioning/Tut07%20The%20Perils%20of%20World%20Space.html)

### Create paths for tessellating streets

Streets can have unusual shaped like shown [here](https://www.google.de/maps/@48.1353883,11.5717232,19z) in Munich. OSM
does not offer such data and therefore just renders an ordinary street contour like
shown [here](https://www.openstreetmap.org/query?lat=48.13533&lon=11.57143).
Because the data is probably not available this is a very hard challenge.
