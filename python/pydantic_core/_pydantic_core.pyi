from __future__ import annotations

import datetime
import decimal
import sys
from typing import Any, Callable, Generic, Optional, Type, TypeVar

from pydantic_core import ErrorDetails, ErrorTypeInfo, InitErrorDetails, MultiHostHost
from pydantic_core.core_schema import CoreConfig, CoreSchema, ErrorType

if sys.version_info < (3, 8):
    from typing_extensions import final
else:
    from typing import final

if sys.version_info < (3, 11):
    from typing_extensions import Literal, LiteralString, Self, TypeAlias
else:
    from typing import Literal, LiteralString, Self, TypeAlias

from _typeshed import SupportsAllComparisons

__all__ = [
    '__version__',
    'build_profile',
    'build_info',
    '_recursion_limit',
    'ArgsKwargs',
    'SchemaValidator',
    'SchemaSerializer',
    'Url',
    'MultiHostUrl',
    'SchemaError',
    'ValidationError',
    'PydanticCustomError',
    'PydanticKnownError',
    'PydanticOmit',
    'PydanticUseDefault',
    'PydanticSerializationError',
    'PydanticSerializationUnexpectedValue',
    'PydanticUndefined',
    'PydanticUndefinedType',
    'Some',
    'to_json',
    'to_jsonable_python',
    'list_all_errors',
    'TzInfo',
]
__version__: str
build_profile: str
build_info: str
_recursion_limit: int

_T = TypeVar('_T', default=Any, covariant=True)

@final
class Some(Generic[_T]):
    """
    Similar to Rust's [`Option::Some`](https://doc.rust-lang.org/std/option/enum.Option.html) type, this
    identifies a value as being present, and provides a way to access it.

    Generally used in a union with `None` to different between "some value which could be None" and no value.
    """

    __match_args__ = ('value',)

    @property
    def value(self) -> _T:
        """
        Returns the value wrapped by `Some`.
        """
    @classmethod
    def __class_getitem__(cls, __item: Any) -> Type[Self]: ...

