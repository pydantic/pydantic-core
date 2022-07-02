from typing import Optional

import pytest
from dirty_equals import AnyThing, HasAttributes, IsList, IsPartialDict

from pydantic_core import SchemaError, SchemaValidator, ValidationError

from ..conftest import Err
from .test_typed_dict import Cls


def test_branch_nullable():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'ref': 'Branch',
            'fields': {
                'name': {'schema': {'type': 'str'}},
                'sub_branch': {
                    'schema': {
                        'type': 'union',
                        'choices': [{'type': 'none'}, {'type': 'recursive-ref', 'schema_ref': 'Branch'}],
                    },
                    'default': None,
                },
            },
        }
    )

    assert v.validate_python({'name': 'root'}) == {'name': 'root', 'sub_branch': None}

    assert v.validate_python({'name': 'root', 'sub_branch': {'name': 'b1'}}) == (
        {'name': 'root', 'sub_branch': {'name': 'b1', 'sub_branch': None}}
    )
    assert v.validate_python({'name': 'root', 'sub_branch': {'name': 'b1', 'sub_branch': {'name': 'b2'}}}) == (
        {'name': 'root', 'sub_branch': {'name': 'b1', 'sub_branch': {'name': 'b2', 'sub_branch': None}}}
    )


def test_nullable_error():
    v = SchemaValidator(
        {
            'ref': 'Branch',
            'type': 'typed-dict',
            'fields': {
                'width': {'schema': 'int'},
                'sub_branch': {
                    'schema': {
                        'type': 'union',
                        'choices': [{'type': 'none'}, {'type': 'recursive-ref', 'schema_ref': 'Branch'}],
                    },
                    'default': None,
                },
            },
        }
    )
    assert v.validate_python({'width': 123, 'sub_branch': {'width': 321}}) == (
        {'width': 123, 'sub_branch': {'width': 321, 'sub_branch': None}}
    )
    with pytest.raises(ValidationError) as exc_info:
        assert v.validate_python({'width': 123, 'sub_branch': {'width': 'wrong'}})
    assert exc_info.value.errors() == [
        {
            'kind': 'none_required',
            'loc': ['sub_branch', 'none'],
            'message': 'Value must be None/null',
            'input_value': {'width': 'wrong'},
        },
        {
            'kind': 'int_parsing',
            'loc': ['sub_branch', 'recursive-ref', 'width'],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'wrong',
        },
    ]


def test_list():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'ref': 'BranchList',
            'fields': {
                'width': {'schema': 'int'},
                'branches': {
                    'schema': {'type': 'list', 'items_schema': {'type': 'recursive-ref', 'schema_ref': 'BranchList'}},
                    'default': None,
                },
            },
        }
    )
    assert v.validate_python({'width': 1, 'branches': [{'width': 2}, {'width': 3, 'branches': [{'width': 4}]}]}) == (
        {
            'width': 1,
            'branches': [{'width': 2, 'branches': None}, {'width': 3, 'branches': [{'width': 4, 'branches': None}]}],
        }
    )


def test_multiple_intertwined():
    """
    like:
    from typing import List, Optional
    class Foo:
        height: int
        class Bar:
            width: int
            bars: List['Bar']
            foo: Optional['Foo']
        bar = Bar
    """

    v = SchemaValidator(
        {
            'ref': 'Foo',
            'type': 'typed-dict',
            'fields': {
                'height': {'schema': 'int'},
                'bar': {
                    'schema': {
                        'ref': 'Bar',
                        'type': 'typed-dict',
                        'fields': {
                            'width': {'schema': 'int'},
                            'bars': {
                                'schema': {
                                    'type': 'list',
                                    'items_schema': {'type': 'recursive-ref', 'schema_ref': 'Bar'},
                                },
                                'default': None,
                            },
                            'foo': {
                                'schema': {
                                    'type': 'union',
                                    'choices': [{'type': 'none'}, {'type': 'recursive-ref', 'schema_ref': 'Foo'}],
                                },
                                'default': None,
                            },
                        },
                    }
                },
            },
        }
    )
    v.validate_python(
        {
            'height': 1,
            'bar': {
                'width': 2,
                'bars': [{'width': 3}],
                'foo': {'height': 4, 'bar': {'width': 5, 'bars': [], 'foo': None}},
            },
        }
    )


