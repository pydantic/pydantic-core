import decimal
import sys
from typing import Any, Callable, Sequence

from pydantic_core import ErrorDetails, InitErrorDetails
from pydantic_core.core_schema import CoreConfig, CoreSchema, ErrorType

if sys.version_info < (3, 9):
    from typing_extensions import TypedDict
else:
    from typing import TypedDict

if sys.version_info < (3, 11):
    from typing_extensions import Literal, LiteralString, NotRequired, TypeAlias
else:
    from typing import Literal, LiteralString, NotRequired, TypeAlias

from _typeshed import SupportsAllComparisons

__all__ = (
    '__version__',
    'build_profile',
    'SchemaValidator',
    'SchemaSerializer',
    'Url',
    'MultiHostUrl',
    'SchemaError',
    'ValidationError',
    'PydanticCustomError',
    'PydanticKnownError',
    'PydanticOmit',
    'PydanticSerializationError',
    'list_all_errors',
)
__version__: str
build_profile: str

class SchemaValidator:
    @property
    def title(self) -> str: ...
    def __init__(self, schema: CoreSchema, config: 'CoreConfig | None' = None) -> None: ...
    def validate_python(
        self, input: Any, *, strict: 'bool | None' = None, context: Any = None, self_instance: 'Any | None' = None
    ) -> Any: ...
    def isinstance_python(
        self, input: Any, *, strict: 'bool | None' = None, context: Any = None, self_instance: 'Any | None' = None
    ) -> bool: ...
    def validate_json(
        self,
        input: 'str | bytes | bytearray',
        *,
        strict: 'bool | None' = None,
        context: Any = None,
        self_instance: 'Any | None' = None,
    ) -> Any: ...
    def isinstance_json(
        self,
        input: 'str | bytes | bytearray',
        *,
        strict: 'bool | None' = None,
        context: Any = None,
        self_instance: 'Any | None' = None,
    ) -> bool: ...
    def validate_assignment(
        self, obj: Any, field: str, input: Any, *, strict: 'bool | None' = None, context: Any = None
    ) -> 'dict[str, Any]': ...

IncEx: TypeAlias = 'set[int] | set[str] | dict[int, IncEx] | dict[str, IncEx] | None'

class SchemaSerializer:
    def __init__(self, schema: CoreSchema, config: 'CoreConfig | None' = None) -> None: ...
    def to_python(
        self,
        value: Any,
        *,
        mode: str | None = None,
        include: IncEx = None,
        exclude: IncEx = None,
        by_alias: bool = True,
        exclude_unset: bool = False,
        exclude_defaults: bool = False,
        exclude_none: bool = False,
        round_trip: bool = False,
        warnings: bool = True,
        fallback: 'Callable[[Any], Any] | None' = None,
    ) -> Any: ...
    def to_json(
        self,
        value: Any,
        *,
        indent: int | None = None,
        include: IncEx = None,
        exclude: IncEx = None,
        by_alias: bool = True,
        exclude_unset: bool = False,
        exclude_defaults: bool = False,
        exclude_none: bool = False,
        round_trip: bool = False,
        warnings: bool = True,
        fallback: 'Callable[[Any], Any] | None' = None,
    ) -> bytes: ...

def to_json(
    value: Any,
    *,
    indent: int | None = None,
    include: IncEx = None,
    exclude: IncEx = None,
    by_alias: bool = True,
    exclude_none: bool = False,
    round_trip: bool = False,
    timedelta_mode: Literal['iso8601', 'float'] = 'iso8601',
    bytes_mode: Literal['utf8', 'base64'] = 'utf8',
    serialize_unknown: bool = False,
    fallback: 'Callable[[Any], Any] | None' = None,
) -> bytes: ...
def to_jsonable_python(
    value: Any,
    *,
    include: IncEx = None,
    exclude: IncEx = None,
    by_alias: bool = True,
    exclude_none: bool = False,
    round_trip: bool = False,
    timedelta_mode: Literal['iso8601', 'float'] = 'iso8601',
    bytes_mode: Literal['utf8', 'base64'] = 'utf8',
    serialize_unknown: bool = False,
    fallback: 'Callable[[Any], Any] | None' = None,
) -> Any: ...

