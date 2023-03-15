# Stubs for pydantic-core

`pydantic-core` uses large tagged unions of recursive types. Unfortunately `mypy` is _very slow_ as soon as it encounters these sorts of types. If we put typing information in `pydantic-core` directly any user that does `pip install pydantic` would experience unacceptable slowness (hours) in analyzing even the simplest Pydantic model because `mypy` would try to analyze all of `pydantic-core`.

To address this situation we ship `pydantic-core` without types by not including a `py.typed` file. The types are shipped separately via the `pydantic-core-stubs` package.

If you are contributing to `pydantic-core` or `pydantic` you should install `pydantic-core-stubs` and either wait until `mypy` builds a cache of the types or use `pyright` as you type checker.