def test_model_class():
    class Branch:
        # this is not required, but it avoids `__fields_set__` being included in `__dict__`
        __slots__ = '__dict__', '__fields_set__'
        # these are here just as decoration
        width: int
        branch: Optional['Branch']  # noqa F821

    v = SchemaValidator(
        {
            'type': 'model-class',
            'ref': 'Branch',
            'class_type': Branch,
            'schema': {
                'type': 'typed-dict',
                'return_fields_set': True,
                'fields': {
                    'width': {'schema': 'int'},
                    'branch': {
                        'schema': {
                            'type': 'union',
                            'choices': [{'type': 'none'}, {'type': 'recursive-ref', 'schema_ref': 'Branch'}],
                        },
                        'default': None,
                    },
                },
            },
        }
    )
    m1: Branch = v.validate_python({'width': '1'})
    assert isinstance(m1, Branch)
    assert m1.__fields_set__ == {'width'}
    assert m1.__dict__ == {'width': 1, 'branch': None}
    assert m1.width == 1
    assert m1.branch is None

    m2: Branch = v.validate_python({'width': '10', 'branch': {'width': 20}})
    assert isinstance(m2, Branch)
    assert m2.__fields_set__ == {'width', 'branch'}
    assert m2.width == 10
    assert isinstance(m2.branch, Branch)
    assert m2.branch.width == 20
    assert m2.branch.branch is None


def test_invalid_schema():
    with pytest.raises(SchemaError, match="Recursive reference error: ref 'Branch' not found"):
        SchemaValidator(
            {
                'type': 'list',
                'items_schema': {
                    'type': 'typed-dict',
                    'fields': {
                        'width': {'schema': {'type': 'int'}},
                        'branch': {
                            'schema': {'type': 'nullable', 'schema': {'type': 'recursive-ref', 'schema_ref': 'Branch'}},
                            'default': None,
                        },
                    },
                },
            }
        )


def test_outside_parent():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'tuple1': {
                    'schema': {'type': 'tuple-fix-len', 'ref': 'tuple-iis', 'items_schema': ['int', 'int', 'str']}
                },
                'tuple2': {'schema': {'type': 'recursive-ref', 'schema_ref': 'tuple-iis'}},
            },
        }
    )

    assert v.validate_python({'tuple1': [1, '1', 'frog'], 'tuple2': [2, '2', 'toad']}) == {
        'tuple1': (1, 1, 'frog'),
        'tuple2': (2, 2, 'toad'),
    }


def test_recursion_branch():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'ref': 'Branch',
            'config': {'from_attributes': True},
            'fields': {
                'name': {'schema': {'type': 'str'}},
                'branch': {
                    'schema': {'type': 'nullable', 'schema': {'type': 'recursive-ref', 'schema_ref': 'Branch'}},
                    'default': None,
                },
            },
        }
    )
    assert v.validate_python({'name': 'root'}) == {'name': 'root', 'branch': None}
    assert v.validate_python({'name': 'root', 'branch': {'name': 'b1', 'branch': None}}) == {
        'name': 'root',
        'branch': {'name': 'b1', 'branch': None},
    }

    data = Cls(name='root')
    data.branch = Cls(name='b1', branch=None)
    assert v.validate_python(data) == {'name': 'root', 'branch': {'name': 'b1', 'branch': None}}

    b = {'name': 'recursive'}
    b['branch'] = b
    with pytest.raises(ValidationError) as exc_info:
        assert v.validate_python(b)
    assert exc_info.value.title == 'recursive-container'
    assert exc_info.value.errors() == [
        {
            'kind': 'recursion_loop',
            'loc': ['branch'],
            'message': 'Recursion error - cyclic reference detected',
            'input_value': {'name': 'recursive', 'branch': IsPartialDict(name='recursive')},
        }
    ]

    data = Cls(name='root')
    data.branch = data
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python(data)
    assert exc_info.value.errors() == [
        {
            'kind': 'recursion_loop',
            'loc': ['branch'],
            'message': 'Recursion error - cyclic reference detected',
            'input_value': HasAttributes(name='root', branch=AnyThing()),
        }
    ]


def test_recursive_list():
    v = SchemaValidator(
        {'type': 'list', 'ref': 'the-list', 'items_schema': {'type': 'recursive-ref', 'schema_ref': 'the-list'}}
    )
    assert v.validate_python([]) == []
    assert v.validate_python([[]]) == [[]]

    data = list()
    data.append(data)
    with pytest.raises(ValidationError, match='Recursion error - cyclic reference detected'):
        assert v.validate_python(data)


@pytest.fixture(scope='module')
def multiple_tuple_schema():
    return SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'f1': {
                    'schema': {
                        'type': 'tuple-fix-len',
                        'ref': 't',
                        'items_schema': [
                            {'type': 'int'},
                            {'type': 'nullable', 'schema': {'type': 'recursive-ref', 'schema_ref': 't'}},
                        ],
                    }
                },
                'f2': {
                    'schema': {'type': 'nullable', 'schema': {'type': 'recursive-ref', 'schema_ref': 't'}},
                    'default': None,
                },
            },
        }
    )


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'f1': [1, None]}, {'f1': (1, None), 'f2': None}),
        ({'f1': [1, None], 'f2': [2, None]}, {'f1': (1, None), 'f2': (2, None)}),
        (
            {'f1': [1, (3, None)], 'f2': [2, (4, (4, (5, None)))]},
            {'f1': (1, (3, None)), 'f2': (2, (4, (4, (5, None))))},
        ),
        ({'f1': [1, 2]}, Err(r'f1 -> 1\s+Value must be a valid tuple')),
        (
            {'f1': [1, (3, None)], 'f2': [2, (4, (4, (5, 6)))]},
            Err(r'f2 -> 1 -> 1 -> 1 -> 1\s+Value must be a valid tuple'),
        ),
    ],
)
def test_multiple_tuple_param(multiple_tuple_schema, input_value, expected):
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message):
            multiple_tuple_schema.validate_python(input_value)
        # debug(repr(exc_info.value))
    else:
        assert multiple_tuple_schema.validate_python(input_value) == expected


