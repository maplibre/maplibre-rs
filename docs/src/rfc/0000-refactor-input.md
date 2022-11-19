- Feature Name: (`refract_input`)
- Start Date: (2022-11-13)
- RFC PR: [maplibre/maplibre-rs#00??](https://github.com/maplibre/maplibre-rs/pull/00??)
- maplibre-rs Issue: #91 and otherrs [maplibre/maplibre-rs#0000](https://github.com/maplibre/maplibre-rs/issues/91)

# Summary

[summary]: #summary

One paragraph explanation of the feature.

Refract the input modules:
* A) Implement an event loop. Consider, in maplibre-gl-js the loop is part of the map-camera. (Issue #91)
* B) Check, if/what variations of winit, we need.
* C) Refract the splitting of the input controller functionality in modules/source-codes, may be like maplibre-gl-js (Issue #91)
* D) Define and document the function of the default input-controller, equaly to the input UI of maplibre-gl-js (Issue #186)
* E) Sepatate the input-controller in an extra Rust module/crate (Issue #185)

Should all this realy be in one RFC? Separate A) in an extra RFC? And B)? Oh! RFCs are ment for users of maplibre-rs only, not for internal development! That means, only C) and E) remains?

# Motivation

[motivation]: #motivation

Why are we doing this? What use cases does it support? What is the expected outcome?

A) At the current version, each input module does the map move/slide itselve.
If we do it in the map-module, maintaining the inputs and writing a custom imput controler would be les complex.

B) Will the default winit work for all targets? - Todo: maxammann

C) The input module, which handles key presses and other inputs is quite cluttered. That means that, the logic in order to update the libraries state is quite complicated. We have to decide, how to split. Just one input module for key, mouse and touch, calling the map-API (see B) would be simpler. 

D) The user of maplibre-rs sould "feel at home", have the identical UI expirience, while using the input controls.

E) Actually, a fork of the whole repository is needed to build a individual changed application.
If we offer two extra rust crates for rendering and for control/UI, just the UI could be forked and changed.


# Guide-level explanation        ???Detailed design???

[guide-level-explanation]: #guide-level-explanation

Explain the proposal as if it was already included in the project and you were teaching it to another maplibre-rs user.
That generally means:

- Introducing new named concepts.
- Explaining the feature largely in terms of examples.
- Explaining how maplibre-rs users should *think* about the feature, and how it should impact the way they use
  maplibre-rs. It should explain the impact as concretely as possible.
- If applicable, provide sample error messages, deprecation warnings, or migration guidance.
- If applicable, describe the differences between teaching this to existing maplibre-rs users and new maplibre-rs users.

For implementation-oriented RFCs (e.g. for internals), this section should focus on how maplibre-rs contributors should
think about the change, and give examples of its concrete impact (= A,B,C)

A)
The map should offers an API, a function or a message queue to set the new map view, both imidiately and like an intrpolating move of the camera. We could do it like in maplibre-gl-js, there is only a single API, setting all values at once: position, angle, zoom.
The developer of a new control don't need to set or interpolate the camera but just calls that API.

B) Todo: maxammann

C)
At the monent, there is no concept, what input handler sould do what inputs and actions. If we do it like maplibre-gl-js,
there would be one handler for one input type. A keyboard handler would process all keybpard actions, change and set all values: position, angle, zoom.

D) If a user knows maplibre-gl-js, one should expirience and find in the documentation exactly the same input UI.
All keyboard, mouse and touch input actions should be descripted and coded equal to maplibre-gl-js. We then could extend the UI and i.e. add keys for rotaton etc.

E) The user of maplibre-rs should find
1) a crate to make the map visible, but don't need to se its code, just a well descripted API
2) a crate to control the map as an example to adapt the Rust code as wiched.
3) and examples to build applications for different targets.


# Reference-level explanation

[reference-level-explanation]: #reference-level-explanation

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

The section should return to the examples given in the previous section, and explain more fully how the detailed
proposal makes those examples work.

# Drawbacks

[drawbacks]: #drawbacks

Why should we *not* do this?

# Rationale and alternatives

[rationale-and-alternatives]: #rationale-and-alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?

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

# Unresolved questions

[unresolved-questions]: #unresolved-questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before stabilization?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of
  the solution that comes out of this RFC?

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