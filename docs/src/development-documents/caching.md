# Caching

The caching for maplibre-rs is handled on the networking layer. This means that data which is fetched over slow IO is cached in
the format of the network requests. The maplibre-rs library is not introducing a separate serialization format for caching.

Instead, caching functionality of HTTP client libraries of the web platform are used. This has the advantage that we can
honor HTTP headers which configure caching. This is very important for fetched tiles, as they can have an expiry date.

* On the web the browser is automatically caching raw tiles.
* On Linux, macOS, iOS and Android we are
  utilizing [reqwest-middleware-cache](https://crates.io/crates/reqwest-middleware-cache/), which writes raw network
  requests to disk.
