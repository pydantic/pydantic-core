import dataclasses
import re

import pytest
from dirty_equals import IsListOrTuple

from pydantic_core import SchemaValidator, ValidationError, core_schema

from ..conftest import Err, PyAndJson


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (('hello', True), ({'a': 'hello', 'b': True}, None)),
        (['hello', True], ({'a': 'hello', 'b': True}, None)),
        ({'a': 'hello', 'b': True}, ({'a': 'hello', 'b': True}, None)),
        ({'a': 'hello', 'b': 'true'}, ({'a': 'hello', 'b': True}, None)),
        ({'__args__': ('hello', True), '__kwargs__': {}}, ({'a': 'hello', 'b': True}, None)),
        ({'__args__': (), '__kwargs__': {'a': 'hello', 'b': True}}, ({'a': 'hello', 'b': True}, None)),
        (
            {'__args__': ('hello',), '__kwargs__': {'a': 'hello', 'b': True}},
            Err(
                'Got multiple values for argument',
                errors=[
                    {
                        'type': 'multiple_argument_values',
                        'loc': ('a',),
                        'msg': 'Got multiple values for argument',
                        'input': 'hello',
                    }
                ],
            ),
        ),
        (
            {'a': 'hello'},
            Err(
                'Missing required keyword argument',
                errors=[
                    {
                        'type': 'missing_keyword_argument',
                        'loc': ('b',),
                        'msg': 'Missing required keyword argument',
                        'input': {'a': 'hello'},
                    }
                ],
            ),
        ),
    ],
)
def test_dataclass_args(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), positional=True),
        core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), positional=True),
    )
    v = py_and_json(schema)
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)

        # debug(exc_info.value.errors())
        if expected.errors is not None:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (('hello', True), ({'a': 'hello'}, (True,))),
        (['hello', True], ({'a': 'hello'}, (True,))),
        (('hello', 'true'), ({'a': 'hello'}, (True,))),
        ({'__args__': ('hello', True), '__kwargs__': {}}, ({'a': 'hello'}, (True,))),
        ({'__args__': (), '__kwargs__': {'a': 'hello', 'b': True}}, ({'a': 'hello'}, (True,))),
        (
            {'__args__': ('hello',), '__kwargs__': {'a': 'hello', 'b': True}},
            Err(
                'Got multiple values for argument',
                errors=[
                    {
                        'type': 'multiple_argument_values',
                        'loc': ('a',),
                        'msg': 'Got multiple values for argument',
                        'input': 'hello',
                    }
                ],
            ),
        ),
        (
            {'a': 'hello'},
            Err(
                'Missing required keyword argument',
                errors=[
                    {
                        'type': 'missing_keyword_argument',
                        'loc': ('b',),
                        'msg': 'Missing required keyword argument',
                        'input': {'a': 'hello'},
                    }
                ],
            ),
        ),
        (
            {'a': 'hello', 'b': 'wrong'},
            Err(
                'Input should be a valid boolean, unable to interpret input',
                errors=[
                    {
                        'type': 'bool_parsing',
                        'loc': ('b',),
                        'msg': 'Input should be a valid boolean, unable to interpret input',
                        'input': 'wrong',
                    }
                ],
            ),
        ),
    ],
)
def test_dataclass_args_init_only(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), positional=True),
        core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), positional=True, init_only=True),
        collect_init_only=True,
    )
    v = py_and_json(schema)

    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)

        # debug(exc_info.value.errors())
        if expected.errors is not None:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'a': 'hello'}, ({'a': 'hello'}, ())),
        ({'__args__': (), '__kwargs__': {'a': 'hello'}}, ({'a': 'hello'}, ())),
        (
            ('hello',),
            Err(
                '2 validation errors for dataclass-args',
                errors=[
                    {
                        'type': 'missing_keyword_argument',
                        'loc': ('a',),
                        'msg': 'Missing required keyword argument',
                        'input': IsListOrTuple('hello'),
                    },
                    {
                        'type': 'unexpected_positional_argument',
                        'loc': (0,),
                        'msg': 'Unexpected positional argument',
                        'input': 'hello',
                    },
                ],
            ),
        ),
    ],
)
def test_dataclass_args_init_only_no_fields(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema()), collect_init_only=True
    )
    v = py_and_json(schema)

    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)

        # debug(exc_info.value.errors())
        if expected.errors is not None:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected


