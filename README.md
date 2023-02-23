A fast non-cryptographic random number generator.

The random number generator has a state space of size `2**128 - 1` and also
a period of length `2**128 - 1`.

# Design

## The State Space and Output Space

Like many other random number generator designs, this one can be viewed of
as a combination of two components: a state transition function and an
output function. Let `U` be the state space and `V` be the output space.
Then with the two functions

```text
f : U -> U
g : U -> V
```

the `i`th state and output are

```text
u_i = f(f(... f(u_0)))
      \_______/
       i times

v_i = g(u_i)
```

respectively. In our case, the state space is `NonZeroU128` and the
output space is `u64`.

```text
f : NonZeroU128 -> NonZeroU128
g : NonZeroU128 -> u64
```

The size of the state space was chosen because 64 bits is too small for
some plausible applications, while 128 bits should be sufficient for almost
all non-cryptographic purposes.

## The State Transition Function and its Period

The state transition function is a member of `GL(128, 2)`, that is, it is
an invertible linear transformation from the vector space of dimension 128
over the finite field of order 2 to itself.

In order to see that `f` is invertible, note that ...

TODO

Checking that `f` has period `2**128 - 1` takes a bit of computation. Let
`A` be the binary matrix corresponding to `f`. We can take `A` to the power
of `2**128 - 1` using `O(log(n))` exponentiation and verify that it is the
identity matrix.

Also, we can factor `2**128 - 1` first into a product of Fermat numbers and
then into a product of primes.

```text
2**128 - 1 = (2**1 + 1) (2**2 + 1) (2**4 + 1) (2**8 + 1) (2**16 + 1) (2**32 + 1)
           = 3 * 5 * 17 * 257 * 65537 * 641 * 6700417
```

Then it is sufficient to check that `A ** ((2**128 - 1) / p_i)` is *not*
the identity for each prime factor `p_i` and to recall some elementary
facts about finite groups.

## The Output Function

## A Survey of Alternate State Transition Functions

- counter

- LCG

- LFSR

- xorshift & co

- approximating a random invertible transition

## A Survey of Alternate Output Functions

- projection

- xor, add

- hash mixer

## Comparisons with Selected RNGs

- pcg64-dxsm

- xoroshiro128++

- lxm-l64x128

- mwc256xxa64
