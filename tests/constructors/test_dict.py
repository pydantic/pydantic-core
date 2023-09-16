import re
from collections import OrderedDict
from collections.abc import Mapping

import pytest
from dirty_equals import HasRepr, IsStr

from pydantic_core import SchemaValidator, ValidationError, core_schema

from ..conftest import Err, PyAndJson


@pytest.mark.parametrize('recursive', [False, True])
def test_dict(py_and_json: PyAndJson, recursive):
    v = py_and_json({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    assert v.construct_test({'1': 2, '3': 4}, recursive=recursive) == {'1': 2, '3': 4}
    # Strict should have no effect
    v = py_and_json({'type': 'dict', 'strict': True, 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    assert v.construct_test({'1': 2, '3': 4}) == {'1': 2, '3': 4}
    assert v.construct_test({}, recursive=recursive) == {}
    assert v.construct_test([], recursive=recursive) == []


class Foobar:
    x: int

    def __init__(self, x):
        self.x = x

    def __eq__(self, other):
        return self.x == other.x


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'1': b'1', '2': b'2'}, {'1': b'1', '2': b'2'}),
        (OrderedDict(a=b'1', b='2'), OrderedDict(a=b'1', b='2')),
        ({}, {}),
        ('foobar', 'foobar'),
        ([], []),
        ([('x', 'y')], [('x', 'y')]),
        ([('x', 'y'), ('z', 'z')], [('x', 'y'), ('z', 'z')]),
        ((), ()),
        ((('x', 'y'),), (('x', 'y'),)),
        (Foobar(1), Foobar(1)),
    ],
    ids=repr,
)
def test_dict_cases(input_value, expected, recursive):
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'str'}, 'values_schema': {'type': 'str'}})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.construct_python(input_value, recursive=recursive)
    else:
        assert v.construct_python(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [False, True])
def test_dict_value_no_error(py_and_json: PyAndJson, recursive):
    v = py_and_json({'type': 'dict', 'values_schema': {'type': 'int'}})
    assert v.construct_test({'a': 2, 'b': '4'}, recursive=recursive) == {'a': 2, 'b': '4'}
    assert v.construct_test({'a': 2, 'b': 'wrong'}, recursive=recursive) == {'a': 2, 'b': 'wrong'}


@pytest.mark.parametrize('recursive', [False, True])
def test_dict_no_error_key_int(recursive):
    v = SchemaValidator({'type': 'dict', 'values_schema': {'type': 'int'}})
    assert v.construct_python({1: 2, 3: 'wrong', -4: 'wrong2'}, recursive=recursive) == {1: 2, 3: 'wrong', -4: 'wrong2'}


@pytest.mark.parametrize('recursive', [False, True])
def test_dict_no_error_key_other(recursive):
    v = SchemaValidator({'type': 'dict', 'values_schema': {'type': 'int'}})
    assert v.construct_python({1: 2, (1, 2): 'wrong'}, recursive=recursive) == {1: 2, (1, 2): 'wrong'}


@pytest.mark.parametrize('recursive', [False, True])
def test_dict_any_value(recursive):
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'str'}})
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'str'}})
    assert v.construct_python({'1': 1, '2': 'a', '3': None}, recursive=recursive) == {'1': 1, '2': 'a', '3': None}


def test_mapping():
    class MyMapping(Mapping):
        def __init__(self, d):
            self._d = d

        def __getitem__(self, key):
            return self._d[key]

        def __iter__(self):
            return iter(self._d)

        def __len__(self):
            return len(self._d)

    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    assert v.construct_python(MyMapping({'1': 2, 3: '4'})) == {'1': 2, 3: '4'}
    # Strict should have no effect
    v = SchemaValidator(
        {'type': 'dict', 'strict': True, 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}}
    )
    assert v.construct_python(MyMapping({'1': 2, 3: '4'})) == {'1': 2, 3: '4'}


def test_key_no_error():
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    assert v.construct_python({'1': True}) == {'1': True}
    assert v.construct_python({'x': 1}) == {'x': 1}


def test_mapping_error():
    class BadMapping(Mapping):
        def __getitem__(self, key):
            raise None

        def __iter__(self):
            raise RuntimeError('intentional error')

        def __len__(self):
            return 1

    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    # Mapping items are not traversed...
    bad_mapping = BadMapping()
    assert v.construct_python(bad_mapping) is bad_mapping

    # Unless recursive
    with pytest.raises(ValidationError) as exc_info:
        v.construct_python(BadMapping(), recursive=True)

    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'mapping_type',
            'loc': (),
            'msg': 'Input should be a valid mapping, error: RuntimeError: intentional error',
            'input': HasRepr(IsStr(regex='.+BadMapping object at.+')),
            'ctx': {'error': 'RuntimeError: intentional error'},
        }
    ]


@pytest.mark.parametrize('mapping_items', [[(1,)], ['foobar'], [(1, 2, 3)], 'not list'])
def test_mapping_error_yield_1(mapping_items):
    class BadMapping(Mapping):
        def items(self):
            return mapping_items

        def __iter__(self):
            pytest.fail('unexpected call to __iter__')

        def __getitem__(self, key):
            pytest.fail('unexpected call to __getitem__')

        def __len__(self):
            return 1

    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})

    # Mapping items are not traversed...
    bad_mapping = BadMapping()
    assert v.construct_python(bad_mapping) is bad_mapping

    # Unless recursive
    with pytest.raises(ValidationError) as exc_info:
        v.construct_python(BadMapping(), recursive=True)

    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'mapping_type',
            'loc': (),
            'msg': 'Input should be a valid mapping, error: Mapping items must be tuples of (key, value) pairs',
            'input': HasRepr(IsStr(regex='.+BadMapping object at.+')),
            'ctx': {'error': 'Mapping items must be tuples of (key, value) pairs'},
        }
    ]


def test_json_dict():
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    assert v.construct_json('{"1": 2, "3": 4}') == {'1': 2, '3': 4}
    assert v.construct_json('1') == 1


def test_recursive_model_key():
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

    # Dict[str, Child]
    v = SchemaValidator(core_schema.dict_schema(keys_schema=child_schema, values_schema=core_schema.str_schema()))

    assert v.construct_python(None) is None
    assert v.construct_python(None, recursive=True) is None

    hash_dict = HashableDict({'a': 10, 'b': 'wrong'})
    m = v.construct_python({hash_dict: 'string'})
    assert m == {hash_dict: 'string'}
    assert hash_dict in m
    assert m[hash_dict] == 'string'
    m = v.construct_python({hash_dict: 'string'}, recursive=True)
    assert Child(10, 'wrong') in m
    assert m[Child(10, 'wrong')] == 'string'

    # Presumably a round-trip solution would also customize serialization back into HashableDict


def test_recursive_model_value():
    class Child:
        a: int
        b: int

    child_schema = core_schema.model_schema(
        Child,
        core_schema.model_fields_schema(
            {
                'a': core_schema.model_field(core_schema.int_schema()),
                'b': core_schema.model_field(core_schema.int_schema()),
            }
        ),
    )

    # Dict[str, Child]
    v = SchemaValidator(core_schema.dict_schema(keys_schema=core_schema.str_schema(), values_schema=child_schema))

    assert v.construct_python(None) is None
    assert v.construct_python(None, recursive=True) is None

    assert v.construct_python({'string': {'a': 10, 'b': 'wrong'}}) == {'string': {'a': 10, 'b': 'wrong'}}
    m = v.construct_python({'string': {'a': 10, 'b': 'wrong'}}, recursive=True)
    assert isinstance(m['string'], Child)
    assert m['string'].a == 10
    assert m['string'].b == 'wrong'
