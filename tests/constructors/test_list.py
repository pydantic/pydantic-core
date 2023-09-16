import collections.abc
import re
from collections import deque
from dataclasses import dataclass
from typing import Any, Dict, Iterator, List, Union

import pytest
from dirty_equals import HasRepr, IsStr

from pydantic_core import SchemaValidator, ValidationError, core_schema

from ..conftest import Err


def test_list_no_copy():
    v = SchemaValidator({'type': 'list'})
    assert v.construct_python([1, 2, 3]) is not [1, 2, 3]


def test_list_json():
    v = SchemaValidator({'type': 'list', 'items_schema': {'type': 'int'}})
    assert v.construct_json('[1, "2", 3]') == [1, '2', 3]
    assert v.construct_json('1') == 1


@pytest.mark.parametrize(
    'kwargs,input_value,expected',
    [
        ({}, [1, 2, 3, 4], [1, 2, 3, 4]),
        ({'min_length': 3}, [1, 2, 3, 4], [1, 2, 3, 4]),
        ({'min_length': 3}, [1, 2], [1, 2]),
        ({'min_length': 1}, [], []),
        ({'max_length': 4}, [1, 2, 3, 4], [1, 2, 3, 4]),
        ({'max_length': 3}, [1, 2, 3, 4], [1, 2, 3, 4]),
        ({'max_length': 3}, [1, 2, 3, 4, 5, 6, 7], [1, 2, 3, 4, 5, 6, 7]),
        ({'max_length': 1}, [1, 2], [1, 2]),
        ({'max_length': 4, 'items_schema': {'type': 'int'}}, [0, 1, 2, 3, 4, 5, 6, 7, 8], [0, 1, 2, 3, 4, 5, 6, 7, 8]),
    ],
)
def test_list_length_constraints(kwargs: Dict[str, Any], input_value, expected):
    v = SchemaValidator({'type': 'list', **kwargs})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.construct_python(input_value)
    else:
        assert v.construct_python(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [([1, 2, 3, 4], [1, 2, 3, 4]), ([1, 2, 3, 4, 5], [1, 2, 3, 4, 5]), ([1, 2, 3, 'x', 4], [1, 2, 3, 'x', 4])],
)
def test_list_length_constraints_omit(input_value, expected):
    v = SchemaValidator(
        {
            'type': 'list',
            'items_schema': {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'},
            'max_length': 4,
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.construct_python(input_value)
    else:
        assert v.construct_python(input_value) == expected


def test_list_function():
    """Validation functions have no effect"""

    def f(input_value, info):
        return input_value * 2

    v = SchemaValidator(
        {'type': 'list', 'items_schema': {'type': 'function-plain', 'function': {'type': 'general', 'function': f}}}
    )

    assert v.construct_python([1, 2, 3]) == [1, 2, 3]


def test_list_function_val_error():
    """Validation functions are not called"""

    def f(input_value, info):
        raise ValueError(f'error {input_value}')

    v = SchemaValidator(
        {'type': 'list', 'items_schema': {'type': 'function-plain', 'function': {'type': 'general', 'function': f}}}
    )

    assert v.construct_python([1, 2]) == [1, 2]


def test_generator_error():
    """Generators are traversed when they're constructed; if they error its a ValidationError"""

    def gen(error: bool):
        yield 1
        yield 2
        if error:
            raise RuntimeError('error')
        yield 3

    v = SchemaValidator({'type': 'list', 'items_schema': {'type': 'int'}})
    # Does not raise because the generator is not traversed # TODO: <-- make sure this is actually the case
    v.construct_python(gen(False))
    # v.construct_python(gen(True)) # TODO: uncomment
    # Raises validation error when traversing the generator
    with pytest.raises(ValidationError) as exc_info:
        v.construct_python(gen(True), recursive=True) == gen(True)

    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'iteration_error',
            'loc': (2,),
            'msg': 'Error iterating over object, error: RuntimeError: error',
            'input': HasRepr(IsStr(regex='<generator object test_generator_error.<locals>.gen at 0x[0-9a-fA-F]+>')),
            'ctx': {'error': 'RuntimeError: error'},
        }
    ]


@pytest.mark.parametrize('items_schema', ['int', 'any'])
def test_bad_iter(items_schema):
    """Iterators are traversed when they're constructed; if they error its a ValidationError"""

    class BadIter:
        def __init__(self, success: bool):
            self._success = success
            self._index = 0

        def __iter__(self):
            return self

        def __len__(self):
            return 2

        def __next__(self):
            self._index += 1
            if self._index == 1:
                return 1
            elif self._success:
                raise StopIteration()
            else:
                raise RuntimeError('broken')

        def __eq__(self, other):
            if isinstance(other, BadIter):
                return self._success == other._success
            return False

    v = SchemaValidator({'type': 'list', 'items_schema': {'type': items_schema}})
    # If recursive is False, a passed iterator won't be traversed or coerced, and will be the same object
    ok_iter = BadIter(True)
    assert v.construct_python(ok_iter) is ok_iter
    # If recursive is True, a passed iterator must be traversed, but it needs a concrete return type; so it gains the
    # type of the annotation, which in this case is list:
    assert v.construct_python(BadIter(True), recursive=True) == [1]
    # If the iterator raises a RuntimeError or any other exception, a Validation error is raised:
    bad_iter = BadIter(False)
    with pytest.raises(ValidationError) as exc_info:
        v.construct_python(bad_iter, recursive=True)

    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'iteration_error',
            'loc': (1,),
            'msg': 'Error iterating over object, error: RuntimeError: broken',
            'input': bad_iter,  # IsInstance(BadIter) doesn't want to work for some reason
            'ctx': {'error': 'RuntimeError: broken'},
        }
    ]


