import pytest

from pydantic_core import SchemaValidator


@pytest.mark.parametrize(
    'input_value,output_value',
    [('false', False), ('true', True), ('0', False), ('1', True), ('"yes"', True), ('"no"', False)],
)
def test_bool(input_value, output_value):
    v = SchemaValidator({'type': 'bool', 'title': 'TestModel'})
    assert v.validate_json(input_value) == output_value


def test_model():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'type': 'str'}, 'field_b': {'type': 'int'}}})

    # language=json
    input_str = '{"field_a": 123, "field_b": 1}'
    assert v.validate_json(input_str) == ({'field_a': '123', 'field_b': 1}, {'field_b', 'field_a'})


def test_list():
    v = SchemaValidator({'type': 'list', 'items': {'type': 'int'}, 'title': 'TestModel'})
    # language=json
    input_str = '[1, 2, "3"]'
    assert v.validate_json(input_str) == [1, 2, 3]


def test_float_no_remainder():
    v = SchemaValidator({'type': 'int'})
    assert v.validate_json('123.0') == 123
