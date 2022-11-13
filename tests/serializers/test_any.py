import json

import pytest

from pydantic_core import SchemaSerializer, core_schema


@pytest.mark.parametrize('value', [None, 1, 1.0, True, 'foo', [1, 2, 3], {'a': 1, 'b': 2}])
def test_any_json_round_trip(value):
    v = SchemaSerializer(core_schema.any_schema())
    assert v.to_python(value) == value
    assert v.to_python(value, format='json') == value
    assert json.loads(v.to_json(value)) == value