class MySequence(collections.abc.Sequence):
    def __init__(self, data: List[Any]):
        self._data = data

    def __getitem__(self, index: int) -> Any:
        return self._data[index]

    def __len__(self):
        return len(self._data)

    def __repr__(self) -> str:
        return f'MySequence({repr(self._data)})'

    def __eq__(self, other):
        return self._data == other._data


class MyMapping(collections.abc.Mapping):
    def __init__(self, data: Dict[Any, Any]) -> None:
        self._data = data

    def __getitem__(self, key: Any) -> Any:
        return self._data[key]

    def __iter__(self) -> Iterator[Any]:
        return iter(self._data)

    def __len__(self) -> int:
        return len(self._data)

    def __repr__(self) -> str:
        return f'MyMapping({repr(self._data)})'


@dataclass
class ListInputTestCase:
    input: Any
    output: Union[Any, Err]
    strict: Union[bool, None] = None
    recursive: bool = False


CONSTRUCTION_INPUTS: List[Any] = [
    ('123', '123'),
    (b'123', b'123'),
    ((1, 2, 3), (1, 2, 3)),
    (frozenset((1, 2, 3)), frozenset((1, 2, 3))),
    (set((1, 2, 3)), set((1, 2, 3))),
    (deque([1, 2, 3]), deque([1, 2, 3])),
    (
        MySequence([1, 2, 3]),
        MySequence([1, 2, 3]),
    ),  # Custom generics (thankfully) work, but they must be constructable from an __init__ method
    # ((x for x in [1, 2, 3]), [1, 2, 3]), # TODO: generators not traversed for some reason; I'm probbaly consuming them too early somewhere
]


@pytest.mark.parametrize(
    'testcase',
    [
        *[ListInputTestCase([1, 2, 3], [1, 2, 3], strict) for strict in (True, False, None)],
        *[ListInputTestCase(inp, oup, True) for (inp, oup) in CONSTRUCTION_INPUTS],
        *[ListInputTestCase(inp, oup, True, True) for (inp, oup) in CONSTRUCTION_INPUTS],
        *[ListInputTestCase(inp, oup, False) for (inp, oup) in CONSTRUCTION_INPUTS],
        *[ListInputTestCase(inp, oup, False, True) for (inp, oup) in CONSTRUCTION_INPUTS],
        *[
            ListInputTestCase(inp, inp, False)
            for inp in ['123', b'123', MyMapping({1: 'a', 2: 'b', 3: 'c'}), {1: 'a', 2: 'b', 3: 'c'}]
        ],
        *[
            ListInputTestCase(inp, inp, False, True)
            for inp in ['123', b'123', MyMapping({1: 'a', 2: 'b', 3: 'c'}), {1: 'a', 2: 'b', 3: 'c'}]
        ],
    ],
    ids=repr,
)
def test_list_allowed_inputs_python(testcase: ListInputTestCase):
    v = SchemaValidator(core_schema.list_schema(core_schema.int_schema(), strict=testcase.strict))
    if isinstance(testcase.output, Err):
        with pytest.raises(ValidationError, match=re.escape(testcase.output.message)):
            v.construct_python(testcase.input)
    else:
        output = v.construct_python(testcase.input, recursive=testcase.recursive)
        assert output == testcase.output


