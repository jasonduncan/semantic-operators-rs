# Source attribution and notices

Original project code is copyright (c) 2026 Jason Duncan and licensed under MIT
(see LICENSE). Portions retain the upstream terms below; Cargo therefore describes
the combined package as `MIT AND Python-2.0`.

## CPython accurate summation

The `accurate_sum` routines in src/answer.rs, src/rerank.rs and src/bench.rs follow
CPython's math.fsum nonoverlapping partial sums and final half-even correction.
We preserve the Python notices for these adapted portions. CPython attributes the
summation algorithm to Raymond Hettinger, with exact partial summation and roundoff
enhancements by Mark Dickinson.

Source: CPython 3.13.13, Modules/mathmodule.c, math_fsum and its preceding notes,
commit `01104ce1beb3135c2e0c01ec835b994c1f55a1c0`:
https://github.com/python/cpython/blob/01104ce1beb3135c2e0c01ec835b994c1f55a1c0/Modules/mathmodule.c

Copyright (c) 2001-2024 Python Software Foundation; All Rights Reserved.
The complete upstream license and historical notices are retained verbatim in
LICENSE-PYTHON from that same revision.

Changes: translated arithmetic into safe Rust with Vec-backed partials; removed
Python object/iterator/runtime integration; restricted inputs through validated
semantic distributions; benchmark/reranking variants report typed errors for
nonfinite values or intermediate overflow. These are local Rust routines, not a
Python runtime dependency. This attribution does not imply PSF endorsement.

## Semantic operator references and fixtures

The Python semantic-operators 0.12.0 source at
`07fb1f3109dac2e69a7dafe9fa349d0daa52012d` and C++ libsemop source at
`dc3b28f2d22ac6ca27ddcf0b7529a7082de56d82` both carry the same MIT license and
copyright (c) 2026 Jason Duncan. The Rust port preserves that notice in LICENSE.
Repository fixtures are synthetic reference outputs with pinned generator/source
hashes; there are no copied customer documents or external image assets.