def test_aliases(py_and_json: PyAndJson):
    schema = core_schema.dataclass_args_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), validation_alias='Apple'),
        core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), validation_alias=['Banana', 1]),
        core_schema.dataclass_field(
            name='c', schema=core_schema.int_schema(), validation_alias=['Carrot', 'v'], init_only=True
        ),
        collect_init_only=True,
    )
    v = py_and_json(schema)
    assert v.validate_test({'Apple': 'a', 'Banana': ['x', 'false'], 'Carrot': {'v': '42'}}) == (
        {'a': 'a', 'b': False},
        (42,),
    )


def test_dataclass():
    @dataclasses.dataclass
    class Foo:
        a: str
        b: bool

    schema = core_schema.dataclass_schema(
        Foo,
        core_schema.dataclass_args_schema(
            core_schema.dataclass_field(name='a', schema=core_schema.str_schema()),
            core_schema.dataclass_field(name='b', schema=core_schema.bool_schema()),
        ),
    )

    v = SchemaValidator(schema)
    foo = v.validate_python({'a': 'hello', 'b': True})
    assert dataclasses.is_dataclass(foo)
    assert foo.a == 'hello'
    assert foo.b is True

    assert dataclasses.asdict(v.validate_python(Foo(a='hello', b=True))) == {'a': 'hello', 'b': True}

    with pytest.raises(ValidationError, match='Input should be an instance of Foo') as exc_info:
        v.validate_python({'a': 'hello', 'b': True}, strict=True)

    # insert_assert(exc_info.value.errors())
    assert exc_info.value.errors() == [
        {
            'type': 'model_class_type',
            'loc': (),
            'msg': 'Input should be an instance of Foo',
            'input': {'a': 'hello', 'b': True},
            'ctx': {'class_name': 'Foo'},
        }
    ]


def test_dataclass_post_init():
    @dataclasses.dataclass
    class Foo:
        a: str
        b: bool

        def __post_init__(self):
            self.a = self.a.upper()

    schema = core_schema.dataclass_schema(
        Foo,
        core_schema.dataclass_args_schema(
            core_schema.dataclass_field(name='a', schema=core_schema.str_schema()),
            core_schema.dataclass_field(name='b', schema=core_schema.bool_schema()),
        ),
        post_init=True,
    )

    v = SchemaValidator(schema)
    foo = v.validate_python({'a': 'hello', 'b': True})
    assert foo.a == 'HELLO'
    assert foo.b is True


def test_dataclass_post_init_args():
    c_value = None

    @dataclasses.dataclass
    class Foo:
        a: str
        b: bool
        c: dataclasses.InitVar[int]

        def __post_init__(self, c: int):
            nonlocal c_value
            c_value = c

    schema = core_schema.dataclass_schema(
        Foo,
        core_schema.dataclass_args_schema(
            core_schema.dataclass_field(name='a', schema=core_schema.str_schema()),
            core_schema.dataclass_field(name='b', schema=core_schema.bool_schema()),
            core_schema.dataclass_field(name='c', schema=core_schema.int_schema(), init_only=True),
            collect_init_only=True,
        ),
        post_init=True,
    )

    v = SchemaValidator(schema)
    foo = v.validate_python({'a': b'hello', 'b': 'true', 'c': '42'})
    assert foo.a == 'hello'
    assert foo.b is True
    assert not hasattr(foo, 'c')
    assert c_value == 42


def test_dataclass_post_init_args_multiple():
    dc_args = None

    @dataclasses.dataclass
    class Foo:
        a: str
        b: dataclasses.InitVar[bool]
        c: dataclasses.InitVar[int]

        def __post_init__(self, *args):
            nonlocal dc_args
            dc_args = args

    schema = core_schema.dataclass_schema(
        Foo,
        core_schema.dataclass_args_schema(
            core_schema.dataclass_field(name='a', schema=core_schema.str_schema()),
            core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), init_only=True),
            core_schema.dataclass_field(name='c', schema=core_schema.int_schema(), init_only=True),
            collect_init_only=True,
        ),
        post_init=True,
    )

    v = SchemaValidator(schema)
    foo = v.validate_python({'a': b'hello', 'b': 'true', 'c': '42'})
    assert dataclasses.asdict(foo) == {'a': 'hello'}
    assert dc_args == (True, 42)
