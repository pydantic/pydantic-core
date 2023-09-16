import re
from collections import deque
from typing import Any, Dict

import pytest

from pydantic_core import SchemaValidator, ValidationError, core_schema

from ..conftest import Err, PyAndJson


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [([], []), ([1, 2, 3], [1, 2, 3]), ([1, 2, '3'], [1, 2, '3']), ([1, 2, 3, 2, 3], [1, 2, 3, 2, 3]), (5, 5)],
)
def test_set_ints_both(py_and_json: PyAndJson, input_value, expected, recursive):
    v = py_and_json({'type': 'set', 'items_schema': {'type': 'int'}})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.construct_test(input_value, recursive=recursive)
    else:
        assert v.construct_test(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize('input_value,expected', [([1, 2.5, '3'], [1, 2.5, '3'])])
def test_set_no_validators_both(py_and_json: PyAndJson, input_value, expected, recursive):
    v = py_and_json({'type': 'set'})
    assert v.construct_test(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected', [([1, 2.5, '3'], [1, 2.5, '3']), ('foo', 'foo'), (1, 1), (1.0, 1.0), (False, False)]
)
def test_frozenset_no_validators_both(py_and_json: PyAndJson, input_value, expected, recursive):
    v = py_and_json({'type': 'set'})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message):
            v.construct_test(input_value, recursive=recursive)
    else:
        assert v.construct_test(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({1, 2, 3}, {1, 2, 3}),
        (set(), set()),
        ([1, 2, 3, 2, 3], [1, 2, 3, 2, 3]),
        ([], []),
        ((1, 2, 3, 2, 3), (1, 2, 3, 2, 3)),
        ((), ()),
        (frozenset([1, 2, 3, 2, 3]), frozenset([1, 2, 3, 2, 3])),
        (deque((1, 2, '3')), deque([1, 2, '3'])),
        ({1: 10, 2: 20, '3': '30'}.keys(), {1, 2, '3'}),
        ({1: 10, 2: 20, '3': '30'}, {1: 10, 2: 20, '3': '30'}),
        ({'abc'}, {'abc'}),
        ({1: 2}, {1: 2}),
        ('abc', 'abc'),
    ],
)
def test_set_ints_python(input_value, expected, recursive):
    v = SchemaValidator({'type': 'set', 'items_schema': {'type': 'int'}})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.construct_python(input_value, recursive=recursive)
    else:
        assert v.construct_python(input_value, recursive=recursive) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [({1: 10, 2: 20, '3': '30'}.values(), {10, 20, '30'}), ((x for x in [1, 2, '3']), {1, 2, '3'})],
)
def test_set_ints_special(input_value, expected):
    v = SchemaValidator({'type': 'set', 'items_schema': {'type': 'int'}})

    assert v.construct_python(input_value, recursive=False) is input_value
    output = v.construct_python(input_value, recursive=True)
    assert output == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize('input_value,expected', [([1, 2.5, '3'], [1, 2.5, '3']), ([(1, 2), (3, 4)], [(1, 2), (3, 4)])])
def test_set_no_validators_python(input_value, expected, recursive):
    v = SchemaValidator({'type': 'set'})
    assert v.construct_python(input_value, recursive=recursive) == expected


def test_set_no_multiple_errors():
    v = SchemaValidator({'type': 'set', 'items_schema': {'type': 'int'}})
    assert v.construct_python(['a', (1, 2), []]) == ['a', (1, 2), []]


