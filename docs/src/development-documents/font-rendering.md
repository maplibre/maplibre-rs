# Font Rendering

There exists no universally perfect solution to font rendering. Depending on the runtime environment a method needs to
be chosen. [This StackOverflow post](https://stackoverflow.com/a/5278471) outlines some state-of-the-art methods. Some
more approaches are described [here](https://aras-p.info/blog/2017/02/15/Font-Rendering-is-Getting-Interesting/).

From my perspective the following approaches could work potentially:

1. Tessellate Fonts
2. SDF Font Rendering
3. GPU Text Rendering directly from Bezier Curves
4. Draw text using a Web Canvas and load them to GPU

There is a thesis which summarizes some methods [here](https://lup.lub.lu.se/luur/download?func=downloadFile&recordOId=9024910&fileOId=9024911).
A link collection about font related projects can be viewed [here](../appendix/link-collection.md#font-rendering).

## Approaches

### Tessellate Fonts
There is [ttf2mesh](https://github.com/fetisov/ttf2mesh) which generates meshes. I was already able to generate about 1k
glyphs with ~40FPS.

### SDF Font Rendering

There is a blogpost by Mapbox [here](https://blog.mapbox.com/drawing-text-with-signed-distance-fields-in-mapbox-gl-b0933af6f817).
Some more implementation documents are available [here](https://github.com/mapbox/mapbox-gl-native/wiki/Text-Rendering).
A good foundation for SDF fonts was created by Chlumsky with [msdfgen](https://github.com/Chlumsky/msdfgen).


### GPU Text Rendering from Bezier Curves

The solutions exist:

* [By Will Dobbie](https://wdobbie.com/post/gpu-text-rendering-with-vector-textures/) with an
  implementation [here](https://github.com/azsn/gllabel)
* [Slug Library](http://sluglibrary.com/) which is patented and probably therefore not usable

[Here](https://jcgt.org/published/0006/02/02/paper.pdf) is the whitepaper of the Slug library. There is also
a [poster](http://sluglibrary.com/slug_algorithm.pdf) about it. There also exists
an [open implementation](https://github.com/mightycow/Sluggish).

### Draw text using a Web Canvas

This approach has the downside that we can not dynamically scale rendered fonts according to the current zoom level.


## Other Approaches

* [16x AA font rendering using coverage masks](https://superluminal.eu/16x-aa-font-rendering-using-coverage-masks-part-iii/)