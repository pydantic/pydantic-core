"""
Benchmarks for nested / recursive schemas using definitions.
"""

from typing import Callable

from pydantic_core import SchemaValidator

from .nested_schema import input_data_valid, schema


def test_nested_valid(benchmark: Callable[..., None]) -> None:
    v = SchemaValidator(schema())
    data = input_data_valid()
    v.validate_python(data)
    benchmark(v.validate_python, data)