def generate_repeats():
    for i in 1, 2, 3:
        yield i
        yield i


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'kwargs,input_value,expected',
    [
        ({'strict': True}, {1, 2, 3}, {1, 2, 3}),
        ({'strict': True}, set(), set()),
        ({'strict': True}, [1, 2, 3, 2, 3], [1, 2, 3, 2, 3]),
        ({'strict': True}, [], []),
        ({'strict': True}, (), ()),
        ({'strict': True}, (1, 2, 3), (1, 2, 3)),
        ({'strict': True}, frozenset([1, 2, 3]), frozenset([1, 2, 3])),
        ({'strict': True}, 'abc', 'abc'),
        ({'min_length': 3}, {1, 2, 3}, {1, 2, 3}),
        ({'min_length': 3}, {1, 2}, {1, 2}),
        ({'max_length': 3}, {1, 2, 3, 4}, {1, 2, 3, 4}),
        ({'max_length': 3}, [1, 2, 3, 4], [1, 2, 3, 4]),
        ({'max_length': 3, 'items_schema': {'type': 'int'}}, {1, 2, 3, 4}, {1, 2, 3, 4}),
        ({'max_length': 3, 'items_schema': {'type': 'int'}}, [1, 2, 3, 4], [1, 2, 3, 4]),
        # no length check after set creation
        ({'max_length': 3}, [1, 1, 2, 2, 3, 3], [1, 1, 2, 2, 3, 3]),
        # ({'max_length': 3}, generate_repeats(), {1, 2, 3}), # TODO: generators not traversed for some reason; I'm probbaly consuming them too early somewhere
    ],
    ids=repr,
)
def test_set_kwargs_no_effect(kwargs: Dict[str, Any], input_value, expected, recursive):
    v = SchemaValidator({'type': 'set', **kwargs})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            r = v.construct_python(input_value, recursive=recursive)
            print(f'unexpected result: {r!r}')
    else:
        assert v.construct_python(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('input_value,expected', [({1, 2, 3}, {1, 2, 3}), ([1, 2, 3], [1, 2, 3])])
def test_union_set_list(input_value, expected):
    v = SchemaValidator({'type': 'union', 'choices': [{'type': 'set'}, {'type': 'list'}]})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.construct_python(input_value)
    else:
        assert v.construct_python(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected', [({1, 2, 3}, {1, 2, 3}), ({'a', 'b', 'c'}, {'a', 'b', 'c'}), ([1, 'a'], [1, 'a'])]
)
def test_union_set_int_set_str(input_value, expected):
    v = SchemaValidator(
        {
            'type': 'union',
            'choices': [
                {'type': 'set', 'items_schema': {'type': 'int', 'strict': True}},
                {'type': 'set', 'items_schema': {'type': 'str', 'strict': True}},
            ],
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.construct_python(input_value)
        if expected.errors is not None:
            assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        assert v.construct_python(input_value) == expected


def test_set_as_dict_keys(py_and_json: PyAndJson):
    v = py_and_json({'type': 'dict', 'keys_schema': {'type': 'set'}, 'values_schema': {'type': 'int'}})
    assert v.construct_test({'foo': 'bar'}) == {'foo': 'bar'}
    assert v.construct_test({'foo': 'bar'}, recursive=True) == {'foo': 'bar'}


def test_generator_error():
    """Generators are consumed when constructing even if recursive is False"""

    def gen(error: bool):
        yield 1
        yield 2
        if error:
            raise RuntimeError('my error')
        yield 3

    v = SchemaValidator({'type': 'set', 'items_schema': {'type': 'int'}})
    goodgen = gen(False)
    assert v.construct_python(goodgen) is goodgen
    assert v.construct_python(goodgen, recursive=True) == {1, 2, 3}

    # No error because recursive=False
    badgen = gen(True)
    assert v.construct_python(badgen) is badgen

    msg = r'Error iterating over object, error: RuntimeError: my error \[type=iteration_error,'
    with pytest.raises(ValidationError, match=msg):
        v.construct_python(gen(True), recursive=True)


@pytest.mark.parametrize(
    'input_value,items_schema,expected',
    [
        pytest.param(
            {1: 10, 2: 20, '3': '30'}.items(),
            {'type': 'tuple-variable', 'items_schema': {'type': 'any'}},
            {(1, 10), (2, 20), ('3', '30')},
            id='Tuple[Any, Any]',
        ),
        pytest.param(
            {1: 10, 2: 20, '3': '30'}.items(),
            {'type': 'tuple-variable', 'items_schema': {'type': 'int'}},
            {(1, 10), (2, 20), ('3', '30')},
            id='Tuple[int, int]',
        ),
        pytest.param({1: 10, 2: 20, '3': '30'}.items(), {'type': 'any'}, {(1, 10), (2, 20), ('3', '30')}, id='Any'),
    ],
)
def test_set_from_dict_items(input_value, items_schema, expected):
    v = SchemaValidator({'type': 'set', 'items_schema': items_schema})
    output = v.construct_python(input_value, recursive=True)
    assert isinstance(output, set)
    assert output == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ([], []),
        ([1, '2', b'3'], [1, '2', b'3']),
        ({1, '2', b'3'}, {1, '2', b'3'}),
        (frozenset([1, '2', b'3']), frozenset([1, '2', b'3'])),
        # (deque([1, '2', b'3']), deque([1, '2', b'3'])), # TODO
    ],
)
def test_set_any(input_value, expected, recursive):
    v = SchemaValidator({'type': 'set'})
    output = v.construct_python(input_value, recursive=recursive)
    assert output == expected


def test_set_recursive():
    class HashableDict(dict):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, **kwargs)

        def __hash__(self):
            return id(self) >> 4  # bad, but sufficient

    class Child:
        a: int
        b: int

        def __init__(self, a, b):
            self.a = a
            self.b = b

        def __hash__(self):
            return hash((self.a, self.b))

        def __eq__(self, other):
            return self.a == other.a and self.b == other.b

    child_schema = core_schema.model_schema(
        Child,
        core_schema.model_fields_schema(
            {
                'a': core_schema.model_field(core_schema.int_schema()),
                'b': core_schema.model_field(core_schema.int_schema()),
            }
        ),
    )

    # Set[Child]
    v = SchemaValidator(core_schema.set_schema(items_schema=child_schema))

    assert v.construct_python(None) is None
    assert v.construct_python(None, recursive=True) is None

    assert v.construct_python({'some', 'strings'}) == {'some', 'strings'}
    assert v.construct_python({'some', 'strings'}, recursive=True) == {'some', 'strings'}

    # Non-recursive remains a HashableDict
    hash_dict = HashableDict({'a': 10, 'b': 'wrong'})
    m = v.construct_python({hash_dict})
    assert m == {hash_dict}
    assert hash_dict in m
    # Recursive gets converted to Child instance
    m = v.construct_python({hash_dict}, recursive=True)
    assert m == {Child(10, 'wrong')}
    assert Child(10, 'wrong') in m

    # Test mixture of coercable and non-coercable
    mixture_set = {'a string', hash_dict}
    m = v.construct_python(mixture_set)
    assert 'a string' in m
    assert hash_dict in m
    assert m == mixture_set
    m = v.construct_python(mixture_set, recursive=True)
    assert 'a string' in m
    assert Child(10, 'wrong') in m

    # Presumably a round-trip solution would also customize serialization back into HashableDict
