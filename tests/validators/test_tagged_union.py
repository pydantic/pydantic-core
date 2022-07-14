import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err, PyAndJson


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': 'apple', 'bar': '123'}, {'foo': 'apple', 'bar': 123}),
        ({'foo': 'banana', 'spam': [1, 2, '3']}, {'foo': 'banana', 'spam': [1, 2, 3]}),
        (
            {'foo': 'apple', 'bar': 'wrong'},
            Err(
                'Value must be a valid integer',
                [
                    {
                        'kind': 'int_parsing',
                        'loc': ['apple', 'bar'],
                        'message': 'Value must be a valid integer, unable to parse string as an integer',
                        'input_value': 'wrong',
                    }
                ],
            ),
        ),
        (
            {'foo': 'banana'},
            Err(
                'Field required',
                [
                    {
                        'kind': 'missing',
                        'loc': ['banana', 'spam'],
                        'message': 'Field required',
                        'input_value': {'foo': 'banana'},
                    }
                ],
            ),
        ),
        (
            {'foo': 'other'},
            Err(
                'union_tag_not_found',
                [
                    {
                        'kind': 'union_tag_not_found',
                        'loc': [],
                        'message': 'Input key "foo" must match one of the allowed tags "apple", "banana"',
                        'input_value': {'foo': 'other'},
                        'context': {'key': 'foo', 'tags': '"apple", "banana"'},
                    }
                ],
            ),
        ),
        (
            'not a dict',
            Err(
                'dict_type',
                [
                    {
                        'kind': 'dict_type',
                        'loc': [],
                        'message': 'Value must be a valid dictionary',
                        'input_value': 'not a dict',
                    }
                ],
            ),
        ),
    ],
    ids=repr,
)
def test_simple_tagged_union(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        {
            'type': 'tagged-union',
            'tag_key': 'foo',
            'choices': {
                'apple': {'type': 'typed-dict', 'fields': {'foo': {'schema': 'str'}, 'bar': {'schema': 'int'}}},
                'banana': {
                    'type': 'typed-dict',
                    'fields': {'foo': {'schema': 'str'}, 'spam': {'schema': {'type': 'list', 'items_schema': 'int'}}},
                },
            },
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message) as exc_info:
            v.validate_python(input_value)
        # debug(exc_info.value.errors())
        assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected


def test_tag_keys():
    v = SchemaValidator(
        {
            'type': 'tagged-union',
            'tag_keys': [['food'], ['menu', 1]],
            'choices': {
                'apple': {'type': 'typed-dict', 'fields': {'a': {'schema': 'str'}, 'b': {'schema': 'int'}}},
                'banana': {
                    'type': 'typed-dict',
                    'fields': {'c': {'schema': 'str'}, 'd': {'schema': {'type': 'list', 'items_schema': 'int'}}},
                },
            },
        }
    )
    assert v.validate_python({'food': 'apple', 'a': 'apple', 'b': '13'}) == {'a': 'apple', 'b': 13}
    assert v.validate_python({'menu': ['x', 'banana'], 'c': 'C', 'd': [1, '2']}) == {'c': 'C', 'd': [1, 2]}
