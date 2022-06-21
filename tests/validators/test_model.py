import pytest

from pydantic_core import SchemaError, SchemaValidator, ValidationError

from ..conftest import Err


def test_simple():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}}}
    )

    assert v.validate_python({'field_a': 123, 'field_b': 1}) == (
        {'field_a': '123', 'field_b': 1},
        {'field_b', 'field_a'},
    )


def test_strict():
    v = SchemaValidator(
        {
            'type': 'model',
            'config': {'strict': True},
            'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}},
        }
    )

    assert v.validate_python({'field_a': 'hello', 'field_b': 12}) == (
        {'field_a': 'hello', 'field_b': 12},
        {'field_b', 'field_a'},
    )
    with pytest.raises(ValidationError) as exc_info:
        assert v.validate_python({'field_a': 123, 'field_b': '123'})
    assert exc_info.value.errors() == [
        {'kind': 'str_type', 'loc': ['field_a'], 'message': 'Value must be a valid string', 'input_value': 123},
        {'kind': 'int_type', 'loc': ['field_b'], 'message': 'Value must be a valid integer', 'input_value': '123'},
    ]


def test_with_default():
    v = SchemaValidator(
        {
            'type': 'model',
            'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}, 'default': 666}},
        }
    )

    assert v.validate_python({'field_a': 123}) == ({'field_a': '123', 'field_b': 666}, {'field_a'})
    assert v.validate_python({'field_a': 123, 'field_b': 1}) == (
        {'field_a': '123', 'field_b': 1},
        {'field_b', 'field_a'},
    )


def test_missing_error():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}}}
    )
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python({'field_a': 123})
    assert (
        str(exc_info.value)
        == """\
1 validation error for Model
field_b
  Field required [kind=missing, input_value={'field_a': 123}, input_type=dict]"""
    )


def test_ignore_extra():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}, 'field_b': {'schema': {'type': 'int'}}}}
    )

    assert v.validate_python({'field_a': 123, 'field_b': 1, 'field_c': 123}) == (
        {'field_a': '123', 'field_b': 1},
        {'field_b', 'field_a'},
    )


def test_forbid_extra():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}}, 'config': {'extra': 'forbid'}}
    )

    with pytest.raises(ValidationError, match='field_b | Extra values are not permitted'):
        v.validate_python({'field_a': 123, 'field_b': 1})


def test_allow_extra():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}}, 'config': {'extra': 'allow'}}
    )

    assert v.validate_python({'field_a': 123, 'field_b': (1, 2)}) == (
        {'field_a': '123', 'field_b': (1, 2)},
        {'field_a', 'field_b'},
    )


def test_allow_extra_validate():
    v = SchemaValidator(
        {
            'type': 'model',
            'fields': {'field_a': {'schema': {'type': 'str'}}},
            'extra_validator': {'type': 'int'},
            'config': {'extra': 'allow'},
        }
    )

    assert v.validate_python({'field_a': 'test', 'other_value': '123'}) == (
        {'field_a': 'test', 'other_value': 123},
        {'field_a', 'other_value'},
    )

    with pytest.raises(ValidationError) as exc_info:
        v.validate_python({'field_a': 'test', 'other_value': 12.5})
    assert exc_info.value.errors() == [
        {
            'kind': 'int_from_float',
            'loc': ['other_value'],
            'message': 'Value must be a valid integer, got a number with a fractional part',
            'input_value': 12.5,
        }
    ]


def test_str_config():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}}, 'config': {'str_max_length': 5}}
    )
    assert v.validate_python({'field_a': 'test'}) == ({'field_a': 'test'}, {'field_a'})

    with pytest.raises(ValidationError, match='String must have at most 5 characters'):
        v.validate_python({'field_a': 'test long'})


def test_validate_assignment():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}}})

    assert v.validate_python({'field_a': 'test'}) == ({'field_a': 'test'}, {'field_a'})

    assert v.validate_assignment('field_a', 456, {'field_a': 'test'}) == ({'field_a': '456'}, {'field_a'})


