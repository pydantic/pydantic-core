from __future__ import annotations as _annotations

import sys
from datetime import date, datetime, time, timedelta
from typing import Any, Callable, Dict, List, Optional, Set, Type, Union

if sys.version_info < (3, 11):
    from typing_extensions import Protocol, Required
else:
    from typing import Protocol, Required

if sys.version_info < (3, 9):
    from typing_extensions import Literal, TypedDict
else:
    from typing import Literal, TypedDict


def dict_not_none(**kwargs: Any) -> Any:
    return {k: v for k, v in kwargs.items() if v is not None}


class CoreConfig(TypedDict, total=False):
    title: str
    strict: bool
    # higher priority configs take precedence of over lower, if priority matches the two configs are merged, default 0
    config_choose_priority: int
    # if configs are merged, which should take precedence, default 0, default means child takes precedence
    config_merge_priority: int
    # settings related to typed_dicts only
    typed_dict_extra_behavior: Literal['allow', 'forbid', 'ignore']
    typed_dict_total: bool  # default: True
    # used on typed-dicts and tagged union keys
    from_attributes: bool
    revalidate_models: bool
    # used on typed-dicts and arguments
    populate_by_name: bool  # replaces `allow_population_by_field_name` in pydantic v1
    # fields related to string fields only
    str_max_length: int
    str_min_length: int
    str_strip_whitespace: bool
    str_to_lower: bool
    str_to_upper: bool
    # fields related to float fields only
    allow_inf_nan: bool  # default: True


class AnySchema(TypedDict, total=False):
    type: Required[Literal['any']]
    ref: str
    extra: Any


def any_schema(*, ref: str | None = None, extra: Any = None) -> AnySchema:
    """Returns a schema that matches any value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.any_schema()
    v = SchemaValidator(schema)
    v.validate_python(1)
    ```
    """
    return dict_not_none(type='any', ref=ref, extra=extra)


class NoneSchema(TypedDict, total=False):
    type: Required[Literal['none']]
    ref: str
    extra: Any


def none_schema(*, ref: str | None = None, extra: Any = None) -> NoneSchema:
    """Returns a schema that matches a None value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.none_schema()
    v = SchemaValidator(schema)
    v.validate_python(None)
    ```
    """
    return dict_not_none(type='none', ref=ref, extra=extra)


class BoolSchema(TypedDict, total=False):
    type: Required[Literal['bool']]
    strict: bool
    ref: str
    extra: Any


def bool_schema(strict: bool | None = None, ref: str | None = None, extra: Any = None) -> BoolSchema:
    """Returns a schema that matches a bool value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.bool_schema()
    v = SchemaValidator(schema)
    v.validate_python(True)
    ```
    """
    return dict_not_none(type='bool', strict=strict, ref=ref, extra=extra)


class IntSchema(TypedDict, total=False):
    type: Required[Literal['int']]
    multiple_of: int
    le: int
    ge: int
    lt: int
    gt: int
    strict: bool
    ref: str
    extra: Any


