import json

import pytest

from pydantic_core import PydanticSerializationError, SchemaSerializer, core_schema


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


def test_function_error():
    def raise_error(value, **kwargs):
        raise TypeError('foo')

    s = SchemaSerializer(core_schema.any_schema(serialization={'function': raise_error}))

    with pytest.raises(RuntimeError, match='Error calling `raise_error`: TypeError: foo'):
        s.to_python('abc')


def test_function_known_type():
    def append_42(value, **kwargs):
        if isinstance(value, list):
            value.append(42)
        return value

    s = SchemaSerializer(core_schema.any_schema(serialization={'function': append_42, 'type': 'list'}))
    assert s.to_python([1, 2, 3]) == [1, 2, 3, 42]
    assert s.to_python([1, 2, 3], format='json') == [1, 2, 3, 42]
    assert s.to_json([1, 2, 3]) == b'[1,2,3,42]'

    assert s.to_python('abc') == 'abc'

    with pytest.raises(TypeError, match="'str' object cannot be converted to 'PyList'"):
        s.to_python('abc', format='json')

    msg = "Error serializing to JSON: 'str' object cannot be converted to 'PyList'"
    with pytest.raises(PydanticSerializationError, match=msg):
        s.to_json('abc')
