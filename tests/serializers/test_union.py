import json

import pytest

from pydantic_core import SchemaSerializer, core_schema


@pytest.mark.parametrize('input_value,expected_value', [(True, True), (False, False), (1, 1), (123, 123), (-42, -42)])
def test_union_bool_int(input_value, expected_value):
    v = SchemaSerializer(core_schema.union_schema(core_schema.bool_schema(), core_schema.int_schema()))
    assert v.to_python(input_value) == expected_value
    assert v.to_python(input_value, mode='json') == expected_value
    assert v.to_json(input_value) == json.dumps(expected_value).encode()


def test_union_error():
    v = SchemaSerializer(core_schema.union_schema(core_schema.bool_schema(), core_schema.int_schema()))
    assert v.to_python('a string') == 'a string'
