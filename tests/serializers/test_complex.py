import json
import math

import pytest

from pydantic_core import SchemaSerializer, core_schema


@pytest.mark.parametrize(
    'value,substr,expected',
    [
        (complex(1, 2), '"real":1.0', {'real': 1.0, 'imag': 2.0}),
        (complex(-float('inf'), 2), '"real":-Infinity', {'real': -float('inf'), 'imag': 2.0}),
        (complex(float('inf'), 2), '"real":Infinity', {'real': float('inf'), 'imag': 2.0}),
        (complex(float('nan'), 2), '"real":NaN', {'real': float('nan'), 'imag': 2.0}),
    ],
)
def test_complex_json(value, substr, expected):
    v = SchemaSerializer(core_schema.complex_schema())
    c = v.to_python(value)
    c_json = v.to_python(value, mode='json')
    json_str = v.to_json(value).decode()
    c_reloaded = json.loads(json_str)

    assert substr in json_str
    assert c.imag == expected['imag']

    if math.isnan(expected['real']):
        assert math.isnan(c.real)
        assert math.isnan(c_json['real'])
        assert math.isnan(c_reloaded['real'])
    else:
        assert c.real == expected['real']
        assert c_json['real'] == expected['real']
        assert c_reloaded['real'] == expected['real']
