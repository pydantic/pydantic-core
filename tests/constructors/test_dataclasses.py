import dataclasses
import re
from typing import Any, Dict, Union

import pytest
from dirty_equals import IsListOrTuple, IsStr

from pydantic_core import ArgsKwargs, SchemaValidator, ValidationError, core_schema

from ..conftest import Err, PyAndJson


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (ArgsKwargs(('hello', True)), ({'a': 'hello', 'b': True}, None)),
        ({'a': 'hello', 'b': 'incorrect'}, ({'a': 'hello', 'b': 'incorrect'}, None)),
        ({'a': 'hello', 'b': 'true'}, ({'a': 'hello', 'b': 'true'}, None)),
        (ArgsKwargs(('hello', True)), ({'a': 'hello', 'b': True}, None)),
        (ArgsKwargs((), {'a': 'hello', 'b': True}), ({'a': 'hello', 'b': True}, None)),
        (
            ArgsKwargs(('hello',), {'a': 'hello', 'b': True}),
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
        ({'a': 'hello'}, ({'a': 'hello'}, None)),
    ],
)
def test_dataclass_args(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        'MyDataclass',
        [
            core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), kw_only=False),
            core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), kw_only=False),
        ],
    )
    v = py_and_json(schema)
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.construct_test(input_value)

        # debug(exc_info.value.errors(include_url=False))
        if expected.errors is not None:
            assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        assert v.construct_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (ArgsKwargs((10, True)), ({'a': 10}, (True,))),
        (ArgsKwargs(('hello', 'true')), ({'a': 'hello'}, ('true',))),
        (ArgsKwargs(('hello', True)), ({'a': 'hello'}, (True,))),
        (ArgsKwargs((), {'a': 'hello', 'b': True}), ({'a': 'hello'}, (True,))),
        (
            ArgsKwargs(('hello',), {'a': 'hello', 'b': True}),
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
        ({'a': 'hello'}, ({'a': 'hello'}, ())),
        ({'a': 'hello', 'b': 'wrong'}, ({'a': 'hello'}, ('wrong',))),
    ],
)
def test_dataclass_args_init_only(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        'MyDataclass',
        [
            core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), kw_only=False),
            core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), kw_only=False, init_only=True),
        ],
        collect_init_only=True,
    )
    v = py_and_json(schema)

    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.construct_test(input_value)

        # debug(exc_info.value.errors(include_url=False))
        if expected.errors is not None:
            assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        assert v.construct_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'a': 'hello'}, ({'a': 'hello'}, ())),
        (ArgsKwargs((), {'a': 'hello'}), ({'a': 'hello'}, ())),
        (
            ('hello',),
            Err(
                'Input should be (an object|a dictionary or an instance of MyDataclass)',
                errors=[
                    {
                        'type': 'dataclass_type',
                        'loc': (),
                        'msg': IsStr(regex='Input should be (an object|a dictionary or an instance of MyDataclass)'),
                        'input': IsListOrTuple('hello'),
                        'ctx': {'class_name': 'MyDataclass'},
                    }
                ],
            ),
        ),
    ],
)
def test_dataclass_args_init_only_no_fields(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        'MyDataclass', [core_schema.dataclass_field(name='a', schema=core_schema.str_schema())], collect_init_only=True
    )
    v = py_and_json(schema)

    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message) as exc_info:
            v.construct_test(input_value)

        # debug(exc_info.value.errors(include_url=False))
        if expected.errors is not None:
            assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        assert v.construct_test(input_value) == expected


def test_aliases(py_and_json: PyAndJson):
    schema = core_schema.dataclass_args_schema(
        'MyDataclass',
        [
            core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), validation_alias='Apple'),
            core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), validation_alias=['Banana', 1]),
            core_schema.dataclass_field(
                name='c', schema=core_schema.int_schema(), validation_alias=['Carrot', 'v'], init_only=True
            ),
        ],
        collect_init_only=True,
    )
    v = py_and_json(schema)
    assert v.construct_test({'Apple': 10, 'Banana': ['x', 'false'], 'Carrot': {'v': '42'}}) == (
        {'a': 10, 'b': 'false'},
        ('42',),
    )


