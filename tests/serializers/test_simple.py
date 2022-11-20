import json

import pytest

from pydantic_core import SchemaSerializer


class IntSubClass(int):
    pass


class FloatSubClass(float):
    pass


@pytest.mark.parametrize(
    'schema_type,value,expected_python,expected_json',
    [
        ('int', 1, 1, b'1'),
        ('bool', True, True, b'true'),
        ('bool', False, False, b'false'),
        ('float', 1.0, 1.0, b'1.0'),
        ('float', 42.31415, 42.31415, b'42.31415'),
        ('none', None, None, b'null'),
        ('int', IntSubClass(42), IntSubClass(42), b'42'),
        ('float', FloatSubClass(42), FloatSubClass(42), b'42.0'),
        # same with any as type
        ('any', 1, 1, b'1'),
        ('any', True, True, b'true'),
        ('any', False, False, b'false'),
        ('any', 1.0, 1.0, b'1.0'),
        ('any', 42.31415, 42.31415, b'42.31415'),
        ('any', None, None, b'null'),
        ('any', IntSubClass(42), IntSubClass(42), b'42'),
        ('any', FloatSubClass(42), FloatSubClass(42), b'42.0'),
    ],
)
def test_simple_serializers(schema_type, value, expected_python, expected_json):
    s = SchemaSerializer({'type': schema_type})
    v = s.to_python(value)
    assert v == expected_python
    assert type(v) == type(expected_python)

    assert s.to_json(value) == expected_json

    v_json = s.to_python(value, format='json')
    v_json_expected = json.loads(expected_json)
    assert v_json == v_json_expected
    assert type(v_json) == type(v_json_expected)


@pytest.mark.parametrize('schema_type', ['int', 'bool', 'float', 'none'])
def test_simple_serializers_fallback(schema_type):
    s = SchemaSerializer({'type': schema_type})
    assert s.to_python([1, 2, 3]) == [1, 2, 3]

    # with pytest.warns(UserWarning, match=f'Expected `{schema_type}` but got `list` - slight slowdown possible'):
    #     assert s.to_python([1, 2, 3], format='json') == [1, 2, 3]

    with pytest.warns(UserWarning, match=f'Expected `{schema_type}` but got `list` - slight slowdown possible'):
        assert s.to_json([1, 2, 3]) == b'[1,2,3]'
