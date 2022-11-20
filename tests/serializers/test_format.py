import json
import re
from datetime import date

import pytest

from pydantic_core import PydanticSerializationError, SchemaSerializer, core_schema


@pytest.mark.parametrize(
    'value,formatting_string,expected_python,expected_json',
    [
        (42.12345, '0.4f', '42.1234', b'"42.1234"'),
        (date(2022, 11, 20), '%Y-%m-%d', '2022-11-20', b'"2022-11-20"'),
        ('foo', '^5s', ' foo ', b'" foo "'),
    ],
)
def test_format(value, formatting_string, expected_python, expected_json):
    s = SchemaSerializer(
        core_schema.any_schema(serialization={'type': 'format', 'formatting_string': formatting_string})
    )
    assert s.to_python(value) == expected_python
    assert s.to_json(value) == expected_json
    assert s.to_python(value, mode='json') == json.loads(expected_json)


def test_format_error():
    s = SchemaSerializer(core_schema.any_schema(serialization={'type': 'format', 'formatting_string': '^5d'}))

    msg = "Error calling `format(value, '^5d')`: ValueError: Unknown format code 'd' for object of type 'str'"
    with pytest.raises(PydanticSerializationError, match=re.escape(msg)):
        s.to_python('x')

    with pytest.raises(PydanticSerializationError, match=re.escape(msg)):
        s.to_json('x')