@dataclasses.dataclass
class FooDataclass:
    a: str
    b: bool


def test_dataclass_construct():
    schema = core_schema.dataclass_schema(
        FooDataclass,
        core_schema.dataclass_args_schema(
            'FooDataclass',
            [
                core_schema.dataclass_field(name='a', schema=core_schema.str_schema()),
                core_schema.dataclass_field(name='b', schema=core_schema.bool_schema()),
            ],
        ),
        ['a', 'b'],
    )

    v = SchemaValidator(schema)
    foo = v.construct_python({'a': 'hello', 'b': True})
    assert dataclasses.is_dataclass(foo)
    assert foo.a == 'hello'
    assert foo.b is True

    assert dataclasses.asdict(v.construct_python(FooDataclass(a='hello', b=True))) == {'a': 'hello', 'b': True}

    incorrect_foo = v.construct_python({'a': {}, 'b': 'incorrect'})
    assert dataclasses.is_dataclass(incorrect_foo)
    assert incorrect_foo.a == {}
    assert incorrect_foo.b == 'incorrect'


@dataclasses.dataclass
class FooDataclassSame(FooDataclass):
    pass


@dataclasses.dataclass
class FooDataclassMore(FooDataclass):
    c: str


@dataclasses.dataclass
class DuplicateDifferent:
    a: str
    b: bool


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'revalidate_instances,input_value,expected',
    [
        ('always', {'a': 'hello', 'b': True}, {'a': 'hello', 'b': True}),
        ('always', FooDataclass(a='hello', b=True), {'a': 'hello', 'b': True}),
        ('always', FooDataclassSame(a='hello', b=True), {'a': 'hello', 'b': True}),
        ('always', FooDataclassMore(a='hello', b=True, c='more'), {'a': 'hello', 'b': True, 'c': 'more'}),
        ('always', FooDataclassSame(a='hello', b='wrong'), {'a': 'hello', 'b': 'wrong'}),
        # Construct should pass as-is with different class
        ('always', DuplicateDifferent(a='hello', b=True), {'a': 'hello', 'b': True}),
        ('subclass-instances', {'a': 'hello', 'b': True}, {'a': 'hello', 'b': True}),
        ('subclass-instances', FooDataclass(a='hello', b=True), {'a': 'hello', 'b': True}),
        ('subclass-instances', FooDataclass(a=b'hello', b='true'), {'a': b'hello', 'b': 'true'}),
        ('subclass-instances', FooDataclassSame(a='hello', b=True), {'a': 'hello', 'b': True}),
        ('subclass-instances', FooDataclassSame(a=b'hello', b='true'), {'a': b'hello', 'b': 'true'}),
        ('subclass-instances', FooDataclassMore(a='hello', b=True, c='more'), {'a': 'hello', 'b': True, 'c': 'more'}),
        ('subclass-instances', FooDataclassSame(a='hello', b='wrong'), {'a': 'hello', 'b': 'wrong'}),
        # Construct should pass as-is with different class
        ('subclass-instances', DuplicateDifferent(a='hello', b=True), {'a': 'hello', 'b': True}),
        ('never', {'a': 'hello', 'b': True}, {'a': 'hello', 'b': True}),
        ('never', FooDataclass(a='hello', b=True), {'a': 'hello', 'b': True}),
        ('never', FooDataclassSame(a='hello', b=True), {'a': 'hello', 'b': True}),
        ('never', FooDataclassMore(a='hello', b=True, c='more'), {'a': 'hello', 'b': True, 'c': 'more'}),
        ('never', FooDataclassMore(a='hello', b='wrong', c='more'), {'a': 'hello', 'b': 'wrong', 'c': 'more'}),
        # Construct should pass as-is with different class
        ('never', DuplicateDifferent(a='hello', b=True), {'a': 'hello', 'b': True}),
    ],
)
def test_dataclass_revalidate_instances(revalidate_instances, input_value, expected, recursive):
    """
    Revalidation parameter should have no effect on construction, recursive or otherwise.
    """
    schema = core_schema.dataclass_schema(
        FooDataclass,
        core_schema.dataclass_args_schema(
            'FooDataclass',
            [
                core_schema.dataclass_field(name='a', schema=core_schema.str_schema()),
                core_schema.dataclass_field(name='b', schema=core_schema.bool_schema()),
            ],
            extra_behavior='forbid',
        ),
        ['a', 'b'],
        revalidate_instances=revalidate_instances,
    )
    v = SchemaValidator(schema)

    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message) as exc_info:
            print(v.construct_python(input_value, recursive=recursive))

        # debug(exc_info.value.errors(include_url=False))
        if expected.errors is not None:
            assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        dc = v.construct_python(input_value, recursive=recursive)
        assert dataclasses.is_dataclass(dc)
        assert dataclasses.asdict(dc) == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'reconstruct_instances,input_value,expected,expected_type',
    [
        ('always', {'a': 'hello', 'b': True}, {'a': 'hello', 'b': True}, FooDataclass),
        ('always', FooDataclass(a='hello', b=True), {'a': 'hello', 'b': True}, FooDataclass),
        ('always', FooDataclassSame(a='hello', b=True), {'a': 'hello', 'b': True}, FooDataclass),
        # Extra keywords, but gets converted to different dataclass without extra
        ('always', FooDataclassMore(a='hello', b=True, c='more'), {'a': 'hello', 'b': True}, FooDataclass),
        ('always', FooDataclassSame(a='hello', b='wrong'), {'a': 'hello', 'b': 'wrong'}, FooDataclass),
        # Construct should pass as-is with different class
        ('always', DuplicateDifferent(a='hello', b=True), {'a': 'hello', 'b': True}, DuplicateDifferent),
        # reconstruct_instances='subclass-instances'
        ('subclass-instances', {'a': 'hello', 'b': True}, {'a': 'hello', 'b': True}, FooDataclass),
        ('subclass-instances', FooDataclass(a='hello', b=True), {'a': 'hello', 'b': True}, FooDataclass),
        ('subclass-instances', FooDataclass(a=b'hello', b='true'), {'a': b'hello', 'b': 'true'}, FooDataclass),
        ('subclass-instances', FooDataclassSame(a='hello', b=True), {'a': 'hello', 'b': True}, FooDataclass),
        ('subclass-instances', FooDataclassSame(a=b'hello', b='true'), {'a': b'hello', 'b': 'true'}, FooDataclass),
        # Extra keywords, but gets converted to different dataclass without extra
        ('subclass-instances', FooDataclassMore(a='hello', b=True, c='more'), {'a': 'hello', 'b': True}, FooDataclass),
        ('subclass-instances', FooDataclassSame(a='hello', b='wrong'), {'a': 'hello', 'b': 'wrong'}, FooDataclass),
        # Construct should pass as-is with different class
        ('subclass-instances', DuplicateDifferent(a='hello', b=True), {'a': 'hello', 'b': True}, DuplicateDifferent),
        # reconstruct_instances='never'
        ('never', {'a': 'hello', 'b': True}, {'a': 'hello', 'b': True}, FooDataclass),
        ('never', FooDataclass(a='hello', b=True), {'a': 'hello', 'b': True}, FooDataclass),
        ('never', FooDataclassSame(a='hello', b=True), {'a': 'hello', 'b': True}, FooDataclass),
        # Because reconstruct_instances is 'never', the input instances stay as-is
        (
            'never',
            FooDataclassMore(a='hello', b=True, c='more'),
            {'a': 'hello', 'b': True, 'c': 'more'},
            FooDataclassMore,
        ),
        (
            'never',
            FooDataclassMore(a='hello', b='wrong', c='more'),
            {'a': 'hello', 'b': 'wrong', 'c': 'more'},
            FooDataclassMore,
        ),
        # Construct should pass as-is with different class
        ('never', DuplicateDifferent(a='hello', b=True), {'a': 'hello', 'b': True}, DuplicateDifferent),
    ],
)
def test_dataclass_reconstruct_instances(reconstruct_instances, input_value, expected, expected_type, recursive):
    schema = core_schema.dataclass_schema(
        FooDataclass,
        core_schema.dataclass_args_schema(
            'FooDataclass',
            [
                core_schema.dataclass_field(name='a', schema=core_schema.str_schema()),
                core_schema.dataclass_field(name='b', schema=core_schema.bool_schema()),
            ],
            extra_behavior='forbid',
        ),
        ['a', 'b'],
        reconstruct_instances=reconstruct_instances,
    )
    v = SchemaValidator(schema)

    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=expected.message) as exc_info:
            print(v.construct_python(input_value, recursive=recursive))

        # debug(exc_info.value.errors(include_url=False))
        if expected.errors is not None:
            assert exc_info.value.errors(include_url=False) == expected.errors
    else:
        dc = v.construct_python(input_value, recursive=recursive)
        print(input_value)
        assert dataclasses.is_dataclass(dc)
        assert dataclasses.asdict(dc) == expected
        assert isinstance(dc, expected_type)


