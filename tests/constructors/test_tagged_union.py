from enum import Enum

import pytest

from pydantic_core import SchemaValidator, ValidationError, core_schema

from ..conftest import Err, PyAndJson
from .test_typed_dict import Cls


@pytest.mark.parametrize('recursive', [True, False])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': 'apple', 'bar': '123'}, {'foo': 'apple', 'bar': '123'}),
        ({'foo': 'banana', 'spam': [1, 2, '3']}, {'foo': 'banana', 'spam': [1, 2, '3']}),
        ({'foo': 'apple', 'bar': 'wrong'}, {'foo': 'apple', 'bar': 'wrong'}),
        ({'foo': 'banana'}, {'foo': 'banana'}),
        ({'foo': 'other'}, {'foo': 'other'}),
        ({}, {}),
        ('not a dict', 'not a dict'),
    ],
    ids=repr,
)
def test_simple_tagged_union(py_and_json: PyAndJson, input_value, expected, recursive):
    v = py_and_json(
        {
            'type': 'tagged-union',
            'discriminator': 'foo',
            'from_attributes': False,
            'choices': {
                'apple': {
                    'type': 'typed-dict',
                    'fields': {
                        'foo': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                        'bar': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                    },
                },
                'banana': {
                    'type': 'typed-dict',
                    'fields': {
                        'foo': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                        'spam': {
                            'type': 'typed-dict-field',
                            'schema': {'type': 'list', 'items_schema': {'type': 'int'}},
                        },
                    },
                },
            },
        }
    )
    assert 'discriminator: LookupKey' in repr(v.validator)
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message) as exc_info:
            v.construct_test(input_value, recursive=recursive)
        # debug(exc_info.value.errors(include_url=False))
        assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        assert v.construct_test(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [True, False])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'foo': 123, 'bar': '123'}, {'foo': 123, 'bar': '123'}),
        ({'foo': 'banana', 'spam': [1, 2, '3']}, {'foo': 'banana', 'spam': [1, 2, '3']}),
        ({'foo': 123, 'bar': 'wrong'}, {'foo': 123, 'bar': 'wrong'}),
        ({'foo': 1234567, 'bar': '123'}, {'foo': 1234567, 'bar': '123'}),
    ],
)
def test_int_choice_keys(py_and_json: PyAndJson, input_value, expected, recursive):
    v = py_and_json(
        {
            'type': 'tagged-union',
            'discriminator': 'foo',
            'choices': {
                123: {
                    'type': 'typed-dict',
                    'fields': {
                        'foo': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                        'bar': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                    },
                },
                'banana': {
                    'type': 'typed-dict',
                    'fields': {
                        'foo': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                        'spam': {
                            'type': 'typed-dict-field',
                            'schema': {'type': 'list', 'items_schema': {'type': 'int'}},
                        },
                    },
                },
            },
        }
    )
    assert 'discriminator: LookupKey' in repr(v.validator)
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message) as exc_info:
            v.construct_test(input_value, recursive=recursive)
        # debug(exc_info.value.errors(include_url=False))
        assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        assert v.construct_test(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [True, False])
def test_enum_keys(recursive):
    class FooEnum(str, Enum):
        APPLE = 'apple'
        BANANA = 'banana'

    class BarEnum(int, Enum):
        ONE = 1

    class PlainEnum(Enum):
        TWO = 'two'

    v = SchemaValidator(
        {
            'type': 'tagged-union',
            'discriminator': 'foo',
            'choices': {
                BarEnum.ONE: {
                    'type': 'typed-dict',
                    'fields': {
                        'foo': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                        'bar': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                    },
                },
                FooEnum.BANANA: {
                    'type': 'typed-dict',
                    'fields': {
                        'foo': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                        'spam': {
                            'type': 'typed-dict-field',
                            'schema': {'type': 'list', 'items_schema': {'type': 'int'}},
                        },
                    },
                },
                PlainEnum.TWO: {
                    'type': 'typed-dict',
                    'fields': {
                        'foo': {'type': 'typed-dict-field', 'schema': {'type': 'any'}},
                        'baz': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                    },
                },
            },
        }
    )

    assert v.construct_python({'foo': FooEnum.BANANA, 'spam': [1, 2, '3']}, recursive=recursive) == {
        'foo': FooEnum.BANANA,
        'spam': [1, 2, '3'],
    }
    assert v.construct_python({'foo': BarEnum.ONE, 'bar': '123'}, recursive=recursive) == {
        'foo': BarEnum.ONE,
        'bar': '123',
    }
    assert v.construct_python({'foo': PlainEnum.TWO, 'baz': '123'}, recursive=recursive) == {
        'foo': PlainEnum.TWO,
        'baz': '123',
    }
    assert v.construct_python({'foo': FooEnum.APPLE, 'spam': [1, 2, '3']}, recursive=recursive) == {
        'foo': FooEnum.APPLE,
        'spam': [1, 2, '3'],
    }


@pytest.mark.parametrize('recursive', [True, False])
def test_discriminator_path(py_and_json: PyAndJson, recursive):
    v = py_and_json(
        {
            'type': 'tagged-union',
            'discriminator': [['food'], ['menu', 1]],
            'choices': {
                'apple': {
                    'type': 'typed-dict',
                    'fields': {
                        'a': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                        'b': {'type': 'typed-dict-field', 'schema': {'type': 'int'}},
                    },
                },
                'banana': {
                    'type': 'typed-dict',
                    'fields': {
                        'c': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                        'd': {'type': 'typed-dict-field', 'schema': {'type': 'list', 'items_schema': {'type': 'int'}}},
                    },
                },
            },
        }
    )
    assert v.construct_test({'food': 'apple', 'a': 'apple', 'b': '13'}, recursive=recursive) == {
        'food': 'apple',
        'a': 'apple',
        'b': '13',
    }
    assert v.construct_test({'menu': ['x', 'banana'], 'c': 'C', 'd': [1, '2']}, recursive=recursive) == {
        'menu': ['x', 'banana'],
        'c': 'C',
        'd': [1, '2'],
    }
    assert v.construct_test({}, recursive=recursive) == {}


