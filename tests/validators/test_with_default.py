import pytest

from pydantic_core import SchemaError, SchemaValidator


def test_typed_dict_default():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {'schema': 'str'},
                'y': {'schema': {'type': 'default', 'schema': 'str', 'default': '[default]'}},
            },
        }
    )
    assert v.validate_python({'x': 'x', 'y': 'y'}) == {'x': 'x', 'y': 'y'}
    assert v.validate_python({'x': 'x'}) == {'x': 'x', 'y': '[default]'}


def test_typed_dict_omit():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {'schema': 'str'},
                'y': {'schema': {'type': 'default', 'schema': 'str', 'on_error': 'omit'}, 'required': False},
            },
        }
    )
    assert v.validate_python({'x': 'x', 'y': 'y'}) == {'x': 'x', 'y': 'y'}
    assert v.validate_python({'x': 'x'}) == {'x': 'x'}
    assert v.validate_python({'x': 'x', 'y': 42}) == {'x': 'x'}


def test_arguments():
    v = SchemaValidator(
        {
            'type': 'arguments',
            'arguments_schema': [
                {
                    'name': 'a',
                    'mode': 'positional_or_keyword',
                    'schema': {'type': 'default', 'schema': 'int', 'default_factory': lambda: 1},
                }
            ],
        }
    )
    assert v.validate_python(((), {'a': 2})) == ((), {'a': 2})
    assert v.validate_python(((2,), {})) == ((2,), {})
    assert v.validate_python(((), {})) == ((), {'a': 1})


def test_arguments_omit():
    with pytest.raises(SchemaError, match="Parameter 'a': omit_on_error cannot be used with arguments"):
        SchemaValidator(
            {
                'type': 'arguments',
                'arguments_schema': [
                    {
                        'name': 'a',
                        'mode': 'positional_or_keyword',
                        'schema': {'type': 'default', 'schema': 'int', 'default': 1, 'on_error': 'omit'},
                    }
                ],
            }
        )


def test_list():
    v = SchemaValidator({'type': 'list', 'items_schema': {'type': 'default', 'schema': 'int', 'on_error': 'omit'}})
    assert v.validate_python([1, 2, 3]) == [1, 2, 3]
    assert v.validate_python([1, '2', 3]) == [1, 2, 3]
    assert v.validate_python([1, 'wrong', 3]) == [1, 3]


def test_set():
    v = SchemaValidator({'type': 'set', 'items_schema': {'type': 'default', 'schema': 'int', 'on_error': 'omit'}})
    assert v.validate_python({1, 2, 3}) == {1, 2, 3}
    assert v.validate_python([1, '2', 3]) == {1, 2, 3}
    assert v.validate_python([1, 'wrong', 3]) == {1, 3}


def test_tuple_variable():
    v = SchemaValidator({'type': 'tuple', 'items_schema': {'type': 'default', 'schema': 'int', 'on_error': 'omit'}})
    assert v.validate_python((1, 2, 3)) == (1, 2, 3)
    assert v.validate_python([1, '2', 3]) == (1, 2, 3)
    assert v.validate_python([1, 'wrong', 3]) == (1, 3)


@pytest.mark.xfail(reason='TODO')
def test_tuple_positional():
    v = SchemaValidator(
        {
            'type': 'tuple',
            'mode': 'positional',
            'items_schema': [{'type': 'int'}, {'type': 'default', 'schema': 'int', 'default': 42}],
        }
    )
    assert v.validate_python((1, '2')) == (1, 2)
    assert v.validate_python((1,)) == (1, 42)


def test_on_error_default():
    v = SchemaValidator({'type': 'default', 'schema': 'int', 'default': 2, 'on_error': 'default'})
    assert v.validate_python(42) == 42
    assert v.validate_python('42') == 42
    assert v.validate_python('wrong') == 2


def test_on_error_default_not_int():
    v = SchemaValidator({'type': 'default', 'schema': 'int', 'default': [1, 2, 3], 'on_error': 'default'})
    assert v.validate_python(42) == 42
    assert v.validate_python('42') == 42
    a = v.validate_python('wrong')
    assert a == [1, 2, 3]
    # default is not copied, so mutating it mutates the default
    a.append(4)
    assert v.validate_python('wrong') == [1, 2, 3, 4]


def test_on_error_default_factory():
    v = SchemaValidator({'type': 'default', 'schema': 'int', 'default_factory': lambda: 17, 'on_error': 'default'})
    assert v.validate_python(42) == 42
    assert v.validate_python('42') == 42
    assert v.validate_python('wrong') == 17


def test_on_error_omit():
    v = SchemaValidator({'type': 'default', 'schema': 'int', 'on_error': 'omit'})
    assert v.validate_python(42) == 42
    with pytest.raises(ValueError, match='Uncaught Omit error, please check your usage of "default" validators.'):
        v.validate_python('wrong')


def test_on_error_wrong():
    with pytest.raises(SchemaError, match="'on_error = default' requires a `default` or `default_factory`"):
        SchemaValidator({'type': 'default', 'schema': 'int', 'on_error': 'default'})


def test_build_default_and_default_factory():
    with pytest.raises(SchemaError, match="'default' and 'default_factory' cannot be used together"):
        SchemaValidator({'type': 'default', 'schema': 'int', 'default_factory': lambda: 1, 'default': 2})