class Url(SupportsAllComparisons):
    @property
    def scheme(self) -> str: ...
    @property
    def username(self) -> 'str | None': ...
    @property
    def password(self) -> 'str | None': ...
    @property
    def host(self) -> 'str | None': ...
    @property
    def port(self) -> 'int | None': ...
    @property
    def path(self) -> 'str | None': ...
    @property
    def query(self) -> 'str | None': ...
    @property
    def fragment(self) -> 'str | None': ...
    def __init__(self, url: str) -> None: ...
    def unicode_host(self) -> 'str | None': ...
    def query_params(self) -> 'list[tuple[str, str]]': ...
    def unicode_string(self) -> str: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class MultiHostHost(TypedDict):
    scheme: str
    path: 'str | None'
    query: 'str | None'
    fragment: 'str | None'

class MultiHostUrl(SupportsAllComparisons):
    @property
    def scheme(self) -> str: ...
    @property
    def path(self) -> 'str | None': ...
    @property
    def query(self) -> 'str | None': ...
    @property
    def fragment(self) -> 'str | None': ...
    def __init__(self, url: str) -> None: ...
    def hosts(self) -> 'list[MultiHostHost]': ...
    def query_params(self) -> 'list[tuple[str, str]]': ...
    def unicode_string(self) -> str: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...

class SchemaError(Exception):
    def error_count(self) -> int: ...
    def errors(self) -> 'list[ErrorDetails]': ...

class ValidationError(ValueError):
    def __init__(
        self, title: str, errors: 'list[InitErrorDetails]', error_mode: Literal['python', 'json'] = 'python'
    ) -> None: ...
    @property
    def title(self) -> str: ...
    def error_count(self) -> int: ...
    def errors(self, *, include_url: bool = True, include_context: bool = True) -> 'list[ErrorDetails]': ...
    def json(self, *, indent: 'int | None' = None, include_url: bool = True, include_context: bool = False) -> str: ...

class PydanticCustomError(ValueError):
    def __init__(
        self, error_type: LiteralString, message_template: LiteralString, context: 'dict[str, Any] | None' = None
    ) -> None: ...
    @property
    def type(self) -> str: ...
    @property
    def message_template(self) -> str: ...
    context: 'dict[str, Any] | None'
    def message(self) -> str: ...

class PydanticKnownError(ValueError):
    @property
    def type(self) -> ErrorType: ...
    @property
    def message_template(self) -> str: ...
    context: 'dict[str, str | int | float] | None'

    def __init__(
        self, error_type: ErrorType, context: 'dict[str, str | int | float | decimal.Decimal] | None' = None
    ) -> None: ...
    def message(self) -> str: ...

class PydanticOmit(Exception):
    def __init__(self) -> None: ...

class PydanticSerializationError(ValueError):
    def __init__(self, message: str) -> None: ...

class PydanticSerializationUnexpectedValue(ValueError):
    def __init__(self, message: 'str | None' = None) -> None: ...

class ErrorTypeInfo(TypedDict):
    type: ErrorType
    message_template_python: str
    example_message_python: str
    message_template_json: NotRequired[str]
    example_message_json: NotRequired[str]
    example_context: 'dict[str, str | int | float] | None'

class ArgsKwargs:
    def __init__(self, args: 'tuple[Any, ...]', kwargs: 'dict[str, Any] | None' = None) -> None: ...
    @property
    def args(self) -> 'tuple[Any, ...]': ...
    @property
    def kwargs(self) -> 'dict[str, Any] | None': ...

def list_all_errors() -> 'list[ErrorTypeInfo]':
    """
    Get information about all built-in errors.
    """

ExcLeafTypes: TypeAlias = AssertionError | ValueError | 'PydanticError'

class ValidationException(Exception):
    def __new__(
        cls, __message: str, __exceptions: Sequence[ExcLeafTypes | 'ValidationException']
    ) -> ValidationException: ...
    def __init__(self, __message: str, __exceptions: Sequence[ExcLeafTypes | 'ValidationException']) -> None: ...
    @property
    def message(self) -> str: ...
    @property
    def exceptions(self) -> Sequence[ExcLeafTypes | ValidationException]: ...
    def subgroup(
        self, __condition: Callable[[ExcLeafTypes | 'ValidationException'], bool]
    ) -> ValidationException | None: ...
    def split(
        self, __condition: Callable[[ExcLeafTypes | 'ValidationException'], bool]
    ) -> tuple[ValidationException | None, ValidationException | None]: ...


class NoSuchAttributeError(Exception):
    def __new__(cls, __attribute: str) -> NoSuchAttributeError: ...
    def __init__(self, __attribute: str) -> None: ...

class JsonInvalidError(Exception):
    def __new__(cls, __error: str) -> JsonInvalidError: ...
    def __init__(self, __error: str) -> None: ...

class JsonTypeError(Exception):
    pass

class RecursionLoopError(Exception):
    pass