@final
class SchemaValidator:
    """
    `SchemaValidator` is the Python wrapper for `pydantic-core`'s Rust validation logic, internally it owns one
    `CombinedValidator` which may in turn own more `CombinedValidator`s which make up the full schema validator.
    """

    def __new__(cls, schema: CoreSchema, config: CoreConfig | None = None) -> Self:
        """
        Create a new SchemaValidator.

        Arguments:
            schema: The [`CoreSchema`][pydantic_core.core_schema.CoreSchema] to use for validation.
            config: Optionally a [`CoreConfig`][pydantic_core.core_schema.CoreConfig] to configure validation.
        """
    @property
    def title(self) -> str:
        """
        The title of the schema, as used in the heading of [`ValidationError.__str__()`][pydantic_core.ValidationError].
        """
    def validate_python(
        self,
        input: Any,
        *,
        strict: bool | None = None,
        from_attributes: bool | None = None,
        context: 'dict[str, Any] | None' = None,
        self_instance: Any | None = None,
    ) -> Any:
        """
        Validate a Python object against the schema and return the validated object.

        Arguments:
            input: The Python object to validate.
            strict: Whether to validate the object in strict mode.
                If `None`, the value of [`CoreConfig.strict`][pydantic_core.core_schema.CoreConfig] is used.
            from_attributes: Whether to validate objects as inputs to models by extracting attributes.
                If `None`, the value of [`CoreConfig.from_attributes`][pydantic_core.core_schema.CoreConfig] is used.
            context: The context to use for validation, this is passed to functional validators as
                [`info.context`][pydantic_core.core_schema.ValidationInfo.context].
            self_instance: An instance of a model set attributes on from validation, this is used when running
                validation from the `__init__` method of a model.

        Raises:
            ValidationError: If validation fails.
            Exception: Other error types maybe raised if internal errors occur.

        Returns:
            The validated object.
        """
    def isinstance_python(
        self,
        input: Any,
        *,
        strict: bool | None = None,
        from_attributes: bool | None = None,
        context: 'dict[str, Any] | None' = None,
        self_instance: Any | None = None,
    ) -> bool:
        """
        Similar to [`validate_python()`][pydantic_core.SchemaValidator.validate_python] but returns a boolean.

        Arguments match `validate_python()`. This method will not raise `ValidationError`s but will raise internal
        errors.

        Returns:
            `True` if validation succeeds, `False` if validation fails.
        """
    def validate_json(
        self,
        input: str | bytes | bytearray,
        *,
        strict: bool | None = None,
        context: 'dict[str, Any] | None' = None,
        self_instance: Any | None = None,
    ) -> Any:
        """
        Validate JSON data directly against the schema and return the validated Python object.

        This method should be significantly faster than `validate_python(json.loads(json_data))` as it avoids the
        need to create intermediate Python objects

        It also handles constructing the correct Python type even in strict mode, where
        `validate_python(json.loads(json_data))` would fail validation.

        Arguments:
            input: The JSON data to validate.
            strict: Whether to validate the object in strict mode.
                If `None`, the value of [`CoreConfig.strict`][pydantic_core.core_schema.CoreConfig] is used.
            context: The context to use for validation, this is passed to functional validators as
                [`info.context`][pydantic_core.core_schema.ValidationInfo.context].
            self_instance: An instance of a model set attributes on from validation.

        Raises:
            ValidationError: If validation fails or if the JSON data is invalid.
            Exception: Other error types maybe raised if internal errors occur.

        Returns:
            The validated Python object.
        """
    def validate_assignment(
        self,
        obj: Any,
        field_name: str,
        field_value: Any,
        *,
        strict: bool | None = None,
        from_attributes: bool | None = None,
        context: 'dict[str, Any] | None' = None,
    ) -> dict[str, Any] | tuple[dict[str, Any], dict[str, Any] | None, set[str]]:
        """
        Validate an assignment to a field on a model.

        Arguments:
            obj: The model instance being assigned to.
            field_name: The name of the field to validate assignment for.
            field_value: The value to assign to the field.
            strict: Whether to validate the object in strict mode.
                If `None`, the value of [`CoreConfig.strict`][pydantic_core.core_schema.CoreConfig] is used.
            from_attributes: Whether to validate objects as inputs to models by extracting attributes.
                If `None`, the value of [`CoreConfig.from_attributes`][pydantic_core.core_schema.CoreConfig] is used.
            context: The context to use for validation, this is passed to functional validators as
                [`info.context`][pydantic_core.core_schema.ValidationInfo.context].

        Raises:
            ValidationError: If validation fails.
            Exception: Other error types maybe raised if internal errors occur.

        Returns:
            Either the model dict or a tuple of `(model_data, model_extra, fields_set)`
        """
    def get_default_value(self, *, strict: bool | None = None, context: Any = None) -> Some | None:
        """
        Get the default value for the schema, including running default value validation.

        Arguments:
            strict: Whether to validate the default value in strict mode.
                If `None`, the value of [`CoreConfig.strict`][pydantic_core.core_schema.CoreConfig] is used.
            context: The context to use for validation, this is passed to functional validators as
                [`info.context`][pydantic_core.core_schema.ValidationInfo.context].

        Raises:
            ValidationError: If validation fails.
            Exception: Other error types maybe raised if internal errors occur.

        Returns:
            `None` if the schema has no default value, otherwise a [`Some`][pydantic_core.Some] containing the default.
        """

_IncEx: TypeAlias = set[int] | set[str] | dict[int, _IncEx] | dict[str, _IncEx] | None

