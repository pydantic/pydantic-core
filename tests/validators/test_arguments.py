import pytest

from pydantic_core import ValidationError

from ..conftest import PyAndJson


def test_positional_args(py_and_json: PyAndJson):
    v = py_and_json(
        {
            'type': 'arguments',
            'positional_args_schema': {'type': 'tuple', 'mode': 'positional', 'items_schema': ['int', 'str', 'bool']},
        }
    )
    assert v.validate_test(((1, 'a', True), None)) == ((1, 'a', True), {})
    assert v.validate_test(((1, 'a', True), {})) == ((1, 'a', True), {})
    assert v.validate_test(([1, 'a', True], None)) == ((1, 'a', True), {})
    assert v.validate_test(((1, 'a', 'true'), None)) == ((1, 'a', True), {})
    with pytest.raises(ValidationError, match='kind=arguments_type,'):
        v.validate_test('x')
    with pytest.raises(ValidationError, match='kind=arguments_type,'):
        # lists are not allowed from python, but no equivalent restriction in JSON
        v.validate_python([(1, 'a', True), None])
    with pytest.raises(ValidationError, match='kind=arguments_type,'):
        v.validate_test(((1, 'a', True), ()))
    with pytest.raises(ValidationError, match='kind=unexpected_keyword_arguments,'):
        v.validate_test(((1, 'a', True), {'x': 1}))
    with pytest.raises(ValidationError, match='kind=tuple_length_mismatch,'):
        v.validate_test(((1, 'a', True, 4), None))
    with pytest.raises(ValidationError) as exc_info:
        v.validate_test((('x', 'a', 'wrong'), None))
    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': [0],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'x',
        },
        {
            'kind': 'bool_parsing',
            'loc': [2],
            'message': 'Value must be a valid boolean, unable to interpret input',
            'input_value': 'wrong',
        },
    ]


def test_keyword_args(py_and_json: PyAndJson):
    v = py_and_json(
        {
            'type': 'arguments',
            'keyword_args_schema': {
                'type': 'typed-dict',
                'extra_behavior': 'forbid',
                'fields': {'a': {'schema': 'int'}, 'b': {'schema': 'str'}, 'c': {'schema': 'bool'}},
            },
        }
    )
    assert v.validate_test((None, {'a': 1, 'b': 'a', 'c': True})) == ((), {'a': 1, 'b': 'a', 'c': True})
    assert v.validate_test({'a': 1, 'b': 'a', 'c': True}) == ((), {'a': 1, 'b': 'a', 'c': True})
    assert v.validate_test((None, {'a': '1', 'b': 'a', 'c': 'True'})) == ((), {'a': 1, 'b': 'a', 'c': True})
    assert v.validate_test(((), {'a': 1, 'b': 'a', 'c': True})) == ((), {'a': 1, 'b': 'a', 'c': True})
    with pytest.raises(ValidationError, match='kind=unexpected_positional_arguments,'):
        v.validate_test(((1,), {'a': 1, 'b': 'a', 'c': True}))
    with pytest.raises(ValidationError, match='kind=extra_forbidden,'):
        v.validate_test(((), {'a': 1, 'b': 'a', 'c': True, 'd': 'wrong'}))
    with pytest.raises(ValidationError) as exc_info:
        v.validate_test(((), {'a': 'x', 'b': 'a', 'c': 'wrong'}))
    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': ['a'],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'x',
        },
        {
            'kind': 'bool_parsing',
            'loc': ['c'],
            'message': 'Value must be a valid boolean, unable to interpret input',
            'input_value': 'wrong',
        },
    ]
