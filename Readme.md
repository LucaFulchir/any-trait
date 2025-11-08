
# AnyTrait

This is a **no_std** crate that lets you upcast from a concrete type to a `AnyTrait`, and then downcast to either:
* the concrete type
* any other trait implemented by your type

If the trait implements `AnyTrait`, that can be cast to `AnyTrait`, too.

### License

EUPL 1.2: tl;dr: non-viral GPLv3.
compatible with GPL*/apache/mit/proprietary.
check the [official compatibility](https://interoperable-europe.ec.europa.eu/collection/eupl/matrix-eupl-compatible-open-source-licences) for doubts

## Does this work?

The test does. Check there for examples, too.
This crate is brand new and I did not have the time to actually use this anywhere.


## Why?

Because we wanted to bring the eldritch horrors of half-done OOP to Rust.
Honestly, there are use cases when this is useful, but we still advise you to think twice and try to limit yourself to more normal rust.

This is not a zero-cost abstraction: we keep a 'static list of available traits and we have to go through that at each cast

## How does this work?

For every type we build a static list of traits you are allowed to downcast to.
With the power of eldritch casting/reinterpret horrors we cast your concrete type to a `usize`. That is then cast back to the trait you requested by the `AnyTrait`

All of this means that this is not free.
We have to walk through the list of enabled traits, and if you have lots and lots of traits that can be expensive.
The list is not odered either, rust does not have `const Ord` on TypeId right now.

## Is this safe?

Yes. You will only use `.as_anytrait()` and `.downcast_ref::<dyn MyTrait>()` which are completely safe.

There are a couple of `unsafe` functions, but they are only used internally and if you try to use them we will find your employer and convince them to make you switch to brainfuck in production.
