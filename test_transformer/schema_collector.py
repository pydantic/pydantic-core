import ast
from dataclasses import dataclass
from typing import Optional, List
import json


from pydantic_core import SchemaValidator, CoreConfig
from pydantic_core import core_schema as cs
from test_transformer.core_schema_gen import convert_schema, config_to_CoreConfig, ASTClass


@dataclass
class LineRange:
    start: int
    end: int | None = None

@dataclass
class CharRange:
    start: int
    end: int

@dataclass
class SchemaValidatorCall:
    schema: dict | ASTClass
    config: dict | ASTClass | None
    lines: LineRange
    char_range: CharRange
    context: str  # The function/method name where this was found
    assigned_to: str | None  # The variable this was assigned to

@dataclass
class ParametrizedConfig:
    config: dict
    lines: LineRange
    char_range: CharRange

@dataclass
class ParametrizedSchema:
    schema: dict
    lines: LineRange
    char_range: CharRange

class SchemaValidatorExtractor(ast.NodeVisitor):
    def __init__(self, source):
        self.validators = []
        self.current_function = None
        self.assigned_to = None
        self.source = source

    def visit_FunctionDef(self, node):
        old_function = self.current_function
        self.current_function = node.name

        # Add decorator handling
        for decorator in node.decorator_list:
            if isinstance(decorator, ast.Call):
                # Check if it's pytest.mark.parametrize
                if (isinstance(decorator.func, ast.Attribute) 
                    and decorator.func.attr == 'parametrize'
                    and isinstance(decorator.func.value, ast.Attribute)
                    and decorator.func.value.attr == 'mark'):
                    self._handle_parametrize_decorator(decorator, node)

        self.generic_visit(node)
        self.current_function = old_function

    def visit_Assign(self, node):
        # Check if the value being assigned is a function
        if isinstance(node.value, (ast.Call)):
            # node.targets[0] is the variable being assigned to
            if isinstance(node.targets[0], ast.Name) and getattr(node.value.func, 'id', None) == 'SchemaValidator':
                self.assigned_to = node.targets[0].id

        self.generic_visit(node)

    def visit_Call(self, node):
        # Check if this is a SchemaValidator instantiation
        if isinstance(node.func, ast.Name) and node.func.id == 'SchemaValidator':
            schema_dict = None
            config_dict = None

            # Extract schema (first argument)
            if len(node.args) > 0:
                schema_dict = self._eval_dict_literal(node.args[0])

            # Extract config (second argument)
            if len(node.args) > 1:
                config_dict = self._eval_dict_literal(node.args[1])

            if schema_dict is not None:
                self.validators.append(
                    SchemaValidatorCall(
                        schema=schema_dict,
                        config=config_dict,
                        lines=LineRange(node.lineno, node.end_lineno),
                        char_range=CharRange(node.col_offset, node.end_col_offset or 0),
                        context=self.current_function or '<module>',
                        assigned_to=self.assigned_to,
                    )
                )

        self.generic_visit(node)

    def _handle_parametrize_decorator(self, decorator, func_node):
        # Extract parameter names and values
        if len(decorator.args) >= 2:
            param_names = self._eval_literal(decorator.args[0])  # Could be string or tuple of strings
            param_values = self._eval_literal(decorator.args[1]) # List of tuples/values
            
            if not isinstance(param_values, (list, tuple)) or not isinstance(param_names, str):
                return

            if isinstance(param_names, str):
                param_names = param_names.split(',')
                
            # Map parameter names to their indices
            param_indices = {name.strip(): idx for idx, name in enumerate(param_names)}
            
            # Look for 'config' and schema-related parameters
            config_idx = param_indices.get('config')
            schema_idx = [idx for idx, name in enumerate(param_names) if 'schema' in name]
            
            # For each test case in param_values, extract config and schema
            for i, test_case in enumerate(param_values):
                if not isinstance(test_case, tuple):
                    continue
                if config_idx is not None:
                    config = test_case[config_idx]
                    ast_obj = decorator.args[1].elts[i].elts[config_idx]
                    if isinstance(config, dict):
                        self.validators.append(
                            ParametrizedConfig(
                                config=config,
                                lines=LineRange(ast_obj.lineno, ast_obj.end_lineno),
                                char_range=CharRange(ast_obj.col_offset, ast_obj.end_col_offset)
                            )
                        )
                for idx in schema_idx:
                    schema = test_case[idx]
                    if isinstance(schema, dict):
                        ast_obj = decorator.args[1].elts[i].elts[idx]
                        self.validators.append(
                            ParametrizedSchema(
                                schema=schema,
                                lines=LineRange(ast_obj.lineno, ast_obj.end_lineno),
                                char_range=CharRange(ast_obj.col_offset, ast_obj.end_col_offset)
                            )
                        )




    def _eval_dict_literal(self, node) -> dict | ASTClass | None:
        """Evaluate a dictionary literal AST node to a Python dict."""
        if isinstance(node, ast.Dict):
            result = {}
            for k, v in zip(node.keys, node.values):
                key = self._eval_literal(k)
                value = self._eval_literal(v)
                if key is not None and value is not None:
                    result[key] = value
            return result
        if isinstance(node, ast.Name):
            return ASTClass(name=node.id)
        return None

    def _eval_literal(self, node):
        """Evaluate a literal AST node to its Python value."""
        if isinstance(node, ast.List):
            return [self._eval_literal(item) for item in node.elts]
        elif isinstance(node, ast.Tuple):
            return tuple(self._eval_literal(item) for item in node.elts)
        if isinstance(node, ast.Constant):
            return node.s
        elif isinstance(node, ast.Dict):
            return self._eval_dict_literal(node)
        elif isinstance(node, ast.Name):
            # Handle common constants
            if node.id == 'True':
                return True
            elif node.id == 'False':
                return False
            elif node.id == 'None':
                return None
            else:
                return ASTClass(name=node.id)
        return None


def extract_validators_from_file(file_path: str) -> List[SchemaValidatorCall]:
    """Extract all SchemaValidator instantiations from a Python file."""
    with open(file_path, 'r') as f:
        source = f.read()
        tree = ast.parse(source)

    extractor = SchemaValidatorExtractor(source)
    extractor.visit(tree)
    return extractor.validators


def format_validator_call(validator: SchemaValidatorCall) -> str:
    """Format a SchemaValidator call into a readable string."""
    parts = [
        f'Lines {validator.lines.start}:{validator.lines.end} in {validator.context}:',
        'SchemaValidator(',
        f'    schema={json.dumps(validator.schema, indent=4, default=str)}',
    ]

    if isinstance(validator.config, dict):
        parts.append(f'    config={json.dumps(validator.config, indent=4)}')
    elif isinstance(validator.config, ASTClass):
        parts.append(f'    config={validator.config.name}')

    parts.append(')')
    return '\n'.join(parts)
