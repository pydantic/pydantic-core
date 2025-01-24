from typing import Callable

from pydantic_core import gather_schemas_for_cleaning

from .nested_schema import inlined_schema, schema_using_defs


def test_nested_schema_using_defs(benchmark: Callable[..., None]) -> None:
    schema = schema_using_defs()
    definitions = {def_schema['ref']: def_schema for def_schema in schema['definitions']}
    schema = schema['schema']
    benchmark(gather_schemas_for_cleaning, schema, definitions, None)


def test_nested_schema_inlined(benchmark: Callable[..., None]) -> None:
    schema = inlined_schema()
    benchmark(gather_schemas_for_cleaning, schema, {}, {'some_meta_key'})
