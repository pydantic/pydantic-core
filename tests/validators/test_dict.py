import re
from collections import OrderedDict
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Any, Dict, List

import pytest
from dirty_equals import HasRepr, IsStr

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err, PyAndJson


def test_dict(py_and_json: PyAndJson):
    v = py_and_json({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    assert v.validate_test({'1': 2, '3': 4}) == {1: 2, 3: 4}
    v = py_and_json({'type': 'dict', 'strict': True, 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    assert v.validate_test({'1': 2, '3': 4}) == {1: 2, 3: 4}
    assert v.validate_test({}) == {}
    with pytest.raises(ValidationError, match='Input should be a valid dictionary'):
        v.validate_test([])


@dataclass
class Foobar:
    x = 1


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'1': b'1', '2': b'2'}, {'1': 1, '2': 2}),
        (OrderedDict(a=b'1', b='2'), {'a': 1, 'b': 2}),
        ({}, {}),
        ('foobar', Err("Input should be a valid dictionary [type=dict_type, input_value='foobar', input_type=str]")),
        ([], {}),
        ([('x', '1')], {'x': 1}),
        ([('x', '1'), ('z', b'2')], {'x': 1, 'z': 2}),
        ((), {}),
        ((('x', '1'),), {'x': 1}),
        (Foobar(), Err('Input should be a valid dictionary [type=dict_type,')),
    ],
    ids=repr,
)
def test_dict_cases(input_value, expected):
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'str'}, 'values_schema': {'type': 'int'}})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        assert v.validate_python(input_value) == expected


def test_dict_value_error(py_and_json: PyAndJson):
    v = py_and_json({'type': 'dict', 'values_schema': {'type': 'int'}})
    assert v.validate_test({'a': 2, 'b': '4'}) == {'a': 2, 'b': 4}
    with pytest.raises(ValidationError, match='Input should be a valid integer') as exc_info:
        v.validate_test({'a': 2, 'b': 'wrong'})
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'int_parsing',
            'loc': ('b',),
            'msg': 'Input should be a valid integer, unable to parse string as an integer',
            'input': 'wrong',
        }
    ]


def test_dict_error_key_int():
    v = SchemaValidator({'type': 'dict', 'values_schema': {'type': 'int'}})
    with pytest.raises(ValidationError, match='Input should be a valid integer') as exc_info:
        v.validate_python({1: 2, 3: 'wrong', -4: 'wrong2'})
    # insert_assert(exc_info.value.errors(include_url=False))
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'int_parsing',
            'loc': (3,),
            'msg': 'Input should be a valid integer, unable to parse string as an integer',
            'input': 'wrong',
        },
        {
            'type': 'int_parsing',
            'loc': (-4,),
            'msg': 'Input should be a valid integer, unable to parse string as an integer',
            'input': 'wrong2',
        },
    ]


def test_dict_error_key_other():
    v = SchemaValidator({'type': 'dict', 'values_schema': {'type': 'int'}})
    with pytest.raises(ValidationError, match='Input should be a valid integer') as exc_info:
        v.validate_python({1: 2, (1, 2): 'wrong'})
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'int_parsing',
            'loc': ('(1, 2)',),
            'msg': 'Input should be a valid integer, unable to parse string as an integer',
            'input': 'wrong',
        }
    ]


def test_dict_any_value():
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'str'}})
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'str'}})
    assert v.validate_python({'1': 1, '2': 'a', '3': None}) == {'1': 1, '2': 'a', '3': None}


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
    assert v.validate_python(MyMapping({'1': 2, 3: '4'})) == {1: 2, 3: 4}
    v = SchemaValidator(
        {'type': 'dict', 'strict': True, 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}}
    )
    with pytest.raises(ValidationError, match='Input should be a valid dictionary'):
        v.validate_python(MyMapping({'1': 2, 3: '4'}))


