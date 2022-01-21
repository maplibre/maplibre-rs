# Appendix

## Challenges:

* Accuracy of floating point numbers is very bad for big world view
  coordinates ([Plot](https://en.wikipedia.org/wiki/IEEE_754#/media/File:IEEE754.svg))
* [Perils of World Space](https://paroj.github.io/gltut/Positioning/Tut07%20The%20Perils%20of%20World%20Space.html)

### Create paths for tesselating streets

Streets can have unusual shaped like shown [here](https://www.google.de/maps/@48.1353883,11.5717232,19z) in Munich. OSM
does not offer such data and therefore just renders an ordinary street contour like
shown [here](https://www.openstreetmap.org/query?lat=48.13533&lon=11.57143).
Because the data is probably not available this is a very hard challenge.

## Future Ideas

* Use [rust-gpu](https://github.com/EmbarkStudios/rust-gpu) as shading language
* Focus on accessibility of maps: https://www.w3.org/WAI/RD/wiki/Accessible_Maps
* Display in AR: https://developer.apple.com/documentation/arkit/displaying_an_ar_experience_with_metal

## Debugging Rendering

For WebGL there is SpectorJS is enabled by default right now. For debugging on a desktop environment you can use
[RenderDoc](https://renderdoc.org/).