@final
class SchemaSerializer:
    """
    `SchemaSerializer` is the Python wrapper for `pydantic-core`'s Rust serialization logic, internally it owns one
    `CombinedSerializer` which may in turn own more `CombinedSerializer`s which make up the full schema serializer.
    """
    def __new__(cls, schema: CoreSchema, config: CoreConfig | None = None) -> Self:
        """
        Create a new SchemaSerializer.

        Arguments:
            schema: The [`CoreSchema`][pydantic_core.core_schema.CoreSchema] to use for serialization.
            config: Optionally a [`CoreConfig`][pydantic_core.core_schema.CoreConfig] to to configure serialization.
        """
    def to_python(
        self,
        value: Any,
        *,
        mode: str | None = None,
        include: _IncEx = None,
        exclude: _IncEx = None,
        by_alias: bool = True,
        exclude_unset: bool = False,
        exclude_defaults: bool = False,
        exclude_none: bool = False,
        round_trip: bool = False,
        warnings: bool = True,
        fallback: Callable[[Any], Any] | None = None,
    ) -> Any:
        """
        Serialize a Python object to a Python object including transforming and filtering data.

        Arguments:
            value: The Python object to serialize.
            mode: The serialization mode to use, either `'python'` or `'json'`, defaults to `'python'`. In JSON mode,
                all values are converted to JSON compatible types, e.g. `None`, `int`, `float`, `str`, `list`, `dict`.
            include: A set of fields to include, if `None` all fields are included.
            exclude: A set of fields to exclude, if `None` no fields are excluded.
            by_alias: Whether to use the alias names of fields.
            exclude_unset: Whether to exclude fields that are not set,
                e.g. are not included in `__pydantic_fields_set__`.
            exclude_defaults: Whether to exclude fields that are equal to their default value.
            exclude_none: Whether to exclude fields that have a value of `None`.
            round_trip: Whether to enable serialization and validation round-trip support.
            warnings: Whether to log warnings when invalid fields are encountered.
            fallback: A function to call when an unknown value is encountered,
                if `None` a [`PydanticSerializationError`][pydantic_core.PydanticSerializationError] error is raised.

        Raises:
            PydanticSerializationError: If serialization fails and no `fallback` function is provided.

        Returns:
            The serialized Python object.
        """
    def to_json(
        self,
        value: Any,
        *,
        indent: int | None = None,
        include: _IncEx = None,
        exclude: _IncEx = None,
        by_alias: bool = True,
        exclude_unset: bool = False,
        exclude_defaults: bool = False,
        exclude_none: bool = False,
        round_trip: bool = False,
        warnings: bool = True,
        fallback: Callable[[Any], Any] | None = None,
    ) -> bytes:
        """
        Serialize a Python object to JSON including transforming and filtering data.

        Arguments:
            value: The Python object to serialize.
            indent: If `None`, the JSON will be compact, otherwise it will be pretty-printed with the indent provided.
            include: A set of fields to include, if `None` all fields are included.
            exclude: A set of fields to exclude, if `None` no fields are excluded.
            by_alias: Whether to use the alias names of fields.
            exclude_unset: Whether to exclude fields that are not set,
                e.g. are not included in `__pydantic_fields_set__`.
            exclude_defaults: Whether to exclude fields that are equal to their default value.
            exclude_none: Whether to exclude fields that have a value of `None`.
            round_trip: Whether to enable serialization and validation round-trip support.
            warnings: Whether to log warnings when invalid fields are encountered.
            fallback: A function to call when an unknown value is encountered,
                if `None` a [`PydanticSerializationError`][pydantic_core.PydanticSerializationError] error is raised.

        Raises:
            PydanticSerializationError: If serialization fails and no `fallback` function is provided.

        Returns:
           JSON bytes.
        """

def to_json(
    value: Any,
    *,
    indent: int | None = None,
    include: _IncEx = None,
    exclude: _IncEx = None,
    by_alias: bool = True,
    exclude_none: bool = False,
    round_trip: bool = False,
    timedelta_mode: Literal['iso8601', 'float'] = 'iso8601',
    bytes_mode: Literal['utf8', 'base64'] = 'utf8',
    serialize_unknown: bool = False,
    fallback: Callable[[Any], Any] | None = None,
) -> bytes: ...
def to_jsonable_python(
    value: Any,
    *,
    include: _IncEx = None,
    exclude: _IncEx = None,
    by_alias: bool = True,
    exclude_none: bool = False,
    round_trip: bool = False,
    timedelta_mode: Literal['iso8601', 'float'] = 'iso8601',
    bytes_mode: Literal['utf8', 'base64'] = 'utf8',
    serialize_unknown: bool = False,
    fallback: Callable[[Any], Any] | None = None,
) -> Any: ...

class Url(SupportsAllComparisons):
    def __new__(cls, url: str) -> Self: ...
    @property
    def scheme(self) -> str: ...
    @property
    def username(self) -> str | None: ...
    @property
    def password(self) -> str | None: ...
    @property
    def host(self) -> str | None: ...
    @property
    def port(self) -> int | None: ...
    @property
    def path(self) -> str | None: ...
    @property
    def query(self) -> str | None: ...
    @property
    def fragment(self) -> str | None: ...
    def unicode_host(self) -> str | None: ...
    def query_params(self) -> list[tuple[str, str]]: ...
    def unicode_string(self) -> str: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __deepcopy__(self, memo: dict) -> str: ...
    @classmethod
    def build(
        cls,
        *,
        scheme: str,
        username: Optional[str] = None,
        password: Optional[str] = None,
        host: str,
        port: Optional[int] = None,
        path: Optional[str] = None,
        query: Optional[str] = None,
        fragment: Optional[str] = None,
    ) -> Self: ...