def int_schema(
    *,
    multiple_of: int | None = None,
    le: int | None = None,
    ge: int | None = None,
    lt: int | None = None,
    gt: int | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> IntSchema:
    """Returns a schema that matches a int value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.int_schema(multiple_of=2, le=6, ge=2)
    v = SchemaValidator(schema)
    v.validate_python(4)
    ```
    """
    return dict_not_none(
        type='int', multiple_of=multiple_of, le=le, ge=ge, lt=lt, gt=gt, strict=strict, ref=ref, extra=extra
    )


class FloatSchema(TypedDict, total=False):
    type: Required[Literal['float']]
    allow_inf_nan: bool  # whether 'NaN', '+inf', '-inf' should be forbidden. default: True
    multiple_of: float
    le: float
    ge: float
    lt: float
    gt: float
    strict: bool
    ref: str
    extra: Any


def float_schema(
    *,
    allow_inf_nan: bool | None = None,
    multiple_of: float | None = None,
    le: float | None = None,
    ge: float | None = None,
    lt: float | None = None,
    gt: float | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> FloatSchema:
    """Returns a schema that matches a float value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.float_schema(le=0.8, ge=0.2)
    v = SchemaValidator(schema)
    v.validate_python(0.5)
    ```
    """
    return dict_not_none(
        type='float',
        allow_inf_nan=allow_inf_nan,
        multiple_of=multiple_of,
        le=le,
        ge=ge,
        lt=lt,
        gt=gt,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class StringSchema(TypedDict, total=False):
    type: Required[Literal['str']]
    pattern: str
    max_length: int
    min_length: int
    strip_whitespace: bool
    to_lower: bool
    to_upper: bool
    strict: bool
    ref: str
    extra: Any


def string_schema(
    *,
    pattern: str | None = None,
    max_length: int | None = None,
    min_length: int | None = None,
    strip_whitespace: bool | None = None,
    to_lower: bool | None = None,
    to_upper: bool | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> StringSchema:
    """Returns a schema that matches a string value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.string_schema(max_length=10, min_length=2)
    v = SchemaValidator(schema)
    v.validate_python('hello')
    ```
    """
    return dict_not_none(
        type='str',
        pattern=pattern,
        max_length=max_length,
        min_length=min_length,
        strip_whitespace=strip_whitespace,
        to_lower=to_lower,
        to_upper=to_upper,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class BytesSchema(TypedDict, total=False):
    type: Required[Literal['bytes']]
    max_length: int
    min_length: int
    strict: bool
    ref: str
    extra: Any


def bytes_schema(
    *,
    max_length: int | None = None,
    min_length: int | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> BytesSchema:
    """Returns a schema that matches a bytes value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.bytes_schema(max_length=10, min_length=2)
    v = SchemaValidator(schema)
    v.validate_python(b'hello')
    ```
    """
    return dict_not_none(
        type='bytes', max_length=max_length, min_length=min_length, strict=strict, ref=ref, extra=extra
    )


class DateSchema(TypedDict, total=False):
    type: Required[Literal['date']]
    strict: bool
    le: date
    ge: date
    lt: date
    gt: date
    now_op: Literal['past', 'future']
    # defaults to current local utc offset from `time.localtime().tm_gmtoff`
    # value is restricted to -86_400 < offset < 86_400 by bounds in generate_self_schema.py
    now_utc_offset: int
    ref: str
    extra: Any


def date_schema(
    *,
    strict: bool | None = None,
    le: date | None = None,
    ge: date | None = None,
    lt: date | None = None,
    gt: date | None = None,
    ref: str | None = None,
    now_op: Literal['past', 'future'] | None = None,
    now_utc_offset: int | None = None,
    extra: Any = None,
) -> DateSchema:
    """Returns a schema that matches a date value, e.g.:

    ```python
    from datetime import date
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.date_schema(le=date(2020, 1, 1), ge=date(2019, 1, 1))
    v = SchemaValidator(schema)
    v.validate_python(date(2019, 6, 1))
    ```
    """
    return dict_not_none(
        type='date',
        strict=strict,
        le=le,
        ge=ge,
        lt=lt,
        gt=gt,
        now_op=now_op,
        now_utc_offset=now_utc_offset,
        ref=ref,
        extra=extra,
    )


class TimeSchema(TypedDict, total=False):
    type: Required[Literal['time']]
    strict: bool
    le: time
    ge: time
    lt: time
    gt: time
    ref: str
    extra: Any


def time_schema(
    *,
    strict: bool | None = None,
    le: time | None = None,
    ge: time | None = None,
    lt: time | None = None,
    gt: time | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> TimeSchema:
    """Returns a schema that matches a time value, e.g.:

    ```python
    from datetime import time
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.time_schema(le=time(12, 0, 0), ge=time(6, 0, 0))
    v = SchemaValidator(schema)
    v.validate_python(time(9, 0, 0))
    ```
    """
    return dict_not_none(type='time', strict=strict, le=le, ge=ge, lt=lt, gt=gt, ref=ref, extra=extra)


class DatetimeSchema(TypedDict, total=False):
    type: Required[Literal['datetime']]
    strict: bool
    le: datetime
    ge: datetime
    lt: datetime
    gt: datetime
    now_op: Literal['past', 'future']
    tz_constraint: Literal['aware', 'naive']
    # defaults to current local utc offset from `time.localtime().tm_gmtoff`
    # value is restricted to -86_400 < offset < 86_400 by bounds in generate_self_schema.py
    now_utc_offset: int
    ref: str
    extra: Any


def datetime_schema(
    *,
    strict: bool | None = None,
    le: datetime | None = None,
    ge: datetime | None = None,
    lt: datetime | None = None,
    gt: datetime | None = None,
    now_op: Literal['past', 'future'] | None = None,
    tz_constraint: Literal['aware', 'naive'] | None = None,
    now_utc_offset: int | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> DatetimeSchema:
    """Returns a schema that matches a datetime value, e.g.:

    ```python
    from datetime import datetime
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.datetime_schema()
    v = SchemaValidator(schema)
    v.validate_python(datetime.now())
    ```
    """
    return dict_not_none(
        type='datetime',
        strict=strict,
        le=le,
        ge=ge,
        lt=lt,
        gt=gt,
        now_op=now_op,
        tz_constraint=tz_constraint,
        now_utc_offset=now_utc_offset,
        ref=ref,
        extra=extra,
    )


class TimedeltaSchema(TypedDict, total=False):
    type: Required[Literal['timedelta']]
    strict: bool
    le: timedelta
    ge: timedelta
    lt: timedelta
    gt: timedelta
    ref: str
    extra: Any


def timedelta_schema(
    *,
    strict: bool | None = None,
    le: timedelta | None = None,
    ge: timedelta | None = None,
    lt: timedelta | None = None,
    gt: timedelta | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> TimedeltaSchema:
    """Returns a schema that matches a timedelta value, e.g.:

    ```python
    from datetime import timedelta
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.timedelta_schema(le=timedelta(days=1), ge=timedelta(days=0))
    v = SchemaValidator(schema)
    v.validate_python(timedelta(hours=12))
    ```
    """
    return dict_not_none(type='timedelta', strict=strict, le=le, ge=ge, lt=lt, gt=gt, ref=ref, extra=extra)


class LiteralSchema(TypedDict, total=False):
    type: Required[Literal['literal']]
    expected: Required[List[Any]]
    ref: str
    extra: Any


def literal_schema(*expected: Any, ref: str | None = None, extra: Any = None) -> LiteralSchema:
    """Returns a schema that matches a literal value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.literal_schema('hello', "world")
    v = SchemaValidator(schema)
    v.validate_python('hello')
    ```
    """
    return dict_not_none(type='literal', expected=expected, ref=ref, extra=extra)


# must match input/parse_json.rs::JsonType::try_from
JsonType = Literal['null', 'bool', 'int', 'float', 'str', 'list', 'dict']


class IsInstanceSchema(TypedDict, total=False):
    type: Required[Literal['is-instance']]
    cls: Required[Any]
    cls_repr: str
    json_types: Set[JsonType]
    json_function: Callable[[Any], Any]
    ref: str
    extra: Any


def is_instance_schema(
    cls: Any,
    *,
    json_types: Set[JsonType] | None = None,
    json_function: Callable[[Any], Any] | None = None,
    cls_repr: str | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> IsInstanceSchema:
    """Returns a schema that matches an object to a class, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema

    class A:
        pass

    schema = core_schema.is_instance_schema(cls=A)
    v = SchemaValidator(schema)
    v.validate_python(A())
    ```
    """
    return dict_not_none(
        type='is-instance',
        cls=cls,
        json_types=json_types,
        json_function=json_function,
        cls_repr=cls_repr,
        ref=ref,
        extra=extra,
    )


class IsSubclassSchema(TypedDict, total=False):
    type: Required[Literal['is-subclass']]
    cls: Required[Type[Any]]
    cls_repr: str
    ref: str
    extra: Any


def is_subclass_schema(
    cls: Type[Any], *, cls_repr: str | None = None, ref: str | None = None, extra: Any = None
) -> IsInstanceSchema:
    """Returns a schema that matches a subclass to a class, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema

    class A:
        pass

    class B(A):
        pass

    schema = core_schema.is_subclass_schema(cls=A)
    v = SchemaValidator(schema)
    v.validate_python(B)
    ```
    """
    return dict_not_none(type='is-subclass', cls=cls, cls_repr=cls_repr, ref=ref, extra=extra)


class CallableSchema(TypedDict, total=False):
    type: Required[Literal['callable']]
    ref: str
    extra: Any


def callable_schema(*, ref: str | None = None, extra: Any = None) -> CallableSchema:
    """Returns a schema that matches a callable value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.callable_schema()
    v = SchemaValidator(schema)
    v.validate_python(min)
    ```
    """
    return dict_not_none(type='callable', ref=ref, extra=extra)


class ListSchema(TypedDict, total=False):
    type: Required[Literal['list']]
    items_schema: CoreSchema
    min_length: int
    max_length: int
    strict: bool
    allow_any_iter: bool
    ref: str
    extra: Any


def list_schema(
    items_schema: CoreSchema | None = None,
    *,
    min_length: int | None = None,
    max_length: int | None = None,
    strict: bool | None = None,
    allow_any_iter: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> ListSchema:
    """Returns a schema that matches a list value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.list_schema(core_schema.string_schema(), min_length=0, max_length=10)
    v = SchemaValidator(schema)
    v.validate_python(['hello'])
    ```
    """
    return dict_not_none(
        type='list',
        items_schema=items_schema,
        min_length=min_length,
        max_length=max_length,
        strict=strict,
        allow_any_iter=allow_any_iter,
        ref=ref,
        extra=extra,
    )


class TuplePositionalSchema(TypedDict, total=False):
    type: Required[Literal['tuple']]
    mode: Required[Literal['positional']]
    items_schema: Required[List[CoreSchema]]
    extra_schema: CoreSchema
    strict: bool
    ref: str
    extra: Any


def tuple_positional_schema(
    *items_schema: CoreSchema,
    extra_schema: CoreSchema | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> TuplePositionalSchema:
    """Returns a schema that matches a tuple of schemas, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.tuple_positional_schema(core_schema.int_schema(), core_schema.string_schema())
    v = SchemaValidator(schema)
    v.validate_python((1, 'hello'))
    ```
    """
    return dict_not_none(
        type='tuple',
        mode='positional',
        items_schema=items_schema,
        extra_schema=extra_schema,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class TupleVariableSchema(TypedDict, total=False):
    type: Required[Literal['tuple']]
    mode: Literal['variable']
    items_schema: CoreSchema
    min_length: int
    max_length: int
    strict: bool
    ref: str
    extra: Any


def tuple_variable_schema(
    items_schema: CoreSchema | None = None,
    *,
    min_length: int | None = None,
    max_length: int | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> TupleVariableSchema:
    """Returns a schema that matches a tuple of a given schema, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.tuple_variable_schema(items_schema=core_schema.int_schema(), min_length=0, max_length=10)
    v = SchemaValidator(schema)
    v.validate_python((1, 2, 3))
    ```
    """
    return dict_not_none(
        type='tuple',
        mode='variable',
        items_schema=items_schema,
        min_length=min_length,
        max_length=max_length,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class SetSchema(TypedDict, total=False):
    type: Required[Literal['set']]
    items_schema: CoreSchema
    min_length: int
    max_length: int
    generator_max_length: int
    strict: bool
    ref: str
    extra: Any


def set_schema(
    items_schema: CoreSchema | None = None,
    *,
    min_length: int | None = None,
    max_length: int | None = None,
    generator_max_length: int | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> SetSchema:
    """Returns a schema that matches a set of a given schema, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.set_schema(items_schema=core_schema.int_schema(), min_length=0, max_length=10)
    v = SchemaValidator(schema)
    v.validate_python({1, 2, 3})
    ```
    """
    return dict_not_none(
        type='set',
        items_schema=items_schema,
        min_length=min_length,
        max_length=max_length,
        generator_max_length=generator_max_length,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class FrozenSetSchema(TypedDict, total=False):
    type: Required[Literal['frozenset']]
    items_schema: CoreSchema
    min_length: int
    max_length: int
    generator_max_length: int
    strict: bool
    ref: str
    extra: Any


def frozenset_schema(
    items_schema: CoreSchema | None = None,
    *,
    min_length: int | None = None,
    max_length: int | None = None,
    generator_max_length: int | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> FrozenSetSchema:
    """Returns a schema that matches a frozenset of a given schema, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.frozenset_schema(items_schema=core_schema.int_schema(), min_length=0, max_length=10)
    v = SchemaValidator(schema)
    v.validate_python(frozenset({1, 2, 3}))
    ```
    """
    return dict_not_none(
        type='frozenset',
        items_schema=items_schema,
        min_length=min_length,
        max_length=max_length,
        generator_max_length=generator_max_length,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class GeneratorSchema(TypedDict, total=False):
    type: Required[Literal['generator']]
    items_schema: CoreSchema
    max_length: int
    ref: str
    extra: Any


def generator_schema(
    items_schema: CoreSchema | None = None, *, max_length: int | None = None, ref: str | None = None, extra: Any = None
) -> GeneratorSchema:
    """Returns a schema that matches a generator value, e.g.:

    ```python
    from typing import Iterator
    from pydantic_core import SchemaValidator, core_schema

    def gen() -> Iterator[int]:
        yield 1

    schema = core_schema.generator_schema(items_schema=core_schema.int_schema())
    v = SchemaValidator(schema)
    v.validate_python(gen())
    ```
    """
    return dict_not_none(type='generator', items_schema=items_schema, max_length=max_length, ref=ref, extra=extra)


class DictSchema(TypedDict, total=False):
    type: Required[Literal['dict']]
    keys_schema: CoreSchema  # default: AnySchema
    values_schema: CoreSchema  # default: AnySchema
    min_length: int
    max_length: int
    strict: bool
    ref: str
    extra: Any


def dict_schema(
    keys_schema: CoreSchema | None = None,
    values_schema: CoreSchema | None = None,
    *,
    min_length: int | None = None,
    max_length: int | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> DictSchema:
    """Returns a schema that matches a dict value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.dict_schema(
        keys_schema=core_schema.string_schema(), values_schema=core_schema.int_schema()
    )
    v = SchemaValidator(schema)
    v.validate_python({'a': 1, 'b': 2})
    ```
    """
    return dict_not_none(
        type='dict',
        keys_schema=keys_schema,
        values_schema=values_schema,
        min_length=min_length,
        max_length=max_length,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class ValidatorFunction(Protocol):
    def __call__(
        self, __input_value: Any, *, data: Any, config: CoreConfig | None, context: Any, **future_kwargs: Any
    ) -> Any:  # pragma: no cover
        ...


class FunctionSchema(TypedDict, total=False):
    type: Required[Literal['function']]
    mode: Required[Literal['before', 'after']]
    function: Required[ValidatorFunction]
    schema: Required[CoreSchema]
    ref: str
    extra: Any


def function_before_schema(
    function: ValidatorFunction, schema: CoreSchema, *, ref: str | None = None, extra: Any = None
) -> FunctionSchema:
    """Returns a schema that calls a validator function before validating the provided schema, e.g.:

    ```python
    from typing import Any
    from pydantic_core import SchemaValidator, core_schema

    def fn(v: Any, **kwargs) -> str:
        v_str = str(v)
        assert 'hello' in v_str
        return v_str + 'world'

    schema = core_schema.function_before_schema(function=fn, schema=core_schema.string_schema())
    v = SchemaValidator(schema)
    v.validate_python(b"hello ")
    ```
    """
    return dict_not_none(type='function', mode='before', function=function, schema=schema, ref=ref, extra=extra)


def function_after_schema(
    schema: CoreSchema, function: ValidatorFunction, *, ref: str | None = None, extra: Any = None
) -> FunctionSchema:
    """Returns a schema that calls a validator function after validating the provided schema, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema

    def fn(v: str, **kwargs) -> str:
        assert 'hello' in v
        return v + 'world'

    schema = core_schema.function_after_schema(schema=core_schema.string_schema(), function=fn)
    v = SchemaValidator(schema)
    v.validate_python("hello ")
    ```
    """
    return dict_not_none(type='function', mode='after', function=function, schema=schema, ref=ref, extra=extra)


class CallableValidator(Protocol):
    def __call__(self, input_value: Any, outer_location: str | int | None = None) -> Any:  # pragma: no cover
        ...


class WrapValidatorFunction(Protocol):
    def __call__(
        self,
        __input_value: Any,
        *,
        validator: CallableValidator,
        data: Any,
        config: CoreConfig | None,
        context: Any,
        **future_kwargs: Any,
    ) -> Any:  # pragma: no cover
        ...


class FunctionWrapSchema(TypedDict, total=False):
    type: Required[Literal['function']]
    mode: Required[Literal['wrap']]
    function: Required[WrapValidatorFunction]
    schema: Required[CoreSchema]
    ref: str
    extra: Any


def function_wrap_schema(
    function: WrapValidatorFunction, schema: CoreSchema, *, ref: str | None = None, extra: Any = None
) -> FunctionWrapSchema:
    """Returns a schema that wraps a function around the result of schema validation, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema

    def fn(v: str, **kwargs) -> str:
        assert 'hello' in v
        return v + 'world'

    schema = core_schema.function_wrap_schema(function=fn, schema=core_schema.string_schema())
    v = SchemaValidator(schema)
    v.validate_python("hello ")
    ```
    """
    return dict_not_none(type='function', mode='wrap', function=function, schema=schema, ref=ref, extra=extra)


class FunctionPlainSchema(TypedDict, total=False):
    type: Required[Literal['function']]
    mode: Required[Literal['plain']]
    function: Required[ValidatorFunction]
    ref: str
    extra: Any


def function_plain_schema(
    function: ValidatorFunction, *, ref: str | None = None, extra: Any = None
) -> FunctionPlainSchema:
    """Returns a schema that uses the provided function for validation, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema

    def fn(v: str, **kwargs) -> str:
        assert 'hello' in v
        return v + 'world'

    schema = core_schema.function_plain_schema(function=fn)
    v = SchemaValidator(schema)
    v.validate_python("hello ")
    ```
    """
    return dict_not_none(type='function', mode='plain', function=function, ref=ref, extra=extra)


class WithDefaultSchema(TypedDict, total=False):
    type: Required[Literal['default']]
    schema: Required[CoreSchema]
    default: Any
    default_factory: Callable[[], Any]
    on_error: Literal['raise', 'omit', 'default']  # default: 'raise'
    strict: bool
    ref: str
    extra: Any


Omitted = object()


def with_default_schema(
    schema: CoreSchema,
    *,
    default: Any = Omitted,
    default_factory: Callable[[], Any] | None = None,
    on_error: Literal['raise', 'omit', 'default'] | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> WithDefaultSchema:
    """Returns a schema that adds a default value to the given schema, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.with_default_schema(core_schema.string_schema(), default='hello')
    wrapper_schema = core_schema.typed_dict_schema(
        {'a': core_schema.typed_dict_field(schema)}
    )
    v = SchemaValidator(wrapper_schema)
    assert v.validate_python({}) == v.validate_python({'a': 'hello'})
    ```
    """
    s = dict_not_none(
        type='default',
        schema=schema,
        default_factory=default_factory,
        on_error=on_error,
        strict=strict,
        ref=ref,
        extra=extra,
    )
    if default is not Omitted:
        s['default'] = default
    return s


class NullableSchema(TypedDict, total=False):
    type: Required[Literal['nullable']]
    schema: Required[CoreSchema]
    strict: bool
    ref: str
    extra: Any


def nullable_schema(
    schema: CoreSchema, *, strict: bool | None = None, ref: str | None = None, extra: Any = None
) -> NullableSchema:
    """Returns a schema that matches a nullable value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.nullable_schema(core_schema.string_schema())
    v = SchemaValidator(schema)
    v.validate_python(None)
    ```
    """
    return dict_not_none(type='nullable', schema=schema, strict=strict, ref=ref, extra=extra)


class UnionSchema(TypedDict, total=False):
    type: Required[Literal['union']]
    choices: Required[List[CoreSchema]]
    custom_error_type: str
    custom_error_message: str
    custom_error_context: Dict[str, Union[str, int, float]]
    strict: bool
    ref: str
    extra: Any


def union_schema(
    *choices: CoreSchema,
    custom_error_type: str | None = None,
    custom_error_message: str | None = None,
    custom_error_context: dict[str, str | int] | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> UnionSchema:
    """Returns a schema that matches a union value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.union_schema(core_schema.string_schema(), core_schema.int_schema())
    v = SchemaValidator(schema)
    v.validate_python('hello')
    v.validate_python(1)
    ```
    """
    return dict_not_none(
        type='union',
        choices=choices,
        custom_error_type=custom_error_type,
        custom_error_message=custom_error_message,
        custom_error_context=custom_error_context,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class TaggedUnionSchema(TypedDict, total=False):
    type: Required[Literal['tagged-union']]
    choices: Required[Dict[str, CoreSchema]]
    discriminator: Required[
        Union[str, List[Union[str, int]], List[List[Union[str, int]]], Callable[[Any], Optional[str]]]
    ]
    custom_error_type: str
    custom_error_message: str
    custom_error_context: Dict[str, Union[str, int, float]]
    strict: bool
    ref: str
    extra: Any


def tagged_union_schema(
    choices: Dict[str, CoreSchema],
    discriminator: str | list[str | int] | list[list[str | int]] | Callable[[Any], str | None],
    *,
    custom_error_type: str | None = None,
    custom_error_message: str | None = None,
    custom_error_context: dict[str, int | str | float] | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> TaggedUnionSchema:
    """Returns a schema that matches a tagged union value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    apple_schema = core_schema.typed_dict_schema(
        {
            'foo': core_schema.typed_dict_field(core_schema.string_schema()),
            'bar': core_schema.typed_dict_field(core_schema.int_schema()),
        }
    )
    banana_schema = core_schema.typed_dict_schema(
        {
            'foo': core_schema.typed_dict_field(core_schema.string_schema()),
            'spam': core_schema.typed_dict_field(core_schema.list_schema(items_schema=core_schema.int_schema())),
        }
    )
    schema = core_schema.tagged_union_schema(
        choices={
            'apple': apple_schema,
            'banana': banana_schema,
        },
        discriminator='foo',
    )
    v = SchemaValidator(schema)
    v.validate_python({'foo': 'apple', 'bar': '123'})
    v.validate_python({'foo': 'banana', 'spam': [1, 2, 3]})
    ```
    """
    return dict_not_none(
        type='tagged-union',
        choices=choices,
        discriminator=discriminator,
        custom_error_type=custom_error_type,
        custom_error_message=custom_error_message,
        custom_error_context=custom_error_context,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class ChainSchema(TypedDict, total=False):
    type: Required[Literal['chain']]
    steps: Required[List[CoreSchema]]
    ref: str
    extra: Any


def chain_schema(*steps: CoreSchema, ref: str | None = None, extra: Any = None) -> ChainSchema:
    """Returns a schema that chains the provided validation schemas, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema

    def fn(v: str, **kwargs) -> str:
        assert 'hello' in v
        return v + ' world'

    fn_schema = core_schema.function_plain_schema(function=fn)
    schema = core_schema.chain_schema(fn_schema, fn_schema, fn_schema, core_schema.string_schema())
    v = SchemaValidator(schema)
    assert v.validate_python("hello") == 'hello world world world'
    ```
    """
    return dict_not_none(type='chain', steps=steps, ref=ref, extra=extra)


class LaxOrStrictSchema(TypedDict, total=False):
    type: Required[Literal['lax-or-strict']]
    lax_schema: Required[CoreSchema]
    strict_schema: Required[CoreSchema]
    strict: bool
    ref: str
    extra: Any


def lax_or_strict_schema(
    lax_schema: CoreSchema,
    strict_schema: CoreSchema,
    *,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> LaxOrStrictSchema:
    """Returns a schema that uses the lax or strict schema, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema

    def fn(v: str, **kwargs) -> str:
        assert 'hello' in v
        return v + ' world'

    lax_schema = core_schema.int_schema(strict=False)
    strict_schema = core_schema.int_schema(strict=True)

    schema = core_schema.lax_or_strict_schema(lax_schema=lax_schema, strict_schema=strict_schema, strict=True)
    v = SchemaValidator(schema)
    assert v.validate_python(123) == 123

    schema = core_schema.lax_or_strict_schema(lax_schema=lax_schema, strict_schema=strict_schema, strict=False)
    v = SchemaValidator(schema)
    assert v.validate_python('123') == 123
    ```
    """
    return dict_not_none(
        type='lax-or-strict', lax_schema=lax_schema, strict_schema=strict_schema, strict=strict, ref=ref, extra=extra
    )


class TypedDictField(TypedDict, total=False):
    schema: Required[CoreSchema]
    required: bool
    alias: Union[str, List[Union[str, int]], List[List[Union[str, int]]]]
    frozen: bool


def typed_dict_field(
    schema: CoreSchema,
    *,
    required: bool | None = None,
    alias: str | list[str | int] | list[list[str | int]] | None = None,
    frozen: bool | None = None,
) -> TypedDictField:
    """Returns a schema that matches a typed dict field, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    field = core_schema.typed_dict_field(schema=core_schema.int_schema(), required=True)
    ```
    """
    return dict_not_none(schema=schema, required=required, alias=alias, frozen=frozen)


class TypedDictSchema(TypedDict, total=False):
    type: Required[Literal['typed-dict']]
    fields: Required[Dict[str, TypedDictField]]
    strict: bool
    extra_validator: CoreSchema
    return_fields_set: bool
    ref: str
    extra: Any
    # all these values can be set via config, equivalent fields have `typed_dict_` prefix
    extra_behavior: Literal['allow', 'forbid', 'ignore']
    total: bool  # default: True
    populate_by_name: bool  # replaces `allow_population_by_field_name` in pydantic v1
    from_attributes: bool


def typed_dict_schema(
    fields: Dict[str, TypedDictField],
    *,
    strict: bool | None = None,
    extra_validator: CoreSchema | None = None,
    return_fields_set: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
    extra_behavior: Literal['allow', 'forbid', 'ignore'] | None = None,
    total: bool | None = None,
    populate_by_name: bool | None = None,
    from_attributes: bool | None = None,
) -> TypedDictSchema:
    """Returns a schema that matches a typed dict, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    wrapper_schema = core_schema.typed_dict_schema(
        {'a': core_schema.typed_dict_field(core_schema.string_schema())}
    )
    v = SchemaValidator(wrapper_schema)
    v.validate_python({'a': 'hello'})
    ```
    """
    return dict_not_none(
        type='typed-dict',
        fields=fields,
        strict=strict,
        extra_validator=extra_validator,
        return_fields_set=return_fields_set,
        ref=ref,
        extra=extra,
        extra_behavior=extra_behavior,
        total=total,
        populate_by_name=populate_by_name,
        from_attributes=from_attributes,
    )


class NewClassSchema(TypedDict, total=False):
    type: Required[Literal['new-class']]
    cls: Required[Type[Any]]
    schema: Required[CoreSchema]
    call_after_init: str
    strict: bool
    ref: str
    extra: Any
    config: CoreConfig


def new_class_schema(
    cls: Type[Any],
    schema: CoreSchema,
    *,
    call_after_init: str | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
    config: CoreConfig | None = None,
) -> NewClassSchema:
    """Returns a schema that matches a new class, e.g.:

    ```python
    from pydantic_core import CoreConfig, SchemaValidator, core_schema

    class MyModel:
        __slots__ = '__dict__', '__fields_set__'

    schema = core_schema.new_class_schema(
        cls=MyModel,
        config=CoreConfig(str_max_length=5),
        schema=core_schema.typed_dict_schema(
            fields={'a': core_schema.typed_dict_field(core_schema.string_schema())},
        ),
    )
    v = SchemaValidator(schema)
    assert v.isinstance_python({'a': 'hello'}) is True
    assert v.isinstance_python({'a': 'too long'}) is False
    ```
    """
    return dict_not_none(
        type='new-class', cls=cls, schema=schema, call_after_init=call_after_init, strict=strict, ref=ref, config=config
    )


class ArgumentsParameter(TypedDict, total=False):
    name: Required[str]
    schema: Required[CoreSchema]
    mode: Literal['positional_only', 'positional_or_keyword', 'keyword_only']  # default positional_or_keyword
    alias: Union[str, List[Union[str, int]], List[List[Union[str, int]]]]


def arguments_parameter(
    name: str,
    schema: CoreSchema,
    *,
    mode: Literal['positional_only', 'positional_or_keyword', 'keyword_only'] | None = None,
    alias: str | list[str | int] | list[list[str | int]] | None = None,
) -> ArgumentsParameter:
    """Returns a schema that matches an argument parameter, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    param = core_schema.arguments_parameter(name='a', schema=core_schema.string_schema(), mode='positional_only')
    ```
    """
    return dict_not_none(name=name, schema=schema, mode=mode, alias=alias)


class ArgumentsSchema(TypedDict, total=False):
    type: Required[Literal['arguments']]
    arguments_schema: Required[List[ArgumentsParameter]]
    populate_by_name: bool
    var_args_schema: CoreSchema
    var_kwargs_schema: CoreSchema
    ref: str
    extra: Any


def arguments_schema(
    *arguments: ArgumentsParameter,
    populate_by_name: bool | None = None,
    var_args_schema: CoreSchema | None = None,
    var_kwargs_schema: CoreSchema | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> ArgumentsSchema:
    """Returns a schema that matches an arguments schema, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    param_a = core_schema.arguments_parameter(name='a', schema=core_schema.string_schema(), mode='positional_only')
    param_b = core_schema.arguments_parameter(name='b', schema=core_schema.bool_schema(), mode='positional_only')
    schema = core_schema.arguments_schema(param_a, param_b)
    v = SchemaValidator(schema)
    v.validate_python({'__args__': ('hello', True), '__kwargs__': {}})
    ```
    """
    return dict_not_none(
        type='arguments',
        arguments_schema=arguments,
        populate_by_name=populate_by_name,
        var_args_schema=var_args_schema,
        var_kwargs_schema=var_kwargs_schema,
        ref=ref,
        extra=extra,
    )


class CallSchema(TypedDict, total=False):
    type: Required[Literal['call']]
    arguments_schema: Required[CoreSchema]
    function: Required[Callable[..., Any]]
    return_schema: CoreSchema
    ref: str
    extra: Any


def call_schema(
    arguments: CoreSchema,
    function: Callable[..., Any],
    *,
    return_schema: CoreSchema | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> CallSchema:
    """Returns a schema that matches an arguments schema, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    param_a = core_schema.arguments_parameter(name='a', schema=core_schema.string_schema(), mode='positional_only')
    param_b = core_schema.arguments_parameter(name='b', schema=core_schema.bool_schema(), mode='positional_only')
    args_schema = core_schema.arguments_schema(param_a, param_b)

    schema = core_schema.call_schema(
        arguments=args_schema,
        function=lambda a, b: a + str(not b),
        return_schema=core_schema.string_schema(),
    )
    v = SchemaValidator(schema)
    assert v.validate_python((('hello', True))) == 'helloFalse'
    ```
    """
    return dict_not_none(
        type='call', arguments_schema=arguments, function=function, return_schema=return_schema, ref=ref, extra=extra
    )


class RecursiveReferenceSchema(TypedDict, total=False):
    type: Required[Literal['recursive-ref']]
    schema_ref: Required[str]


def recursive_reference_schema(schema_ref: str) -> RecursiveReferenceSchema:
    """Returns a schema that matches a recursive reference value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema_recursive = core_schema.recursive_reference_schema('list-schema')
    schema = core_schema.list_schema(items_schema=schema_recursive, ref='list-schema')
    v = SchemaValidator(schema)
    v.validate_python([[]])
    ```
    """
    return {'type': 'recursive-ref', 'schema_ref': schema_ref}


class CustomErrorSchema(TypedDict, total=False):
    type: Required[Literal['custom_error']]
    schema: Required[CoreSchema]
    custom_error_type: Required[str]
    custom_error_message: str
    custom_error_context: Dict[str, Union[str, int, float]]
    ref: str
    extra: Any


def custom_error_schema(
    schema: CoreSchema,
    custom_error_type: str,
    *,
    custom_error_message: str | None = None,
    custom_error_context: dict[str, str | int | float] | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> CustomErrorSchema:
    """Returns a schema that matches a custom error value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.custom_error_schema(
        schema=core_schema.int_schema(), custom_error_type='MyError', custom_error_message="Error msg"
    )
    v = SchemaValidator(schema)
    v.validate_python(1)
    ```
    """
    return dict_not_none(
        type='custom_error',
        schema=schema,
        custom_error_type=custom_error_type,
        custom_error_message=custom_error_message,
        custom_error_context=custom_error_context,
        ref=ref,
        extra=extra,
    )


class JsonSchema(TypedDict, total=False):
    type: Required[Literal['json']]
    schema: CoreSchema
    ref: str
    extra: Any


def json_schema(schema: CoreSchema | None = None, *, ref: str | None = None, extra: Any = None) -> JsonSchema:
    """Returns a schema that matches a JSON value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    dict_schema = core_schema.typed_dict_schema(
        {
            'field_a': core_schema.typed_dict_field(core_schema.string_schema()),
            'field_b': core_schema.typed_dict_field(core_schema.bool_schema()),
        }
    )

    class MyModel:
        __slots__ = '__dict__', '__fields_set__'
        field_a: str
        field_b: bool

    json_schema = core_schema.json_schema(schema=dict_schema)
    schema = core_schema.new_class_schema(cls=MyModel, schema=json_schema)
    v = SchemaValidator(schema)
    m = v.validate_python('{"field_a": "hello", "field_b": true}')
    assert isinstance(m, MyModel)
    ```
    """
    return dict_not_none(type='json', schema=schema, ref=ref, extra=extra)


class UrlSchema(TypedDict, total=False):
    type: Required[Literal['url']]
    max_length: int
    allowed_schemes: List[str]
    host_required: bool  # default False
    default_host: str
    default_port: int
    default_path: str
    strict: bool
    ref: str
    extra: Any


def url_schema(
    *,
    max_length: int | None = None,
    allowed_schemes: list[str] | None = None,
    host_required: bool | None = None,
    default_host: str | None = None,
    default_port: int | None = None,
    default_path: str | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> UrlSchema:
    """Returns a schema that matches a URL value, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.url_schema()
    v = SchemaValidator(schema)
    v.validate_python('https://example.com')
    ```
    """
    return dict_not_none(
        type='url',
        max_length=max_length,
        allowed_schemes=allowed_schemes,
        host_required=host_required,
        default_host=default_host,
        default_port=default_port,
        default_path=default_path,
        strict=strict,
        ref=ref,
        extra=extra,
    )


class MultiHostUrlSchema(TypedDict, total=False):
    type: Required[Literal['multi-host-url']]
    max_length: int
    allowed_schemes: List[str]
    host_required: bool  # default False
    default_host: str
    default_port: int
    default_path: str
    strict: bool
    ref: str
    extra: Any


def multi_host_url_schema(
    *,
    max_length: int | None = None,
    allowed_schemes: list[str] | None = None,
    host_required: bool | None = None,
    default_host: str | None = None,
    default_port: int | None = None,
    default_path: str | None = None,
    strict: bool | None = None,
    ref: str | None = None,
    extra: Any = None,
) -> MultiHostUrlSchema:
    """Returns a schema that matches a URL value with possibly multiple hosts, e.g.:

    ```python
    from pydantic_core import SchemaValidator, core_schema
    schema = core_schema.multi_host_url_schema()
    v = SchemaValidator(schema)
    v.validate_python('redis://localhost,0.0.0.0,127.0.0.1')
    ```
    """
    return dict_not_none(
        type='multi-host-url',
        max_length=max_length,
        allowed_schemes=allowed_schemes,
        host_required=host_required,
        default_host=default_host,
        default_port=default_port,
        default_path=default_path,
        strict=strict,
        ref=ref,
        extra=extra,
    )


CoreSchema = Union[
    AnySchema,
    NoneSchema,
    BoolSchema,
    IntSchema,
    FloatSchema,
    StringSchema,
    BytesSchema,
    DateSchema,
    TimeSchema,
    DatetimeSchema,
    TimedeltaSchema,
    LiteralSchema,
    IsInstanceSchema,
    IsSubclassSchema,
    CallableSchema,
    ListSchema,
    TuplePositionalSchema,
    TupleVariableSchema,
    SetSchema,
    FrozenSetSchema,
    GeneratorSchema,
    DictSchema,
    FunctionSchema,
    FunctionWrapSchema,
    FunctionPlainSchema,
    WithDefaultSchema,
    NullableSchema,
    UnionSchema,
    TaggedUnionSchema,
    ChainSchema,
    LaxOrStrictSchema,
    TypedDictSchema,
    NewClassSchema,
    ArgumentsSchema,
    CallSchema,
    RecursiveReferenceSchema,
    CustomErrorSchema,
    JsonSchema,
    UrlSchema,
    MultiHostUrlSchema,
]

# used in _pydantic_core.pyi::PydanticKnownError
# to update this, call `pytest -k test_all_errors` and copy the output
ErrorType = Literal[
    'json_invalid',
    'json_type',
    'recursion_loop',
    'dict_attributes_type',
    'missing',
    'frozen',
    'extra_forbidden',
    'invalid_key',
    'get_attribute_error',
    'model_class_type',
    'none_required',
    'bool',
    'greater_than',
    'greater_than_equal',
    'less_than',
    'less_than_equal',
    'multiple_of',
    'finite_number',
    'too_short',
    'too_long',
    'iterable_type',
    'iteration_error',
    'string_type',
    'string_sub_type',
    'string_unicode',
    'string_too_short',
    'string_too_long',
    'string_pattern_mismatch',
    'dict_type',
    'mapping_type',
    'list_type',
    'tuple_type',
    'set_type',
    'bool_type',
    'bool_parsing',
    'int_type',
    'int_parsing',
    'int_from_float',
    'float_type',
    'float_parsing',
    'bytes_type',
    'bytes_too_short',
    'bytes_too_long',
    'value_error',
    'assertion_error',
    'literal_error',
    'date_type',
    'date_parsing',
    'date_from_datetime_parsing',
    'date_from_datetime_inexact',
    'date_past',
    'date_future',
    'time_type',
    'time_parsing',
    'datetime_type',
    'datetime_parsing',
    'datetime_object_invalid',
    'datetime_past',
    'datetime_future',
    'datetime_aware',
    'datetime_naive',
    'time_delta_type',
    'time_delta_parsing',
    'frozen_set_type',
    'is_instance_of',
    'is_subclass_of',
    'callable_type',
    'union_tag_invalid',
    'union_tag_not_found',
    'arguments_type',
    'positional_arguments_type',
    'keyword_arguments_type',
    'unexpected_keyword_argument',
    'missing_keyword_argument',
    'unexpected_positional_argument',
    'missing_positional_argument',
    'multiple_argument_values',
    'url_type',
    'url_parsing',
    'url_syntax_violation',
    'url_too_long',
    'url_scheme',
]
