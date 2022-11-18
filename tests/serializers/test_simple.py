import json

import pytest

from pydantic_core import SchemaSerializer


@pytest.mark.parametrize(
    'schema_type,value,expected_python,expected_json',
    [
        ('int', 1, 1, b'1'),
        ('bool', True, True, b'true'),
        ('bool', False, False, b'false'),
        ('float', 1.0, 1.0, b'1.0'),
        ('float', 42.31415, 42.31415, b'42.31415'),
        ('none', None, None, b'null'),
    ],
)
def test_simple_serializers(schema_type, value, expected_python, expected_json):
    s = SchemaSerializer({'type': schema_type})
    assert s.to_python(value) == expected_python
    assert s.to_json(value) == expected_json
    assert s.to_python(value, format='json') == json.loads(expected_json)


@pytest.mark.xfail(reason='TODO to_python(..., format="json")')
@pytest.mark.parametrize('schema_type', ['int', 'bool', 'float', 'none'])
def test_simple_serializers_fallback(schema_type):
    s = SchemaSerializer({'type': schema_type})
    assert s.to_python([1, 2, 3]) == [1, 2, 3]

    with pytest.warns(UserWarning, match=f'Expected `{schema_type}` but got `list` - slight slowdown possible'):
        assert s.to_python([1, 2, 3], format='json') == [1, 2, 3]

    with pytest.warns(UserWarning, match=f'Expected `{schema_type}` but got `list` - slight slowdown possible'):
        assert s.to_json([1, 2, 3]) == b'[1,2,3]'
