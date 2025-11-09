
# AnyTrait

This is a **no_std** crate that lets you cast from:
* your concrete type
* `&dyn AnyTrait`
to:
* the concrete type
* any other trait implemented by your type
* `&dyn AnyTrait`

If the trait implements `AnyTrait`, that too can be cast to:
* the concrete type
* any other trait implemented by your type
* `&dyn AnyTrait`

*This is not zero-cost, since at any cast we need to go through the
list of all possible subtraits.*

This will (almost) enable you to do OOP in rust, but if this is your goal
we still ask you to kindly reconsider

### License

EUPL 1.2: tl;dr: non-viral GPLv3.
compatible with GPL*/apache/mit/proprietary.
check the [official compatibility](https://interoperable-europe.ec.europa.eu/collection/eupl/matrix-eupl-compatible-open-source-licences) for doubts

## Does this work?

The tests do. `cargo miri test` does not complain. Looks sound to me.
Check the tests for examples, too.
This crate is brand new and I did not have the time to actually use this anywhere.

## Why?

Because we wanted to bring the eldritch horrors of half-done OOP to Rust.
Honestly, there are use cases when this is useful, but we still advise you to think twice and try to limit yourself to more normal rust.

This is not a zero-cost abstraction: we keep a 'static list of available traits and we have to go through that at each cast

## How does this work?

For every type we build a static list of traits you are allowed to downcast to.
Every concrete that derives `AnySubTrait` will get a cast functions
that does type-erasure.\
A generic implementation in `AnyTraitCast` will then remove that type-erasure
by making sure we are casting back to the correct type.


All of this means that this is not free.
We have to walk through the list of enabled traits, and if you have lots and lots of traits that can be expensive.
The list is not odered either, rust does not have `const Ord` on TypeId right now.

## Is this safe?

Yes. You will only use `.as_anytrait()` and `.cast_ref::<dyn MyTrait>()` which are completely safe.

There are a couple of `unsafe` functions, but they are only used internally and if you try to use them we will find your employer and convince them to make you switch to brainfuck in production.


# How does this compare to....

Honestly, I built this before realizing there already were a few options out there.
but there are some differences:

* we are `no_std`
* we require unstable rust since we rely on const-comparison on `TypeId`.
* we have no global state/registry
* we rely on how rust implements fat pointer for type-erasure

Not having global state might make us faster when the number of traits grows
a lot, but to be really performant there we require a const-Ord on `TypeId`,
which is not there yet.

## crate `intertrait`/`traitcast`?

They need the `std` library. We are `no_std`.

They work on stable rust, we require unstable for now.

They use a global registry with `Hashmap`.\
We generate a list of supported subtraits for each concrete type.
This is usually much smaller and for now we just walk through it.

Our type-erasure is based on how rust implements fat pointers,
so it might have to change in the future, although it seems unlikely right now.