def test_dataclass_post_init():
    """__post_init__ works when calling construct:"""

    @dataclasses.dataclass
    class Foo:
        a: str
        b: bool

        def __post_init__(self):
            self.a = self.a.upper()

    schema = core_schema.dataclass_schema(
        Foo,
        core_schema.dataclass_args_schema(
            'Foo',
            [
                core_schema.dataclass_field(name='a', schema=core_schema.str_schema()),
                core_schema.dataclass_field(name='b', schema=core_schema.bool_schema()),
            ],
        ),
        ['a', 'b'],
        post_init=True,
    )

    v = SchemaValidator(schema)
    foo = v.construct_python({'a': 'hello', 'b': True})
    assert foo.a == 'HELLO'
    assert foo.b is True


@pytest.mark.parametrize(
    'config,schema_extra_behavior_kw',
    [
        (core_schema.CoreConfig(extra_fields_behavior='ignore'), {}),
        (core_schema.CoreConfig(extra_fields_behavior='ignore'), {'extra_behavior': None}),
        (core_schema.CoreConfig(), {'extra_behavior': 'ignore'}),
        (None, {'extra_behavior': 'ignore'}),
        (core_schema.CoreConfig(extra_fields_behavior='allow'), {'extra_behavior': 'ignore'}),
    ],
)
def test_extra_behavior_ignore(config: Union[core_schema.CoreConfig, None], schema_extra_behavior_kw: Dict[str, Any]):
    """Ignore has no effect when constructing"""

    @dataclasses.dataclass
    class MyModel:
        f: str

    v = SchemaValidator(
        core_schema.dataclass_schema(
            MyModel,
            core_schema.dataclass_args_schema(
                'MyModel', [core_schema.dataclass_field('f', core_schema.str_schema())], **schema_extra_behavior_kw
            ),
            ['f'],
        ),
        config=config,
    )

    m: MyModel = v.construct_python({'f': 10, 'extra_field': 123})
    assert m.f == 10
    assert hasattr(m, 'extra_field')
    assert m.extra_field == 123

    v.validate_assignment(m, 'f', 'y')
    assert m.f == 'y'

    with pytest.raises(ValidationError) as exc_info:
        v.validate_assignment(m, 'not_f', 'xyz')

    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'no_such_attribute',
            'loc': ('not_f',),
            'msg': "Object has no attribute 'not_f'",
            'input': 'xyz',
            'ctx': {'attribute': 'not_f'},
        }
    ]
    assert not hasattr(m, 'not_f')