def test_validate_assignment_functions():
    calls = []

    def func_a(input_value, **kwargs):
        calls.append('func_a')
        return input_value * 2

    def func_b(input_value, **kwargs):
        calls.append('func_b')
        return input_value / 2

    v = SchemaValidator(
        {
            'type': 'model',
            'fields': {
                'field_a': {
                    'schema': {'type': 'function', 'mode': 'after', 'function': func_a, 'schema': {'type': 'str'}}
                },
                'field_b': {
                    'schema': {'type': 'function', 'mode': 'after', 'function': func_b, 'schema': {'type': 'int'}}
                },
            },
        }
    )

    assert v.validate_python({'field_a': 'test', 'field_b': 12.0}) == (
        {'field_a': 'testtest', 'field_b': 6},
        {'field_a', 'field_b'},
    )

    assert calls == ['func_a', 'func_b']
    calls.clear()

    assert v.validate_assignment('field_a', 'new-val', {'field_a': 'testtest', 'field_b': 6}) == (
        {'field_a': 'new-valnew-val', 'field_b': 6},
        {'field_a'},
    )
    assert calls == ['func_a']


def test_validate_assignment_ignore_extra():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}}})

    assert v.validate_python({'field_a': 'test'}) == ({'field_a': 'test'}, {'field_a'})

    with pytest.raises(ValidationError) as exc_info:
        v.validate_assignment('other_field', 456, {'field_a': 'test'})

    assert exc_info.value.errors() == [
        {
            'kind': 'extra_forbidden',
            'loc': ['other_field'],
            'message': 'Extra values are not permitted',
            'input_value': 456,
        }
    ]


def test_validate_assignment_allow_extra():
    v = SchemaValidator(
        {'type': 'model', 'fields': {'field_a': {'schema': {'type': 'str'}}}, 'config': {'extra': 'allow'}}
    )

    assert v.validate_python({'field_a': 'test'}) == ({'field_a': 'test'}, {'field_a'})

    assert v.validate_assignment('other_field', 456, {'field_a': 'test'}) == (
        {'field_a': 'test', 'other_field': 456},
        {'other_field'},
    )


def test_validate_assignment_allow_extra_validate():
    v = SchemaValidator(
        {
            'type': 'model',
            'fields': {'field_a': {'schema': {'type': 'str'}}},
            'extra_validator': {'type': 'int'},
            'config': {'extra': 'allow'},
        }
    )

    assert v.validate_assignment('other_field', '456', {'field_a': 'test'}) == (
        {'field_a': 'test', 'other_field': 456},
        {'other_field'},
    )
    with pytest.raises(ValidationError) as exc_info:
        assert v.validate_assignment('other_field', 'xyz', {'field_a': 'test'})
    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': ['other_field'],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'xyz',
        }
    ]


def test_json_error():
    v = SchemaValidator({'type': 'model', 'fields': {'field_a': {'schema': {'type': 'list', 'items': 'int'}}}})
    with pytest.raises(ValidationError) as exc_info:
        v.validate_json('{"field_a": [123, "wrong"]}')

    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': ['field_a', 1],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'wrong',
        }
    ]


def test_missing_schema_key():
    with pytest.raises(SchemaError, match='"schema" is required'):
        SchemaValidator({'type': 'model', 'fields': {'x': {'type': 'str'}}})


def test_alias(py_or_json):
    v = py_or_json({'type': 'model', 'fields': {'field_a': {'alias': 'FieldA', 'schema': 'int'}}})
    assert v.validate_test({'FieldA': '123'}) == ({'field_a': 123}, {'field_a'})
    with pytest.raises(ValidationError, match=r'field_a\n +Field required \[kind=missing,'):
        assert v.validate_test({'foobar': '123'})
    with pytest.raises(ValidationError, match=r'field_a\n +Field required \[kind=missing,'):
        assert v.validate_test({'field_a': '123'})


