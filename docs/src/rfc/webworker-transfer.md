- Start Date: 2022-12-11
- RFC PR: [maplibre/maplibre-rs#223](https://github.com/maplibre/maplibre-rs/pull/223)
- maplibre-rs Issue: 
[maplibre/maplibre-rs#190](https://github.com/maplibre/maplibre-rs/pull/190) 
[maplibre/maplibre-rs#174](https://github.com/maplibre/maplibre-rs/pull/174)

# Summary

Rendering data in real-time requires developers to carefully decide which work to 
perform on the main rendering thread and which work can be done asynchronously.

This RFC focuses on describing how asynchronous work can be done on the Web platform, while still allowing
other platform to use other paradigms.

# Motivation

On the Web platform we do not have threads or processes available.
Instead, we have WebWorkers. WebWorkers are very similar to processes in the Unix-world.
With the recent "atomics" proposals in WebAssembly and its shared-memory support it is actually possible
to lift WebWorkers from be being processes to fully fledged threads (i.e. synchronizing on shared-memory using mutexes).

Though, using shared-memory requires
[settings special HTTP-headers](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#security_requirements)
, which limit a websites cross-site capabilities.
For this reason maplibre-rs needs a way to do work asynchronously without relying on shared-memory.
Other platforms (Linux, Android, iOS) should still be able to leverage shared-memory though.

# Detailed design



# Alternatives

1. Pod-like structure passing
2. Captnproto


# Unresolved questions

1. 

