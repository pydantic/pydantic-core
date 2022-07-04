from typing import Any, Dict, List, Optional, Union

from pydantic_core._types import Config, Schema

__version__: str

class SchemaValidator:
    def __init__(self, schema: Schema, config: Optional[Config] = None) -> None: ...
    def validate_python(self, input: Any) -> Any: ...
    def isinstance_python(self, input: Any) -> bool: ...
    def validate_json(self, input: Union[str, bytes, bytearray]) -> Any: ...
    def isinstance_json(self, input: Union[str, bytes, bytearray]) -> bool: ...
    def validate_assignment(self, field: str, input: Any, data: Dict[str, Any]) -> Dict[str, Any]: ...

class SchemaError(ValueError):
    pass

class ValidationError(ValueError):
    title: str

    def error_count(self) -> int: ...
    def errors(self) -> List[Dict[str, Any]]: ...