class DictAttributesTypeError(Exception):
    pass

class MissingError(Exception):
    pass

class FrozenFieldError(Exception):
    pass

class FrozenInstanceError(Exception):
    pass

class ExtraForbiddenError(Exception):
    pass

class InvalidKeyError(Exception):
    pass

class GetAttributeError(Exception):
    def __new__(cls, __error: str) -> GetAttributeError: ...
    def __init__(self, __error: str) -> None: ...

class ModelClassTypeError(Exception):
    def __new__(cls, __class_name: str) -> ModelClassTypeError: ...
    def __init__(self, __class_name: str) -> None: ...

class NoneRequiredError(Exception):
    pass

class BoolError(Exception):
    pass

class GreaterThanError(Exception):
    def __new__(cls, __gt: int | float | bool | str | None) -> GreaterThanError: ...
    def __init__(self, __gt: int | float | bool | str | None) -> None: ...

class GreaterThanEqualError(Exception):
    def __new__(cls, __ge: int | float | bool | str | None) -> GreaterThanEqualError: ...
    def __init__(self, __ge: int | float | bool | str | None) -> None: ...

class LessThanError(Exception):
    def __new__(cls, __lt: int | float | bool | str | None) -> LessThanError: ...
    def __init__(self, __lt: int | float | bool | str | None) -> None: ...

class LessThanEqualError(Exception):
    def __new__(cls, __le: int | float | bool | str | None) -> LessThanEqualError: ...
    def __init__(self, __le: int | float | bool | str | None) -> None: ...

class MultipleOfError(Exception):
    def __new__(cls, __multiple_of: int | float | bool | str | None) -> MultipleOfError: ...
    def __init__(self, __multiple_of: int | float | bool | str | None) -> None: ...

class FiniteNumberError(Exception):
    pass

class TooShortError(Exception):
    def __new__(cls, __field_type: str, __min_length: int, __actual_length: int) -> TooShortError: ...
    def __init__(self, __field_type: str, __min_length: int, __actual_length: int) -> None: ...

class TooLongError(Exception):
    def __new__(cls, __field_type: str, __max_length: int, __actual_length: int) -> TooLongError: ...
    def __init__(self, __field_type: str, __max_length: int, __actual_length: int) -> None: ...

class IterableTypeError(Exception):
    pass

class IterationError(Exception):
    def __new__(cls, __error: str) -> IterationError: ...
    def __init__(self, __error: str) -> None: ...

class StringTypeError(Exception):
    pass

class StringSubTypeError(Exception):
    pass

class StringUnicodeError(Exception):
    pass

class StringTooShortError(Exception):
    def __new__(cls, __min_length: int) -> StringTooShortError: ...
    def __init__(self, __min_length: int) -> None: ...

class StringTooLongError(Exception):
    def __new__(cls, __max_length: int) -> StringTooLongError: ...
    def __init__(self, __max_length: int) -> None: ...

class StringPatternMismatchError(Exception):
    def __new__(cls, __pattern: str) -> StringPatternMismatchError: ...
    def __init__(self, __pattern: str) -> None: ...

class DictTypeError(Exception):
    pass

class MappingTypeError(Exception):
    def __new__(cls, __error: str) -> MappingTypeError: ...
    def __init__(self, __error: str) -> None: ...

class ListTypeError(Exception):
    pass

class TupleTypeError(Exception):
    pass

class SetTypeError(Exception):
    pass

class BoolTypeError(Exception):
    pass

class BoolParsingError(Exception):
    pass

class IntTypeError(Exception):
    pass

class IntParsingError(Exception):
    pass

class IntFromFloatError(Exception):
    pass

class FloatTypeError(Exception):
    pass

class FloatParsingError(Exception):
    pass

class BytesTypeError(Exception):
    pass

class BytesTooShortError(Exception):
    def __new__(cls, __min_length: int) -> BytesTooShortError: ...
    def __init__(self, __min_length: int) -> None: ...

class BytesTooLongError(Exception):
    def __new__(cls, __max_length: int) -> BytesTooLongError: ...
    def __init__(self, __max_length: int) -> None: ...

class LiteralError(TypeError):
    pass


class DateTypeError(TypeError):
    pass


class DateParsingError(ValueError):
    pass


class DateFromDatetimeParsingError(ValueError):
    pass


class DateFromDatetimeInexactError(ValueError):
    pass


class DatePastError(ValueError):
    pass


class DateFutureError(ValueError):
    pass


class TimeTypeError(TypeError):
    pass


class TimeParsingError(ValueError):
    pass