def test_alias_allow_pop(py_or_json):
    v = py_or_json(
        {
            'type': 'model',
            'config': {'allow_population_by_field_name': True},
            'fields': {'field_a': {'alias': 'FieldA', 'schema': 'int'}},
        }
    )
    assert v.validate_test({'FieldA': '123'}) == ({'field_a': 123}, {'field_a'})
    assert v.validate_test({'field_a': '123'}) == ({'field_a': 123}, {'field_a'})
    with pytest.raises(ValidationError, match=r'field_a\n +Field required \[kind=missing,'):
        assert v.validate_test({'foobar': '123'})


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': {'bar': '123'}}, ({'field_a': 123}, {'field_a'})),
        ({'x': '123'}, Err(r'field_a\n +Field required \[kind=missing,')),
        ({'foo': '123'}, Err(r'field_a\n +Field required \[kind=missing,')),
        ({'foo': [1, 2, 3]}, Err(r'field_a\n +Field required \[kind=missing,')),
        ({'foo': {'bat': '123'}}, Err(r'field_a\n +Field required \[kind=missing,')),
    ],
    ids=repr,
)
def test_alias_path(py_or_json, input_value, expected):
    v = py_or_json({'type': 'model', 'fields': {'field_a': {'aliases': [['foo', 'bar']], 'schema': 'int'}}})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message):
            v.validate_test(input_value)
    else:
        output = v.validate_test(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': {'bar': {'bat': '123'}}}, ({'field_a': 123}, {'field_a'})),
        ({'foo': [1, 2, 3, 4]}, ({'field_a': 4}, {'field_a'})),
        ({'foo': (1, 2, 3, 4)}, ({'field_a': 4}, {'field_a'})),
        ({'spam': 5}, ({'field_a': 5}, {'field_a'})),
        ({'spam': 1, 'foo': {'bar': {'bat': 2}}}, ({'field_a': 2}, {'field_a'})),
        ({'x': '123'}, Err(r'field_a\n +Field required \[kind=missing,')),
        ({'foo': '01234'}, Err(r'field_a\n +Field required \[kind=missing,')),
        ({'foo': [1]}, Err(r'field_a\n +Field required \[kind=missing,')),
    ],
    ids=repr,
)
def test_alias_path_multiple(py_or_json, input_value, expected):
    v = py_or_json(
        {
            'type': 'model',
            'fields': {'field_a': {'aliases': [['foo', 'bar', 'bat'], ['foo', 3], ['spam']], 'schema': 'int'}},
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message):
            val = v.validate_test(input_value)
            print(f'UNEXPECTED OUTPUT: {val!r}')
    else:
        output = v.validate_test(input_value)
        assert output == expected


@pytest.mark.parametrize(
    'alias_schema,error',
    [
        ({'alias': ['foo']}, "TypeError: 'list' object cannot be converted to 'PyString'"),
        ({'alias': 'foo', 'aliases': []}, "'alias' and 'aliases' cannot be used together"),
        ({'aliases': []}, 'Aliases must have at least one element'),
        ({'aliases': [[]]}, 'Each alias path must have at least one element'),
        ({'aliases': [123]}, "TypeError: 'int' object cannot be converted to 'PyList'"),
        ({'aliases': [[[]]]}, 'TypeError: Alias path items must be with a string or int'),
        ({'aliases': [[1, 'foo']]}, 'TypeError: The first item in an alias path must be a string'),
    ],
    ids=repr,
)
def test_alias_build_error(alias_schema, error):
    with pytest.raises(SchemaError, match=error):
        SchemaValidator({'type': 'model', 'fields': {'field_a': {'schema': 'int', **alias_schema}}})


def test_empty_model():
    v = SchemaValidator({'type': 'model', 'fields': {}})
    assert v.validate_python({}) == ({}, set())
    with pytest.raises(ValidationError):
        v.validate_python('x')