@pytest.mark.parametrize(
    'config,schema_extra_behavior_kw',
    [
        (core_schema.CoreConfig(extra_fields_behavior='forbid'), {}),
        (core_schema.CoreConfig(extra_fields_behavior='forbid'), {'extra_behavior': None}),
        (core_schema.CoreConfig(), {'extra_behavior': 'forbid'}),
        (None, {'extra_behavior': 'forbid'}),
        (core_schema.CoreConfig(extra_fields_behavior='ignore'), {'extra_behavior': 'forbid'}),
    ],
)
def test_extra_behavior_forbid(config: Union[core_schema.CoreConfig, None], schema_extra_behavior_kw: Dict[str, Any]):
    @dataclasses.dataclass
    class MyModel:
        f: str

    v = SchemaValidator(
        core_schema.dataclass_schema(
            MyModel,
            core_schema.dataclass_args_schema(
                'MyModel', [core_schema.dataclass_field('f', core_schema.str_schema())], **schema_extra_behavior_kw
            ),
            ['f'],
        ),
        config=config,
    )

    m: MyModel = v.construct_python({'f': 10})
    assert m.f == 10

    v.validate_assignment(m, 'f', 'y')
    assert m.f == 'y'

    with pytest.raises(ValidationError) as exc_info:
        v.validate_assignment(m, 'not_f', 'xyz')
    assert exc_info.value.errors(include_url=False) == [
        {
            'type': 'no_such_attribute',
            'loc': ('not_f',),
            'msg': "Object has no attribute 'not_f'",
            'input': 'xyz',
            'ctx': {'attribute': 'not_f'},
        }
    ]
    assert not hasattr(m, 'not_f')


