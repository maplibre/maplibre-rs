- Start Date: 2022-10-29
- RFC PR: [maplibre/maplibre-rs#??](https://github.com/maplibre/maplibre-rs/pull/??)
- maplibre-rs Issue: N/A

# Summary

The "RFC" (request for comments) process is intended to provide a
consistent and controlled path for new features to enter the map renderer
and standard libraries, so that all stakeholders can be confident about
the direction the map renderer is evolving in.

This RFC is copied from [rust-lang/rfcs](https://github.com/rust-lang/rfcs).
Thanks to everyone who contributed to the Rust RFC process RFC! Thanks for the
great inspiration!

# Motivation

The freewheeling way that we add new features to maplibre-rs has been good for
early development, but for maplibre-rs to become a mature platform we need to
develop some more self-discipline when it comes to changing the system.
This is a proposal for a more principled RFC process to make it
a more integral part of the overall development process, and one that is
followed consistently to introduce features to maplibre-rs.

# Detailed design

Many changes, including bug fixes and documentation improvements can be
implemented and reviewed via the normal GitHub pull request workflow.

Some changes though are "substantial", and we ask that these be put
through a bit of a design process and produce a consensus among the maplibre-rs
community.

## When you need to follow this process

You need to follow this process if you intend to make "substantial"
changes to the maplibre-rs distribution. What constitutes a "substantial"
change is evolving based on community norms, but may include the following.

- Any semantic or syntactic change to the map renderer that is not a bugfix.
- Removing map renderer features, including those that are feature-gated.

Some changes do not require an RFC:

- Rephrasing, reorganizing, refactoring, or otherwise "changing shape
  does not change meaning".
- Additions that strictly improve objective, numerical quality
  criteria (warning removal, speedup, better platform coverage, more
  parallelism, trap more errors, etc.)
- Additions only likely to be _noticed by_ other developers-of-maplibre-rs,
  invisible to users-of-maplibre-rs.

If you submit a pull request to implement a new feature without going
through the RFC process, it may be closed with a polite request to
submit an RFC first.

## What the process is

In short, to get a major feature added to maplibre-rs, one must first get the
RFC merged into the RFC repo as a markdown file. At that point the RFC
is 'active' and may be implemented with the goal of eventual inclusion
into maplibre-rs.

* Fork the repo https://github.com/maplibre/maplibre-rs
* Copy `rfc/0000-template.md` to `rfc/0000-my-feature.md` (where
  'my-feature' is descriptive. don't assign an RFC number yet).
* Fill in the RFC
* Submit a pull request. The pull request is the time to get review of
  the design from the larger community.
* Build consensus and integrate feedback. RFCs that have broad support
  are much more likely to make progress than those that don't receive any
  comments.

Eventually, somebody on the maplibre-rs team will either accept the RFC by
merging the pull request, at which point the RFC is 'active', or
reject it by closing the pull request.

Who ever merges the RFC should do the following:

* Assign an id, using the PR number of the RFC pull request. (If the RFC
  has multiple pull requests associated with it, choose one PR number,
  preferably the minimal one.)
* Add the file in the `docs/src/rfc/` directory.
* Create a corresponding issue on [maplibre-rs repo](https://github.com/maplibre/maplibre-rs)
* Fill in the remaining metadata in the RFC header, including links for
  the original pull request(s) and the newly created maplibre-rs issue.
* Communicate that given RFC is now considered 'active' on the chat platform of maplibre-rs.
* Commit everything.

Once an RFC becomes active then authors may implement it and submit the
feature as a pull request to the maplibre-rs repo. An 'active' is not a rubber
stamp, and in particular still does not mean the feature will ultimately
be merged; it does mean that in principle all the major stakeholders
have agreed to the feature and are amenable to merging it.

Modifications to active RFC's can be done in followup PR's. An RFC that
makes it through the entire process to implementation is considered
'complete'; an RFC that fails after becoming active is 'inactive'.

# Alternatives

Retain the current informal RFC process. The newly proposed RFC process is
designed to improve over the informal process in the following ways:

* Discourage non-actionable or vague RFCs
* Ensure that all serious RFCs are considered equally
* Give confidence to those with a stake in maplibre-rs development that they
  understand why new features are being merged

As an alternative, we could adopt an even stricter RFC process than the one proposed here. 
If desired, we should likely look to Python's [PEP] process for inspiration.

# Unresolved questions

1. Does this RFC strike a favorable balance between formality and agility?
2. Does this RFC successfully address the aforementioned issues with the current
   informal RFC process?
3. Should we retain rejected RFCs in the archive?

[PEP]: http://legacy.python.org/dev/peps/pep-0001/
