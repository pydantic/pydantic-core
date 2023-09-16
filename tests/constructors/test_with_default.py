import sys
from collections import deque
from typing import Any, Callable, Dict, List, Union, cast

import pytest

from pydantic_core import PydanticUndefined, PydanticUseDefault, SchemaValidator, Some, core_schema

from ..conftest import PyAndJson


def test_typed_dict_default():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'y': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'default', 'schema': {'type': 'str'}, 'default': '[default]'},
                },
            },
        }
    )
    assert v.construct_python({'x': 'x', 'y': 'y'}) == {'x': 'x', 'y': 'y'}
    assert v.construct_python({'x': 'x'}, recursive=False) == {'x': 'x'}
    assert v.construct_python({'x': 'x'}, recursive=True) == {'x': 'x', 'y': '[default]'}


def test_typed_dict_omit():
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'y': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'default', 'schema': {'type': 'str'}, 'on_error': 'omit'},
                    'required': False,
                },
            },
        }
    )
    assert v.construct_python({'x': 'x', 'y': 'y'}) == {'x': 'x', 'y': 'y'}
    assert v.construct_python({'x': 'x'}) == {'x': 'x'}
    assert v.construct_python({'x': 'x', 'y': 42}) == {'x': 'x', 'y': 42}


# TODO
# def test_arguments():
#     v = SchemaValidator(
#         {
#             'type': 'arguments',
#             'arguments_schema': [
#                 {
#                     'name': 'a',
#                     'mode': 'positional_or_keyword',
#                     'schema': {'type': 'default', 'schema': {'type': 'int'}, 'default_factory': lambda: 1},
#                 }
#             ],
#         }
#     )
#     assert v.construct_python({'a': 2}) == ((), {'a': 2})
#     assert v.construct_python(ArgsKwargs((2,))) == ((2,), {})
#     assert v.construct_python(ArgsKwargs((2,), {})) == ((2,), {})
#     assert v.construct_python(()) == ((), {'a': 1})


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected', [([1, 2, 3], [1, 2, 3]), ([1, '2', 3], [1, '2', 3]), ([1, 'wrong', 3], [1, 'wrong', 3])]
)
def test_list_json(py_and_json: PyAndJson, input_value, expected, recursive):
    v = py_and_json(
        {'type': 'list', 'items_schema': {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'}}
    )
    assert v.construct_test(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ([1, '2', 3], [1, '2', 3]),
        ([1, 'wrong', 3], [1, 'wrong', 3]),
        ((1, '2', 3), (1, '2', 3)),
        ((1, 'wrong', 3), (1, 'wrong', 3)),
        (deque([1, '2', 3]), deque([1, '2', 3])),
        (deque([1, 'wrong', 3]), deque([1, 'wrong', 3])),
    ],
)
def test_list(input_value, expected, recursive):
    v = SchemaValidator(
        {'type': 'list', 'items_schema': {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'}}
    )
    assert v.construct_python(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [False, True])
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({1, '2', 3}, {1, '2', 3}),
        ([1, '2', 3], [1, '2', 3]),
        ([1, 'wrong', 3], [1, 'wrong', 3]),
        (deque([1, '2', 3]), deque([1, '2', 3])),
        (deque([1, 'wrong', 3]), deque([1, 'wrong', 3])),
    ],
)
def test_set(input_value, expected, recursive):
    v = SchemaValidator(
        {'type': 'set', 'items_schema': {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'}}
    )
    assert v.construct_python(input_value, recursive=recursive) == expected


@pytest.mark.parametrize('recursive', [False, True])
def test_dict_values(py_and_json: PyAndJson, recursive: bool):
    v = py_and_json(
        {
            'type': 'dict',
            'keys_schema': {'type': 'str'},
            'values_schema': {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'},
        }
    )
    assert v.construct_test({'a': 1, 'b': '2'}, recursive=recursive) == {'a': 1, 'b': '2'}
    assert v.construct_test({'a': 1, 'b': 'wrong'}, recursive=recursive) == {'a': 1, 'b': 'wrong'}
    assert v.construct_test({'a': 1, 'b': 'wrong', 'c': '3'}, recursive=recursive) == {'a': 1, 'b': 'wrong', 'c': '3'}


@pytest.mark.parametrize('recursive', [False, True])
def test_dict_keys(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'dict',
            'keys_schema': {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'},
            'values_schema': {'type': 'str'},
        }
    )
    assert v.construct_python({1: 'a', '2': 'b'}, recursive=recursive) == {1: 'a', '2': 'b'}
    assert v.construct_python({1: 'a', 'wrong': 'b'}, recursive=recursive) == {1: 'a', 'wrong': 'b'}
    assert v.construct_python({1: 'a', 'wrong': 'b', 3: 'c'}, recursive=recursive) == {1: 'a', 'wrong': 'b', 3: 'c'}


@pytest.mark.parametrize('recursive', [False, True])
def test_tuple_variable(recursive: bool):
    v = SchemaValidator(
        {'type': 'tuple-variable', 'items_schema': {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'}}
    )
    assert v.construct_python((1, 2, 3), recursive=recursive) == (1, 2, 3)
    assert v.construct_python([1, '2', 3], recursive=recursive) == [1, '2', 3]
    assert v.construct_python([1, 'wrong', 3], recursive=recursive) == [1, 'wrong', 3]


@pytest.mark.parametrize('recursive', [False, True])
def test_tuple_positional(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'tuple-positional',
            'items_schema': [{'type': 'int'}, {'type': 'default', 'schema': {'type': 'int'}, 'default': 42}],
        }
    )
    assert v.construct_python((1, '2'), recursive=recursive) == (1, '2')
    assert v.construct_python([1, '2'], recursive=recursive) == [1, '2']
    assert v.construct_json('[1, "2"]', recursive=recursive) == [1, '2']
    assert v.construct_python((1,), recursive=False) == (1,)  # Default not called
    assert v.construct_python((1,), recursive=True) == (1, 42)  # Default called


@pytest.mark.parametrize('recursive', [False, True])
def test_tuple_positional_omit(recursive: bool):
    v = SchemaValidator(
        {
            'type': 'tuple-positional',
            'items_schema': [{'type': 'int'}, {'type': 'int'}],
            'extras_schema': {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'},
        }
    )
    assert v.construct_python((1, '2'), recursive=recursive) == (1, '2')
    assert v.construct_python((1, '2', 3, '4'), recursive=recursive) == (1, '2', 3, '4')
    assert v.construct_python((1, '2', 'wrong', '4'), recursive=recursive) == (1, '2', 'wrong', '4')
    assert v.construct_python((1, '2', 3, 'x4'), recursive=recursive) == (1, '2', 3, 'x4')
    assert v.construct_json('[1, "2", 3, "x4"]', recursive=recursive) == [1, '2', 3, 'x4']


def test_on_error_default():
    v = SchemaValidator({'type': 'default', 'schema': {'type': 'int'}, 'default': 2, 'on_error': 'default'})
    assert v.construct_python(42) == 42
    assert v.construct_python('42') == '42'
    # Default is not used because no validation error occurred
    assert v.construct_python('wrong', recursive=True) == 'wrong'


def test_factory_no_runtime_error():
    def broken():
        raise RuntimeError('this is broken')

    v = SchemaValidator(
        {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'default', 'default_factory': broken}
    )
    assert v.construct_python(42) == 42
    assert v.construct_python('42') == '42'
    # Default factory is not used because no validation error occurred
    assert v.construct_python('wrong', recursive=True) == 'wrong'
    # This should call `broken()`
    with pytest.raises(RuntimeError, match='this is broken'):
        v.construct_python(PydanticUndefined, recursive=True)


def test_factory_type_error():
    def broken(x):
        return 7

    v = SchemaValidator(
        {'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'default', 'default_factory': broken}
    )
    assert v.construct_python(42) == 42
    assert v.construct_python('42') == '42'
    # Default factory is not used because no validation error occurred
    assert v.construct_python('wrong', recursive=True) == 'wrong'
    # This should call `broken()`
    with pytest.raises(TypeError, match=r"broken\(\) missing 1 required positional argument: 'x'"):
        v.construct_python(PydanticUndefined, recursive=True)


def test_typed_dict_error():
    """
    If default factory is malformed, treat it as catastrophic; even though we're constructing, raise an error to tell
    the user to fix their function
    """
    v = SchemaValidator(
        {
            'type': 'typed-dict',
            'fields': {
                'x': {'type': 'typed-dict-field', 'schema': {'type': 'str'}},
                'y': {
                    'type': 'typed-dict-field',
                    'schema': {'type': 'default', 'schema': {'type': 'str'}, 'default_factory': lambda y: y * 2},
                },
            },
        }
    )
    assert v.construct_python({'x': 'x', 'y': 'y'}, recursive=True) == {'x': 'x', 'y': 'y'}
    # This is fine because no recursive; matches existing behavior
    assert v.construct_python({'x': 'x'}) == {'x': 'x'}
    # This errors because recursive ensures that the lambda is called
    with pytest.raises(TypeError, match=r"<lambda>\(\) missing 1 required positional argument: 'y'"):
        v.construct_python({'x': 'x'}, recursive=True)


@pytest.mark.parametrize('recursive', [False, True])
def test_on_error_default_not_int(recursive: bool):
    """No effect"""
    v = SchemaValidator({'type': 'default', 'schema': {'type': 'int'}, 'default': [1, 2, 3], 'on_error': 'default'})
    assert v.construct_python(42, recursive=recursive) == 42
    assert v.construct_python('42', recursive=recursive) == '42'
    assert v.construct_python('wrong', recursive=recursive) == 'wrong'


@pytest.mark.parametrize('recursive', [False, True])
def test_on_error_default_factory(recursive: bool):
    """No effect"""
    v = SchemaValidator(
        {'type': 'default', 'schema': {'type': 'int'}, 'default_factory': lambda: 17, 'on_error': 'default'}
    )
    assert v.construct_python(42, recursive=recursive) == 42
    assert v.construct_python('42', recursive=recursive) == '42'
    assert v.construct_python('wrong', recursive=recursive) == 'wrong'


@pytest.mark.parametrize('recursive', [False, True])
def test_on_error_omit(recursive: bool):
    v = SchemaValidator({'type': 'default', 'schema': {'type': 'int'}, 'on_error': 'omit'})
    assert v.construct_python(42, recursive=recursive) == 42
    # value constructs fine, so no reason to omit the value
    assert v.construct_python('wrong', recursive=recursive) == 'wrong'


def test_model_class():
    class MyModel:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        field_a: str
        field_b: int

    v = SchemaValidator(
        {
            'type': 'model',
            'cls': MyModel,
            'schema': {
                'type': 'default',
                'schema': {
                    'type': 'model-fields',
                    'fields': {
                        'field_a': {'type': 'model-field', 'schema': {'type': 'str'}},
                        'field_b': {'type': 'model-field', 'schema': {'type': 'int'}},
                    },
                },
                'default': ({'field_a': '[default-a]', 'field_b': '[default-b]'}, None, set()),
                'on_error': 'default',
            },
        }
    )
    m = v.construct_python({'field_a': 'test', 'field_b': 12}, recursive=True)
    assert isinstance(m, MyModel)
    assert m.field_a == 'test'
    assert m.field_b == 12
    assert m.__pydantic_fields_set__ == {'field_a', 'field_b'}
    # Because no error is raised, all values are passed as if they were valid
    m = v.construct_python({'field_a': 'test', 'field_b': 'wrong'}, recursive=True)
    assert isinstance(m, MyModel)
    assert m.field_a == 'test'
    assert m.field_b == 'wrong'
    assert m.__pydantic_fields_set__ == {'field_a', 'field_b'}


@pytest.mark.parametrize('config_construct_default', [True, False, None])
@pytest.mark.parametrize('schema_construct_default', [True, False, None])
@pytest.mark.parametrize(
    'inner_schema',
    [
        core_schema.no_info_after_validator_function(lambda x: x * 2, core_schema.int_schema()),
        core_schema.no_info_before_validator_function(lambda x: str(int(x) * 2), core_schema.int_schema()),
        core_schema.no_info_wrap_validator_function(lambda x, h: h(str(int(x) * 2)), core_schema.int_schema()),
        core_schema.no_info_wrap_validator_function(lambda x, h: h(x) * 2, core_schema.int_schema()),
    ],
    ids=['after', 'before', 'wrap-before', 'wrap-after'],
)
def test_construct_default(
    config_construct_default: Union[bool, None],
    schema_construct_default: Union[bool, None],
    inner_schema: core_schema.CoreSchema,
):
    """None of the validation functions should be called; default value might get constructed, but otherwise passed as-is"""
    if config_construct_default is not None:
        config = core_schema.CoreConfig(validate_default=config_construct_default)
    else:
        config = None
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            {
                'x': core_schema.typed_dict_field(
                    core_schema.with_default_schema(
                        inner_schema, default='42', validate_default=schema_construct_default
                    )
                )
            },
            config=config,
        )
    )
    assert v.construct_python({'x': '2'}, recursive=False) == {'x': '2'}
    assert v.construct_python({'x': '2'}, recursive=True) == {'x': '2'}
    assert v.construct_python({}, recursive=False) == {}
    assert v.construct_python({}, recursive=True) == {'x': '42'}


def test_validate_default_factory():
    v = SchemaValidator(
        core_schema.tuple_positional_schema(
            [core_schema.with_default_schema(core_schema.int_schema(), default_factory=lambda: '42')]
        ),
        config=dict(validate_default=True),
    )
    assert v.construct_python(('2',), recursive=True) == ('2',)
    assert v.construct_python(('2',), recursive=False) == ('2',)
    # Defaults only work when recursive=True
    assert v.construct_python((), recursive=False) == ()
    assert v.construct_python((), recursive=True) == ('42',)


def test_validate_default_no_error_tuple():
    v = SchemaValidator(
        core_schema.tuple_positional_schema(
            [core_schema.with_default_schema(core_schema.int_schema(), default='wrong', construct_default=True)]
        )
    )
    assert v.construct_python(('2',), recursive=False) == ('2',)
    assert v.construct_python(('2',), recursive=True) == ('2',)
    # Defaults only work when recursive=True
    assert v.construct_python((), recursive=False) == ()
    assert v.construct_python((), recursive=True) == ('wrong',)


def test_validate_default_no_error_typed_dict():
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            {
                'x': core_schema.typed_dict_field(
                    core_schema.with_default_schema(core_schema.int_schema(), default='xx', construct_default=True)
                )
            }
        )
    )
    assert v.construct_python({'x': '2'}, recursive=False) == {'x': '2'}
    assert v.construct_python({'x': '2'}, recursive=True) == {'x': '2'}
    # Defaults only work when recursive=True
    assert v.construct_python({}, recursive=False) == {}
    assert v.construct_python({}, recursive=True) == {'x': 'xx'}


def test_deepcopy_mutable_defaults():
    stored_empty_list = []
    stored_empty_dict = {}

    class Model:
        int_list_with_default: List[int] = stored_empty_list
        str_dict_with_default: Dict[str, str] = stored_empty_dict

    v = SchemaValidator(
        {
            'type': 'model',
            'cls': Model,
            'schema': {
                'type': 'model-fields',
                'fields': {
                    'int_list_with_default': {
                        'type': 'model-field',
                        'schema': {
                            'type': 'default',
                            'schema': {'type': 'list', 'items_schema': {'type': 'int'}},
                            'default': stored_empty_list,
                        },
                    },
                    'str_dict_with_default': {
                        'type': 'model-field',
                        'schema': {
                            'type': 'default',
                            'schema': {
                                'type': 'dict',
                                'keys_schema': {'type': 'str'},
                                'values_schema': {'type': 'str'},
                            },
                            'default': stored_empty_dict,
                        },
                    },
                },
            },
        }
    )

    m1 = v.construct_python({}, recursive=True)

    assert m1.int_list_with_default == []
    assert m1.str_dict_with_default == {}

    assert m1.int_list_with_default is not stored_empty_list
    assert m1.str_dict_with_default is not stored_empty_dict

    m1.int_list_with_default.append(1)
    m1.str_dict_with_default['a'] = 'abc'

    m2 = v.construct_python({}, recursive=True)

    assert m2.int_list_with_default == []
    assert m2.str_dict_with_default == {}

    assert m2.int_list_with_default is not m1.int_list_with_default
    assert m2.str_dict_with_default is not m1.str_dict_with_default


@pytest.mark.skipif(sys.version_info < (3, 10), reason='pattern matching was added in 3.10')
def test_some_pattern_match() -> None:
    code = """\
def f(v: Union[Some[Any], None]) -> str:
    match v:
        case Some(1):
            return 'case1'
        case Some(value=2):
            return 'case2'
        case Some(int(value)):
            return f'case3: {value}'
        case Some(value):
            return f'case4: {type(value).__name__}({value})'
        case None:
            return 'case5'
"""

    local_vars = {}
    exec(code, globals(), local_vars)
    f = cast(Callable[[Union[Some[Any], None]], str], local_vars['f'])

    res = f(SchemaValidator(core_schema.with_default_schema(core_schema.int_schema(), default=1)).get_default_value())
    assert res == 'case1'

    res = f(SchemaValidator(core_schema.with_default_schema(core_schema.int_schema(), default=2)).get_default_value())
    assert res == 'case2'

    res = f(SchemaValidator(core_schema.with_default_schema(core_schema.int_schema(), default=3)).get_default_value())
    assert res == 'case3: 3'

    res = f(SchemaValidator(core_schema.with_default_schema(core_schema.int_schema(), default='4')).get_default_value())
    assert res == 'case4: str(4)'

    res = f(SchemaValidator(core_schema.int_schema()).get_default_value())
    assert res == 'case5'


def test_use_default_error() -> None:
    """Validator functions have no effect when constructing"""

    def val_func(v: Any, handler: core_schema.ValidatorFunctionWrapHandler) -> Any:
        if isinstance(v, str) and v == '':
            raise PydanticUseDefault
        return handler(v)

    validator = SchemaValidator(
        core_schema.with_default_schema(
            core_schema.no_info_wrap_validator_function(val_func, core_schema.int_schema()), default=10
        )
    )

    assert validator.construct_python('1') == '1'
    assert validator.construct_python('') == ''

    validator = SchemaValidator(
        core_schema.with_default_schema(core_schema.no_info_wrap_validator_function(val_func, core_schema.int_schema()))
    )
    assert validator.construct_python('') == ''

    validator = SchemaValidator(core_schema.no_info_wrap_validator_function(val_func, core_schema.int_schema()))
    assert validator.construct_python('') == ''


def test_recursive_construct_default():
    """Default values themselves can be recursively constructed"""

    class Child:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        a: int
        b: int

    class Model:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        value: Child = {'a': 10, 'b': 'wrong'}

    # Turn off constructing defaults
    v = SchemaValidator(
        core_schema.model_schema(
            Model,
            core_schema.model_fields_schema(
                {
                    'value': core_schema.model_field(
                        core_schema.with_default_schema(
                            core_schema.model_schema(
                                Child,
                                core_schema.model_fields_schema(
                                    {
                                        'a': core_schema.model_field(core_schema.int_schema()),
                                        'b': core_schema.model_field(core_schema.int_schema()),
                                    }
                                ),
                            ),
                            default={'a': 10, 'b': 'wrong'},
                            construct_default=False,
                        )
                    )
                }
            ),
        )
    )
    m = v.construct_python({}, recursive=True)
    assert m.value == {'a': 10, 'b': 'wrong'}

    # Turn on constructing defaults
    v = SchemaValidator(
        core_schema.model_schema(
            Model,
            core_schema.model_fields_schema(
                {
                    'value': core_schema.model_field(
                        core_schema.with_default_schema(
                            core_schema.model_schema(
                                Child,
                                core_schema.model_fields_schema(
                                    {
                                        'a': core_schema.model_field(core_schema.int_schema()),
                                        'b': core_schema.model_field(core_schema.int_schema()),
                                    }
                                ),
                            ),
                            default={'a': 10, 'b': 'wrong'},
                            construct_default=True,
                        )
                    )
                }
            ),
        )
    )
    m = v.construct_python({}, recursive=True)

    assert isinstance(m.value, Child)
    assert m.value.a == 10
    assert m.value.b == 'wrong'
    assert m.value.__pydantic_fields_set__ == {'a', 'b'}


def test_all_default_model_configurations():
    class Child:
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'
        a: int
        b: int

        def __init__(self, a, b):
            self.a = a
            self.b = b

        def __eq__(self, other):
            if not isinstance(other, Child):
                return False
            return self.a == other.a and self.b == other.b

    class DefaultTest:
        a: Child
        b: Child
        c: Child
        d: Child

    child_schema = core_schema.model_schema(
        Child,
        core_schema.model_fields_schema(
            {
                'a': core_schema.model_field(core_schema.int_schema()),
                'b': core_schema.model_field(core_schema.int_schema()),
            }
        ),
    )

    default_test_schema = core_schema.model_schema(
        DefaultTest,
        core_schema.model_fields_schema(
            {
                'a': core_schema.model_field(
                    core_schema.with_default_schema(
                        child_schema, default={'a': 10, 'b': 'wrong'}, construct_default=False
                    )
                ),
                'b': core_schema.model_field(
                    core_schema.with_default_schema(
                        child_schema, default={'a': 10, 'b': 'wrong'}, construct_default=True
                    )
                ),
                'c': core_schema.model_field(
                    core_schema.with_default_schema(
                        child_schema, default_factory=lambda: {'a': 10, 'b': 'generated'}, construct_default=False
                    )
                ),
                'd': core_schema.model_field(
                    core_schema.with_default_schema(
                        child_schema, default_factory=lambda: {'a': 10, 'b': 'generated'}, construct_default=True
                    )
                ),
            }
        ),
    )

    # class DefaultTest:
    #     a: Child = Field({'a': 10, 'b': 'wrong'}, construct_defaults=False)
    #     b: Child = Field({'a': 10, 'b': 'wrong'}, construct_defaults=True)
    #     c: Child = Field(default_factory=lambda: {'a': 10, 'b': 'generated'}, construct_defaults=False)
    #     d: Child = Field(default_factory=lambda: {'a': 10, 'b': 'generated'}, construct_defaults=True)
    v = SchemaValidator(default_test_schema)

    m = v.construct_python({}, recursive=True)
    assert isinstance(m, DefaultTest)
    assert m.a == {'a': 10, 'b': 'wrong'}
    assert m.b == Child(10, 'wrong')
    assert m.c == {'a': 10, 'b': 'generated'}
    assert m.d == Child(10, 'generated')