SPECIAL_CASES = [
    ({1: 'a', 2: 'b', 3: 'c'}.keys(), [1, 2, 3]),  # Special case
    ({'a': 1, 'b': 2, 'c': 3}.values(), [1, 2, 3]),  # Special case
    (MyMapping({1: 'a', 2: 'b', 3: 'c'}).keys(), [1, 2, 3]),  # Special case
    (MyMapping({'a': 1, 'b': 2, 'c': 3}).values(), [1, 2, 3]),  # Special case
]


@pytest.mark.parametrize('testcase', [*[ListInputTestCase(inp, oup, False) for inp, oup in SPECIAL_CASES]])
def test_list_allowed_inputs_special(testcase: ListInputTestCase):
    """All of these cases are passed verbatim when recursive=False, and coerced to annotated type when recursive=True"""
    v = SchemaValidator(core_schema.list_schema(core_schema.int_schema(), strict=testcase.strict))
    if isinstance(testcase.output, Err):
        with pytest.raises(ValidationError, match=re.escape(testcase.output.message)):
            v.construct_python(testcase.input)
    else:
        output = v.construct_python(testcase.input, recursive=False)
        assert output is testcase.input
        output = v.construct_python(testcase.input, recursive=True)
        assert output is not testcase.input
        assert output == testcase.output


def test_custom_subclass_malformed_init():
    v = SchemaValidator(core_schema.list_schema(core_schema.int_schema(), strict=False))

    class MySequenceNoInit(collections.abc.Sequence):
        def __getitem__(self, index: int) -> Any:
            return self._data[index]

        def __len__(self):
            return len(self._data)

        def __repr__(self) -> str:
            return f'MySequence({repr(self._data)})'

        def __eq__(self, other):
            return self._data == other._data

    instance = MySequenceNoInit()
    instance._data = [1, 2, 3]

    # Non recursive is fine
    assert v.construct_python(instance, recursive=False) is instance
    # Recursive tries to call `MySequenceNoInit([1, 2, 3])`, which errors
    # TODO: this should probably be wrapped in a ValidationError
    with pytest.raises(TypeError, match=r'MySequenceNoInit\(\) takes no arguments'):
        v.construct_python(instance, recursive=True)


def test_list_of_model():
    class Child:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        a: int
        b: int

    class TestRootModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        root: List[Child]

    v = SchemaValidator(
        core_schema.model_schema(
            TestRootModel,
            schema=core_schema.list_schema(
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
            ),
            root_model=True,
        )
    )

    m1: TestRootModel = v.construct_python([{'a': 10, 'b': 'something'}])
    assert m1.root == [{'a': 10, 'b': 'something'}]
    assert m1.__pydantic_fields_set__ == {'root'}

    m2: TestRootModel = v.construct_python([{'a': 10, 'b': 'something'}], recursive=True)
    assert isinstance(m2.root[0], Child)
    assert m2.root[0].a == 10
    assert m2.root[0].b == 'something'
    assert m2.__pydantic_fields_set__ == {'root'}

    # Make sure attempting with incorrect annotated object doesn't raise and just ends up as-is
    m3 = v.construct_python(['wrong', {'a': 10, 'b': 'something'}], recursive=True)
    assert isinstance(m3.root[0], str)
    assert m3.root[0] == 'wrong'
    assert isinstance(m3.root[1], Child)
    assert m3.root[1].a == 10
    assert m3.root[1].b == 'something'
    assert m3.__pydantic_fields_set__ == {'root'}