@pytest.mark.parametrize(
    'config,schema_extra_behavior_kw',
    [
        (core_schema.CoreConfig(extra_fields_behavior='allow'), {}),
        (core_schema.CoreConfig(extra_fields_behavior='allow'), {'extra_behavior': None}),
        (core_schema.CoreConfig(), {'extra_behavior': 'allow'}),
        (None, {'extra_behavior': 'allow'}),
        (core_schema.CoreConfig(extra_fields_behavior='forbid'), {'extra_behavior': 'allow'}),
    ],
)
def test_extra_behavior_allow(config: Union[core_schema.CoreConfig, None], schema_extra_behavior_kw: Dict[str, Any]):
    @dataclasses.dataclass
    class MyModel:
        f: str

    v = SchemaValidator(
        core_schema.dataclass_schema(
            MyModel,
            core_schema.dataclass_args_schema(
                'MyModel', [core_schema.dataclass_field('f', core_schema.str_schema())], **schema_extra_behavior_kw
            ),
            ['f'],
            config=config,
        )
    )

    m: MyModel = v.construct_python({'f': 10, 'extra_field': '123'})
    assert m.f == 10
    assert getattr(m, 'extra_field') == '123'

    v.validate_assignment(m, 'f', 'y')
    assert m.f == 'y'

    v.validate_assignment(m, 'not_f', '123')
    assert getattr(m, 'not_f') == '123'


def test_dataclass_construct_recursive():
    @dataclasses.dataclass
    class Child:
        f: str

    class Parent:
        child: Child

    @dataclasses.dataclass
    class Grandparent:
        parent: Parent

    c = core_schema.dataclass_schema(
        Child,
        core_schema.dataclass_args_schema('Child', [core_schema.dataclass_field('f', core_schema.str_schema())]),
        ['f'],
    )

    p = core_schema.dataclass_schema(
        Parent, core_schema.dataclass_args_schema('Parent', [core_schema.dataclass_field('child', c)]), ['child']
    )

    g = core_schema.dataclass_schema(
        Grandparent,
        core_schema.dataclass_args_schema('Grandparent', [core_schema.dataclass_field('parent', p)]),
        ['parent'],
    )

    v = SchemaValidator(g)

    m: Grandparent = v.construct_python({'parent': {'child': {'f': 'something'}}}, recursive=True)
    assert isinstance(m, Grandparent)
    assert isinstance(m.parent, Parent)
    assert isinstance(m.parent.child, Child)
    assert m.parent.child.f == 'something'


def test_dataclass_defaults():
    pass  # TODO