def test_multiple_tuple_repeat(multiple_tuple_schema):
    t = (42, None)
    assert multiple_tuple_schema.validate_python({'f1': (1, t), 'f2': (2, t)}) == {
        'f1': (1, (42, None)),
        'f2': (2, (42, None)),
    }


def test_multiple_tuple_recursion(multiple_tuple_schema):
    data = [1]
    data.append(data)
    with pytest.raises(ValidationError) as exc_info:
        multiple_tuple_schema.validate_python({'f1': data, 'f2': data})

    assert exc_info.value.errors() == [
        {
            'kind': 'recursion_loop',
            'loc': ['f1', 1],
            'message': 'Recursion error - cyclic reference detected',
            'input_value': [1, IsList(length=2)],
        },
        {
            'kind': 'recursion_loop',
            'loc': ['f2', 1],
            'message': 'Recursion error - cyclic reference detected',
            'input_value': [1, IsList(length=2)],
        },
    ]


def test_multiple_tuple_recursion_once(multiple_tuple_schema):
    data = [1]
    data.append(data)
    with pytest.raises(ValidationError) as exc_info:
        multiple_tuple_schema.validate_python({'f1': data, 'f2': data})

    assert exc_info.value.errors() == [
        {
            'kind': 'recursion_loop',
            'loc': ['f1', 1],
            'message': 'Recursion error - cyclic reference detected',
            'input_value': [1, IsList(length=2)],
        },
        {
            'kind': 'recursion_loop',
            'loc': ['f2', 1],
            'message': 'Recursion error - cyclic reference detected',
            'input_value': [1, IsList(length=2)],
        },
    ]


def test_recursive_wrap():
    def wrap_func(input_value, *, validator, **kwargs):
        return validator(input_value) + (42,)

    v = SchemaValidator(
        {
            'type': 'function',
            'ref': 'wrapper',
            'mode': 'wrap',
            'function': wrap_func,
            'schema': {
                'type': 'tuple-fix-len',
                'items_schema': [
                    {'type': 'int'},
                    {'type': 'nullable', 'schema': {'type': 'recursive-ref', 'schema_ref': 'wrapper'}},
                ],
            },
        }
    )
    assert v.validate_python((1, None)) == (1, None, 42)
    assert v.validate_python((1, (2, (3, None)))) == (1, (2, (3, None, 42), 42), 42)
    t = [1]
    t.append(t)
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python(t)
    assert exc_info.value.errors() == [
        {
            'kind': 'recursion_loop',
            'loc': [1],
            'message': 'Recursion error - cyclic reference detected',
            'input_value': IsList(positions={0: 1}, length=2),
        }
    ]


def test_union_strictness():
    v = SchemaValidator(
        {
            'fields': {
                'a': {'schema': {'type': 'int', 'ref': 'int-type'}},
                'b': {
                    'schema': {
                        'type': 'union',
                        'choices': [{'type': 'recursive-ref', 'schema_ref': 'int-type'}, {'type': 'str'}],
                    }
                },
            },
            'type': 'typed-dict',
        }
    )
    assert v.validate_python({'a': 1, 'b': '2'}) == {'a': 1, 'b': '2'}


@pytest.mark.parametrize('strict', [True, False], ids=lambda s: f'strict={s}')
def test_union_cycle(strict):
    s = SchemaValidator(
        {
            'choices': [
                {
                    'fields': {
                        'foobar': {
                            'schema': {
                                'items_schema': {'schema_ref': 'root-schema', 'type': 'recursive-ref'},
                                'type': 'list',
                            }
                        }
                    },
                    'type': 'typed-dict',
                }
            ],
            'strict': strict,
            'ref': 'root-schema',
            'type': 'union',
        }
    )

    data = {'foobar': []}
    data['foobar'].append(data)

    with pytest.raises(ValidationError) as exc_info:
        s.validate_python(data)
    assert exc_info.value.errors() == [
        {
            'kind': 'recursion_loop',
            'loc': ['typed-dict', 'foobar', 0],
            'message': 'Recursion error - cyclic reference detected',
            'input_value': {'foobar': [{'foobar': IsList(length=1)}]},
        }
    ]