class MultiHostUrl(SupportsAllComparisons):
    def __new__(cls, url: str) -> Self: ...
    @property
    def scheme(self) -> str: ...
    @property
    def path(self) -> str | None: ...
    @property
    def query(self) -> str | None: ...
    @property
    def fragment(self) -> str | None: ...
    def hosts(self) -> list[MultiHostHost]: ...
    def query_params(self) -> list[tuple[str, str]]: ...
    def unicode_string(self) -> str: ...
    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __deepcopy__(self, memo: dict) -> Self: ...
    @classmethod
    def build(
        cls,
        *,
        scheme: str,
        username: Optional[str] = None,
        password: Optional[str] = None,
        host: Optional[str] = None,
        hosts: Optional[list[MultiHostHost]] = None,
        port: Optional[int] = None,
        path: Optional[str] = None,
        query: Optional[str] = None,
        fragment: Optional[str] = None,
    ) -> Self: ...

@final
class SchemaError(Exception):
    def error_count(self) -> int: ...
    def errors(self) -> list[ErrorDetails]: ...

@final
class ValidationError(ValueError):
    """
    `ValidationError` is the exception raised by `pydantic-core` when validation fails, it contains a list of errors
    which detail why validation failed.
    """

    @staticmethod
    def from_exception_data(
        title: str,
        line_errors: list[InitErrorDetails],
        error_mode: Literal['python', 'json'] = 'python',
        hide_input: bool = False,
    ) -> ValidationError:
        """
        Python constructor for a Validation Error.

        The API for constructing validation errors will probably change in future,
        hence the static method rather than `__init__`.

        Arguments:
            title: The title of the error, as used in the heading of `str(validation_error`
            line_errors: A list of [`InitErrorDetails`][pydantic_core.InitErrorDetails] which contain information
                about errors that occurred during validation.
            error_mode: Whether the error is for a Python object or JSON.
            hide_input: Whether to hide the input value in the error message.
        """
    @property
    def title(self) -> str: ...
    def error_count(self) -> int: ...
    def errors(self, *, include_url: bool = True, include_context: bool = True) -> list[ErrorDetails]: ...
    def json(self, *, indent: int | None = None, include_url: bool = True, include_context: bool = True) -> str: ...

@final
class PydanticCustomError(ValueError):
    def __new__(
        cls, error_type: LiteralString, message_template: LiteralString, context: dict[str, Any] | None = None
    ) -> Self: ...
    @property
    def context(self) -> dict[str, Any] | None: ...
    @property
    def type(self) -> str: ...
    @property
    def message_template(self) -> str: ...
    def message(self) -> str: ...

@final
class PydanticKnownError(ValueError):
    def __new__(
        cls, error_type: ErrorType, context: dict[str, str | int | float | decimal.Decimal] | None = None
    ) -> Self: ...
    @property
    def context(self) -> dict[str, str | int | float] | None: ...
    @property
    def type(self) -> ErrorType: ...
    @property
    def message_template(self) -> str: ...
    def message(self) -> str: ...

@final
class PydanticOmit(Exception):
    def __new__(cls) -> Self: ...

@final
class PydanticUseDefault(Exception):
    def __new__(cls) -> Self: ...

@final
class PydanticSerializationError(ValueError):
    def __new__(cls, message: str) -> Self: ...

@final
class PydanticSerializationUnexpectedValue(ValueError):
    def __new__(cls, message: str | None = None) -> Self: ...

@final
class ArgsKwargs:
    def __new__(cls, args: tuple[Any, ...], kwargs: dict[str, Any] | None = None) -> Self: ...
    @property
    def args(self) -> tuple[Any, ...]: ...
    @property
    def kwargs(self) -> dict[str, Any] | None: ...

@final
class PydanticUndefinedType:
    def __copy__(self) -> Self: ...
    def __deepcopy__(self, memo: Any) -> Self: ...

PydanticUndefined: PydanticUndefinedType

def list_all_errors() -> list[ErrorTypeInfo]:
    """
    Get information about all built-in errors.
    """

@final
class TzInfo(datetime.tzinfo):
    def tzname(self, _dt: datetime.datetime | None) -> str | None: ...
    def utcoffset(self, _dt: datetime.datetime | None) -> datetime.timedelta: ...
    def dst(self, _dt: datetime.datetime | None) -> datetime.timedelta: ...
    def __deepcopy__(self, _memo: dict[Any, Any]) -> 'TzInfo': ...
