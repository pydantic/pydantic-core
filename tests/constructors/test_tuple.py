import re
from collections import deque
from typing import Any

import pytest

from pydantic_core import SchemaValidator, ValidationError, core_schema

from ..conftest import Err


@pytest.mark.parametrize(
    'mode,items', [('variable', {'type': 'int'}), ('positional', [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}])]
)
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ((1, 2, '3'), (1, 2, '3')),
        ([1, 2, '3'], [1, 2, '3']),
        ({1: 10, 2: 20, '3': '30'}, {1: 10, 2: 20, '3': '30'}),
        ({1, 2, '3'}, {1, 2, '3'}),
        (frozenset([1, 2, '3']), frozenset([1, 2, '3'])),
        (deque([1, 2, '3']), deque([1, 2, '3'])),
    ],
    ids=repr,
)
def test_tuple_construct(input_value, expected, mode, items):
    v = SchemaValidator({'type': f'tuple-{mode}', 'items_schema': items})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.construct_python(input_value)
    else:
        assert v.construct_python(input_value) == expected
        assert v.construct_python(input_value, recursive=True) == expected


@pytest.mark.parametrize(
    'mode,items', [('variable', {'type': 'int'}), ('positional', [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}])]
)
@pytest.mark.parametrize(
    'input_value,expected',
    [({1: 10, 2: 20, '3': '30'}.keys(), (1, 2, '3')), ({1: 10, 2: 20, '3': '30'}.values(), (10, 20, '30'))],
)
def test_tuple_construct_special(input_value, expected, mode, items):
    v = SchemaValidator({'type': f'tuple-{mode}', 'items_schema': items})
    assert v.construct_python(input_value) is input_value
    assert v.construct_python(input_value, recursive=True) == expected


def test_generator_error():
    def gen(error: bool):
        yield 1
        yield 2
        if error:
            raise RuntimeError('error')
        yield 3

    v = SchemaValidator({'type': 'tuple-variable', 'items_schema': {'type': 'int'}})
    v.construct_python(gen(False))
    # v.construct_python(gen(True)) # TODO: fixme

    msg = r'Error iterating over object, error: RuntimeError: error \[type=iteration_error,'
    with pytest.raises(ValidationError, match=msg):
        v.construct_python(gen(True), recursive=True)


class Child:
    __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
    a: int
    b: int

    def __init__(self, a, b):
        self.a = a
        self.b = b

    def __eq__(self, other):
        return self.a == other.a and self.b == other.b


@pytest.mark.parametrize(
    'input_val,output_val,recursive',
    [
        ((10, 'wrong'), (10, 'wrong'), False),
        ((10, {'a': 10, 'b': 'wrong'}), (10, {'a': 10, 'b': 'wrong'}), False),
        ((10, {'a': 10, 'b': 'wrong'}), (10, Child(10, 'wrong')), True),
        # Switch the first and last element
        # Because int can't be coerced to Child and Child dict can't be coerced to int, result is same as input
        (({'a': 10, 'b': 'wrong'}, 10), ({'a': 10, 'b': 'wrong'}, 10), True),
    ],
)
def test_positional_recursive(input_val: Any, output_val: Any, recursive: bool):
    # Tuple[int, Child]
    v = SchemaValidator(
        core_schema.tuple_positional_schema(
            items_schema=[
                core_schema.int_schema(),
                core_schema.model_schema(
                    Child,
                    core_schema.model_fields_schema(
                        {
                            'a': core_schema.model_field(core_schema.int_schema()),
                            'b': core_schema.model_field(core_schema.int_schema()),
                        }
                    ),
                    root_model=False,
                ),
            ]
        )
    )

    assert v.construct_python(input_val, recursive=recursive) == output_val


@pytest.mark.parametrize(
    'input_val,output_val,recursive',
    [
        ((), (), False),
        ((), (), True),
        (('wrong',), ('wrong',), True),
        (({'a': 10, 'b': 'wrong'}, 10), ({'a': 10, 'b': 'wrong'}, 10), False),
        (({'a': 10, 'b': 'wrong'}, 10), (Child(10, 'wrong'), 10), True),
        # swapping order should have no effect
        ((10, {'a': 10, 'b': 'wrong'}), (10, Child(10, 'wrong')), True),
    ],
)
def test_variable_recursive(input_val: Any, output_val: Any, recursive: bool):
    class Child:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        a: int
        b: int

        def __init__(self, a, b):
            self.a = a
            self.b = b

        def __eq__(self, other):
            return self.a == other.a and self.b == other.b

    # Tuple[Child, ...]
    v = SchemaValidator(
        core_schema.tuple_variable_schema(
            items_schema=core_schema.model_schema(
                Child,
                core_schema.model_fields_schema(
                    {
                        'a': core_schema.model_field(core_schema.int_schema()),
                        'b': core_schema.model_field(core_schema.int_schema()),
                    }
                ),
                root_model=False,
            )
        )
    )

    assert v.construct_python(input_val, recursive=recursive) == output_val
