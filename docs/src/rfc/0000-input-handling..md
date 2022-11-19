- Feature Name: (`input_handling`)
- Start Date: (2022-11-13)
- RFC PR: [maplibre/maplibre-rs#00??](https://github.com/maplibre/maplibre-rs/pull/00??)
- maplibre-rs Issue: 
[maplibre/maplibre-rs#186](https://github.com/maplibre/maplibre-rs/issues/91) 
[maplibre/maplibre-rs#185](https://github.com/maplibre/maplibre-rs/issues/185) [
[maplibre/maplibre-rs#186](https://github.com/maplibre/maplibre-rs/issues/186)

# Summary

[summary]: #summary

This RFC describes how we want to handle input in maplibre-rs. From a user perspective, input means that certain key 
presses, screen touches or mouse movements translate into actions, which change how the map data is rendered.

We call these inputs *input events*. These events can trigger so-called *actions*. Actions directly modify the state of
the renderer.

This RFC discusses various aspects of input handling:
1. **Event Loop** Acknowledge the presence of an Event Loop (processes *input events*)
2.  **Platform Abstractions** Define how we can define abstractions over the platform's input system
3. **User-Friendly Inputs** Define what inputs end-users expect they can perform 


Input aspects:
* **Event Loop** Implement an event loop. Consider, in `maplibre-gl-js` the loop is part of the map-camera. (Issue #91)
* **Platform Abstractions** Check, if/what variations of `winit`, we need.
* **Modularize Event Handlers** Refract the splitting of the input controller functionality in modules/source-codes, may be like `maplibre-gl-js` (Issue #91)
* **User-Friendly Inputs** Define and document the function of the default input-controller, equally to the input UI of `maplibre-gl-js` (Issue #186)
* **???** Separate the input-controller in an extra Rust module/crate (Issue #185) -maxammann: Is that maybe related to **Modularize Event Handlers**?

# Motivation

[motivation]: #motivation

Why are we doing this? What use cases does it support? What is the expected outcome?

### Event Loop

At the current version, each input module does the map move/slide itself.
If we do it in the map-module, maintaining the inputs and writing a custom input controller would be less complex.
The map should offer an API, a function or a message queue to set the new map view, both immediately and like an interpolating move of the camera. We could do it like in `maplibre-gl-js`, there is only a single API, setting all values at once: position, angle, zoom.
The developer of a new control don't need to set or interpolate the camera but just calls that API.

maxammann: An event loop is required because of the Rust ownership model we can not change the state of the renderer at an arbitrary point. Event a `Rc<RefCell<MapState>>` would not change that because we can never be certain if the render loop is currently borrowing the map state.

maxammann: Therefore, we must acknowledge that the only way how we can change the map state is through events in an event loop.

### Platform Abstractions

Will the default `winit` work for all targets? - Todo: maxammann: No, it does not work for our usecase on iOS and Android right now.

### Modularize Event Handlers

The input module, which handles key presses and other inputs is quite cluttered. 
That means that, the logic in order to update the libraries state is quite complicated. 
We have to decide, how to split. Just one input module for key, mouse and touch, calling the map-API (see B) would be simpler. 
At the moment, there is no concept, what input handler would do what inputs and actions. 
If we do it like `maplibre-gl-js`, there would be one handler for one input type. 
A keyboard handler would process all keyboard actions, change and set all values: position, angle, zoom.

### User-Friendly Inputs

The user of maplibre-rs should "feel at home", have the identical UI experience, while using the input controls.
If a user knows `maplibre-gl-js`, one should experience and find in the documentation exactly the same input UI.
All keyboard, mouse, and touch input actions should be described and coded equal to `maplibre-gl-js`. We then could extend the UI and i.e. add keys for rotation etc.

### ???

Actually, a fork of the whole repository is needed to build a individual changed application.
If we offer two extra rust crates for rendering and for control/UI, just the UI could be forked and changed.

The user of maplibre-rs should provide
1. a crate to make the map visible, but don't need to see its code, just a well described API,
2. a crate to control the map as an example to adapt the Rust code as wished, and
3. examples to build applications for different targets.

# Guide-level explanation

[guide-level-explanation]: #guide-level-explanation

Explain the proposal as if it was already included in the project, and you were teaching it to another maplibre-rs user.
That generally means:

- Introducing new named concepts.
- Explaining the feature largely in terms of examples.
- Explaining how maplibre-rs users should *think* about the feature, and how it should impact the way they use
  maplibre-rs. It should explain the impact as concretely as possible.
- If applicable, provide sample error messages, deprecation warnings, or migration guidance.
- If applicable, describe the differences between teaching this to existing maplibre-rs users and new maplibre-rs users.


TODO


# Reference-level explanation

[reference-level-explanation]: #reference-level-explanation

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

The section should return to the examples given in the previous section, and explain more fully how the detailed
proposal makes those examples work.

TODO

# Drawbacks

[drawbacks]: #drawbacks

Why should we *not* do this?

TODO

# Rationale and alternatives

[rationale-and-alternatives]: #rationale-and-alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?

TODO

# Prior art

[prior-art]: #prior-art

Discuss prior art, both the good and the bad, in relation to this proposal.
A few examples of what this can include are:

- Does this feature exist in other map renderers and what experience have their community had?
- For community proposals: Is this done by some other community and what were their experiences with it?
- For other teams: What lessons can we learn from what other communities have done here?
- Papers: Are there any published papers or great posts that discuss this? If you have some relevant papers to refer to,
  this can serve as a more detailed theoretical background.

This section is intended to encourage you as an author to think about the lessons from other renderers, provide readers
of your RFC with a fuller picture.
If there is no prior art, that is fine - your ideas are interesting to us whether they are brand new or if it is an
adaptation from other map renderers.

Note that while precedent set by other map renderers is some motivation, it does not on its own motivate an RFC.

TODO

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before stabilization?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of
  the solution that comes out of this RFC?

TODO

# Future possibilities

[future-possibilities]: #future-possibilities

Think about what the natural extension and evolution of your proposal would
be and how it would affect the renderer and project as a whole in a holistic
way. Try to use this section as a tool to more fully consider all possible
interactions with the project and renderer in your proposal.
Also consider how this all fits into the roadmap for the project
and of the relevant sub-team.

This is also a good place to "dump ideas", if they are out of scope for the
RFC you are writing but otherwise related.

If you have tried and cannot think of any future possibilities,
you may simply state that you cannot think of anything.

Note that having something written down in the future-possibilities section
is not a reason to accept the current or a future RFC; such notes should be
in the section on motivation or rationale in this or subsequent RFCs.
The section merely provides additional information.

TODO