class DatetimeTypeError(TypeError):
    pass


class DatetimeParsingError(ValueError):
    pass


class DatetimeObjectInvalidError(ValueError):
    pass


class DatetimePastError(ValueError):
    pass


class DatetimeFutureError(ValueError):
    pass


class DatetimeAwareError(ValueError):
    pass


class DatetimeNaiveError(ValueError):
    pass


class TimeDeltaTypeError(TypeError):
    pass


class TimeDeltaParsingError(ValueError):
    pass


class FrozenSetTypeError(TypeError):
    pass


class IsInstanceOfError(TypeError):
    pass


class IsSubclassOfError(TypeError):
    pass


class CallableTypeError(TypeError):
    pass


class UnionTagInvalidError(ValueError):
    pass


class UnionTagNotFoundError(ValueError):
    pass


class ArgumentsTypeError(TypeError):
    pass


class MissingArgumentError(TypeError):
    pass


class UnexpectedKeywordArgumentError(TypeError):
    pass


class MissingKeywordOnlyArgumentError(Exception):
    pass


class UnexpectedPositionalArgumentError(Exception):
    pass


class MissingPositionalOnlyArgumentError(Exception):
    pass


class MultipleArgumentValuesError(Exception):
    pass


class DataclassTypeError(Exception):
    def __new__(cls, __dataclass_name: str) -> DataclassTypeError: ...
    def __init__(self, __dataclass_name: str) -> None: ...
    

class UrlTypeError(Exception):
    pass


class UrlParsingError(Exception):
    def __new__(cls, __error: str) -> UrlParsingError: ...
    def __init__(self, __error: str) -> None: ...
    

class UrlSyntaxViolationError(Exception):
    def __new__(cls, __error: str) -> UrlSyntaxViolationError: ...
    def __init__(self, __error: str) -> None: ...
    

class UrlTooLongError(Exception):
    def __new__(cls, __max_length: int) -> UrlTooLongError: ...
    def __init__(self, __max_length: int) -> None: ...
    

class UrlSchemeError(Exception):
    def __new__(cls, __expected_schemes: str) -> UrlSchemeError: ...
    def __init__(self, __expected_schemes: str) -> None: ...


PydanticError = (
    NoSuchAttributeError
    | JsonInvalidError
    | JsonTypeError
    | RecursionLoopError
    | DictAttributesTypeError
    | MissingError
    | FrozenFieldError
    | FrozenInstanceError
    | ExtraForbiddenError
    | InvalidKeyError
    | GetAttributeError
    | ModelClassTypeError
    | NoneRequiredError
    | BoolError
    | GreaterThanError
    | GreaterThanEqualError
    | LessThanError
    | LessThanEqualError
    | MultipleOfError
    | FiniteNumberError
    | TooShortError
    | TooLongError
    | IterableTypeError
    | IterationError
    | StringTypeError
    | StringSubTypeError
    | StringUnicodeError
    | StringTooShortError
    | StringTooLongError
    | StringPatternMismatchError
    | DictTypeError
    | MappingTypeError
    | ListTypeError
    | TupleTypeError
    | SetTypeError
    | BoolTypeError
    | BoolParsingError
    | IntTypeError
    | IntParsingError
    | IntFromFloatError
    | FloatTypeError
    | FloatParsingError
    | BytesTypeError
    | BytesTooShortError
    | BytesTooLongError
    | LiteralError
    | DateTypeError
    | DateParsingError
    | DateFromDatetimeParsingError
    | DateFromDatetimeInexactError
    | DatePastError
    | DateFutureError
    | TimeTypeError
    | TimeParsingError
    | DatetimeTypeError
    | DatetimeParsingError
    | DatetimeObjectInvalidError
    | DatetimePastError
    | DatetimeFutureError
    | DatetimeAwareError
    | DatetimeNaiveError
    | TimeDeltaTypeError
    | TimeDeltaParsingError
    | FrozenSetTypeError
    | IsInstanceOfError
    | IsSubclassOfError
    | CallableTypeError
    | UnionTagInvalidError
    | UnionTagNotFoundError
    | ArgumentsTypeError
    | MissingArgumentError
    | UnexpectedKeywordArgumentError
    | MissingKeywordOnlyArgumentError
    | UnexpectedPositionalArgumentError
    | MissingPositionalOnlyArgumentError
    | MultipleArgumentValuesError
    | DataclassTypeError
    | UrlTypeError
    | UrlParsingError
    | UrlSyntaxViolationError
    | UrlTooLongError
    | UrlSchemeError
)
