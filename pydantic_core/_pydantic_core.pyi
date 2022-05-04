from typing import Any, Dict, List

__version__: str

class SchemaValidator:
    def __init__(self, schema: Dict[str, Any]) -> None: ...
    def validate_python(self, input: Any) -> Any: ...
    def validate_json(self, input: str) -> Any: ...
    def validate_assignment(self, field: str, input: Any, data: Dict[str, Any]) -> Dict[str, Any]: ...
    @property
    def _schema(self) -> Dict[str, Any]: ...

class SchemaError(ValueError):
    pass

class ValidationError(ValueError):
    model_name: str

    def error_count(self) -> int: ...
    def errors(self) -> List[Dict[str, Any]]: ...
