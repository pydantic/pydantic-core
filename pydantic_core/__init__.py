from typing import Any, Callable, Tuple

from ._pydantic_core import SchemaError, SchemaValidator as SchemaValidatorBase, ValidationError, __version__

__all__ = '__version__', 'SchemaValidator', 'ValidationError', 'SchemaError'


class SchemaValidator(SchemaValidatorBase):
    def __reduce__(self) -> 'Tuple[Callable[..., SchemaValidator], Tuple[Any, ...]]':
        return (SchemaValidator, (self._schema,))