@pytest.mark.parametrize('recursive', [True, False])
@pytest.mark.parametrize(
    'input_value,expected', [('foo', 'foo'), (123, 123), ('baz', 'baz'), (None, None), (['wrong type'], ['wrong type'])]
)
def test_discriminator_function(py_and_json: PyAndJson, input_value, expected, recursive):
    def discriminator_function(obj):
        if isinstance(obj, str):
            return 'str'
        elif isinstance(obj, int):
            return 'int'
        elif obj is None:
            return None
        else:
            return 'other'

    v = py_and_json(
        {
            'type': 'tagged-union',
            'discriminator': discriminator_function,
            'choices': {'str': {'type': 'literal', 'expected': ['foo', 'bar']}, 'int': {'type': 'int'}},
        }
    )
    assert 'discriminator: Function' in repr(v.validator)
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message) as exc_info:
            v.construct_test(input_value, recursive=recursive)
        # debug(exc_info.value.errors(include_url=False))
        assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        assert v.construct_test(input_value, recursive=recursive) == expected


def test_from_attributes():
    v = SchemaValidator(
        {
            'type': 'tagged-union',
            'discriminator': 'foobar',
            'choices': {
                'apple': {
                    'type': 'model-fields',
                    'fields': {
                        'a': {'type': 'model-field', 'schema': {'type': 'str'}},
                        'b': {'type': 'model-field', 'schema': {'type': 'int'}},
                    },
                },
                'banana': {
                    'type': 'model-fields',
                    'fields': {
                        'c': {'type': 'model-field', 'schema': {'type': 'str'}},
                        'd': {'type': 'model-field', 'schema': {'type': 'int'}},
                    },
                },
            },
        },
        {'from_attributes': True},
    )
    assert v.construct_python({'foobar': 'apple', 'a': 'apple', 'b': '13'}, recursive=True) == (
        {'a': 'apple', 'b': '13'},
        None,
        {'a', 'b'},
    )
    assert v.construct_python(Cls(foobar='apple', a='apple', b='13'), recursive=True) == (
        {'a': 'apple', 'b': '13'},
        None,
        {'a', 'b'},
    )
    assert v.construct_python({'foobar': 'banana', 'c': 'banana', 'd': '31'}, recursive=True) == (
        {'c': 'banana', 'd': '31'},
        None,
        {'c', 'd'},
    )
    assert v.construct_python(Cls(foobar='banana', c='banana', d='31'), recursive=True) == (
        {'c': 'banana', 'd': '31'},
        None,
        {'c', 'd'},
    )


def test_discriminator_recursive():
    class Container:
        name: str
        position: dict

    class Inserter:
        name: str
        position: dict
        direction: int

    c = core_schema.model_schema(
        Container,
        core_schema.model_fields_schema(
            fields={
                'name': core_schema.model_field(core_schema.str_schema()),
                'position': core_schema.model_field(core_schema.dict_schema()),
            }
        ),
    )

    i = core_schema.model_schema(
        Inserter,
        core_schema.model_fields_schema(
            fields={
                'name': core_schema.model_field(core_schema.str_schema()),
                'position': core_schema.model_field(core_schema.dict_schema()),
                'direction': core_schema.model_field(core_schema.int_schema()),
            }
        ),
    )

    v = SchemaValidator(
        core_schema.tagged_union_schema(choices={'wooden-chest': c, 'fast-inserter': i}, discriminator='name')
    )
    # Incorrect should pass
    assert v.construct_python('wrong') == 'wrong'

    # Correct type 1
    assert v.construct_python({'name': 'wooden-chest', 'position': {'x': 1, 'y': 1}}) == {
        'name': 'wooden-chest',
        'position': {'x': 1, 'y': 1},
    }
    m = v.construct_python({'name': 'wooden-chest', 'position': {'x': 1, 'y': 1}}, recursive=True)
    assert isinstance(m, Container)
    assert m.name == 'wooden-chest'
    assert m.position == {'x': 1, 'y': 1}

    # Correct type 2
    assert v.construct_python({'name': 'fast-inserter', 'position': {'x': 1, 'y': 1}, 'direction': 1}) == {
        'name': 'fast-inserter',
        'position': {'x': 1, 'y': 1},
        'direction': 1,
    }
    m = v.construct_python({'name': 'fast-inserter', 'position': {'x': 1, 'y': 1}, 'direction': 1}, recursive=True)
    assert isinstance(m, Inserter)
    assert m.name == 'fast-inserter'
    assert m.position == {'x': 1, 'y': 1}
    assert m.direction == 1

    # Correct structure, but fields wrong
    assert v.construct_python({'name': 'wooden-chest', 'position': 'wrong'}) == {
        'name': 'wooden-chest',
        'position': 'wrong',
    }
    m = v.construct_python({'name': 'wooden-chest', 'position': 'wrong'}, recursive=True)
    assert isinstance(m, Container)
    assert m.name == 'wooden-chest'
    assert m.position == 'wrong'

    # Unrecognized literal; return as-is, even if other fields match
    assert v.construct_python({'name': 'who knows?', 'position': {'x': 1, 'y': 1}}) == {
        'name': 'who knows?',
        'position': {'x': 1, 'y': 1},
    }
    assert v.construct_python({'name': 'who knows?', 'position': {'x': 1, 'y': 1}}, recursive=True) == {
        'name': 'who knows?',
        'position': {'x': 1, 'y': 1},
    }
