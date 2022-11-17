import json

import pytest

from pydantic_core import SchemaSerializer, core_schema


@pytest.mark.parametrize(
    'value,expected_python,expected_json',
    [(None, 'None', b'"None"'), (1, '1', b'"1"'), ([1, 2, 3], '[1, 2, 3]', b'"[1, 2, 3]"')],
)
def test_function(value, expected_python, expected_json):
    def repr_function(value, **kwargs):
        return repr(value)

    s = SchemaSerializer(core_schema.any_schema(serialization={'function': repr_function}))
    assert s.to_python(value) == expected_python
    assert s.to_json(value) == expected_json
    assert s.to_python(value, format='json') == json.loads(expected_json)


def test_function_args():
    f_kwargs = None

    def double(value, **kwargs):
        nonlocal f_kwargs
        f_kwargs = kwargs
        return value * 2

    s = SchemaSerializer(core_schema.any_schema(serialization={'function': double}))
    assert s.to_python(4) == 8
    assert f_kwargs == {'format': 'python', 'include': None, 'exclude': None}
    assert s.to_python('x') == 'xx'

    assert s.to_python(4, format='foobar') == 8
    assert f_kwargs == {'format': 'foobar', 'include': None, 'exclude': None}

    assert s.to_json(42) == b'84'
    assert f_kwargs == {'format': 'json', 'include': None, 'exclude': None}

    assert s.to_python(7, format='json') == 14
    assert f_kwargs == {'format': 'json', 'include': None, 'exclude': None}

    assert s.to_python(1, include={1, 2, 3}, exclude={'foo': {'bar'}}) == 2
    assert f_kwargs == {'format': 'python', 'include': {1, 2, 3}, 'exclude': {'foo': {'bar'}}}