def test_key_error():
    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    assert v.validate_python({'1': True}) == {1: 1}
    with pytest.raises(ValidationError, match=re.escape('x.[key]\n  Input should be a valid integer')) as exc_info:
        v.validate_python({'x': 1})
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'int_parsing',
            'loc': ('x', '[key]'),
            'msg': 'Input should be a valid integer, unable to parse string as an integer',
            'input': 'x',
        }
    ]


def test_mapping_error():
    class BadMapping(Mapping):
        def __getitem__(self, key):
            raise None

        def __iter__(self):
            raise RuntimeError('intentional error')

        def __len__(self):
            return 1

    v = SchemaValidator({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python(BadMapping())

    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'dict_type',
            'loc': (),
            'msg': 'Input should be a valid dictionary',
            'input': HasRepr(IsStr(regex='.+BadMapping object at.+')),
        }
    ]


@pytest.mark.parametrize(
    'mapping_items,errors',
    [
        (
            [(1,)],
            [
                {
                    'type': 'iteration_error',
                    'loc': (),
                    'msg': 'Error iterating over object, error: ValueError: expected tuple of length 2, but got tuple of length 1',  # noqa: E501
                    'input': HasRepr(IsStr(regex='.+BadMapping object at.+')),
                    'ctx': {'error': 'ValueError: expected tuple of length 2, but got tuple of length 1'},
                }
            ],
        ),
        (
            ['foobar'],
            [
                {
                    'type': 'iteration_error',
                    'loc': (),
                    'msg': "Error iterating over object, error: TypeError: 'str' object cannot be converted to 'PyTuple'",  # noqa: E501
                    'input': HasRepr(IsStr(regex='.+BadMapping object at.+')),
                    'ctx': {'error': "TypeError: 'str' object cannot be converted to 'PyTuple'"},
                }
            ],
        ),
        (
            [(1, 2, 3)],
            [
                {
                    'type': 'iteration_error',
                    'loc': (),
                    'msg': 'Error iterating over object, error: ValueError: expected tuple of length 2, but got tuple of length 3',  # noqa: E501
                    'input': HasRepr(IsStr(regex='.+BadMapping object at.+')),
                    'ctx': {'error': 'ValueError: expected tuple of length 2, but got tuple of length 3'},
                }
            ],
        ),
        (
            'not list',
            [
                {
                    'type': 'iteration_error',
                    'loc': (),
                    'msg': "Error iterating over object, error: TypeError: 'str' object cannot be converted to 'PyTuple'",  # noqa: E501
                    'input': HasRepr(IsStr(regex='.+BadMapping object at.+')),
                    'ctx': {'error': "TypeError: 'str' object cannot be converted to 'PyTuple'"},
                }
            ],
        ),
    ],
)
def test_mapping_error_yield_1(mapping_items: List[Any], errors: List[Any]):
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
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python(BadMapping())

    assert exc_info.value.errors(include_url=False) == errors


@pytest.mark.parametrize(
    'kwargs,input_value,expected',
    [
        ({}, {'1': 1, '2': 2}, {'1': 1, '2': 2}),
        (
            {'min_length': 3},
            {'1': 1, '2': 2, '3': 3.0, '4': [1, 2, 3, 4]},
            {'1': 1, '2': 2, '3': 3.0, '4': [1, 2, 3, 4]},
        ),
        (
            {'min_length': 3},
            {1: '2', 3: '4'},
            Err('Dictionary should have at least 3 items after validation, not 2 [type=too_short,'),
        ),
        ({'max_length': 4}, {'1': 1, '2': 2, '3': 3.0}, {'1': 1, '2': 2, '3': 3.0}),
        (
            {'max_length': 3},
            {'1': 1, '2': 2, '3': 3.0, '4': [1, 2, 3, 4]},
            Err('Dictionary should have at most 3 items after validation, not >= 4 [type=too_long,'),
        ),
    ],
)
def test_dict_length_constraints(kwargs: Dict[str, Any], input_value, expected):
    v = SchemaValidator({'type': 'dict', **kwargs})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        assert v.validate_python(input_value) == expected
