"""
Tools to process a CoreSchema
"""
from __future__ import annotations as _annotations

from collections import defaultdict
from typing import Callable, TypeVar, Union, cast

from typing_extensions import TypeGuard, get_args

from pydantic_core import core_schema as cs

T = TypeVar('T')


def is_definition_ref_schema(s: cs.CoreSchema) -> TypeGuard[cs.DefinitionReferenceSchema]:
    return s['type'] == 'definition-ref'


def is_definitions_schema(s: cs.CoreSchema) -> TypeGuard[cs.DefinitionsSchema]:
    return s['type'] == 'definitions'


AnyFunctionSchema = Union[
    cs.AfterValidatorFunctionSchema,
    cs.BeforeValidatorFunctionSchema,
    cs.WrapValidatorFunctionSchema,
    cs.PlainValidatorFunctionSchema,
]

FunctionSchemaWithInnerSchema = Union[
    cs.AfterValidatorFunctionSchema, cs.BeforeValidatorFunctionSchema, cs.WrapValidatorFunctionSchema
]


def is_function_with_inner_schema(schema: cs.CoreSchema) -> TypeGuard[FunctionSchemaWithInnerSchema]:
    return schema['type'] in ('function-before', 'function-after', 'function-wrap')


Recurse = Callable[[cs.CoreSchema, 'Walk'], cs.CoreSchema]
Walk = Callable[[cs.CoreSchema, Recurse], cs.CoreSchema]


class _WalkCoreSchema:
    def __init__(self):
        self._schema_type_to_method = self._build_schema_type_to_method()

    def _build_schema_type_to_method(self) -> dict[cs.CoreSchemaType, Recurse]:
        mapping: dict[cs.CoreSchemaType, Recurse] = {}
        key: cs.CoreSchemaType
        for key in get_args(cs.CoreSchemaType):
            method_name = f"handle_{key.replace('-', '_')}_schema"
            mapping[key] = getattr(self, method_name, self._handle_other_schemas)
        return mapping

    def walk(self, schema: cs.CoreSchema, f: Walk) -> cs.CoreSchema:
        return f(schema.copy(), self._walk)

    def _walk(self, schema: cs.CoreSchema, f: Walk) -> cs.CoreSchema:
        return self._schema_type_to_method[schema['type']](schema, f)

    def _handle_other_schemas(self, schema: cs.CoreSchema, f: Walk) -> cs.CoreSchema:
        if 'schema' in schema:
            schema['schema'] = self.walk(schema['schema'], f)  # type: ignore
        return schema

    def handle_definitions_schema(self, schema: cs.DefinitionsSchema, f: Walk) -> cs.CoreSchema:
        new_definitions: list[cs.CoreSchema] = []
        for definition in schema['definitions']:
            updated_definition = self.walk(definition, f)
            if 'ref' in updated_definition:
                # If the updated definition schema doesn't have a 'ref', it shouldn't go in the definitions
                # This is most likely to happen due to replacing something with a definition reference, in
                # which case it should certainly not go in the definitions list
                new_definitions.append(updated_definition)
        new_inner_schema = self.walk(schema['schema'], f)

        if not new_definitions and len(schema) == 3:
            # This means we'd be returning a "trivial" definitions schema that just wrapped the inner schema
            return new_inner_schema

        new_schema = schema.copy()
        new_schema['schema'] = new_inner_schema
        new_schema['definitions'] = new_definitions
        return new_schema

    def handle_list_schema(self, schema: cs.ListSchema, f: Walk) -> cs.CoreSchema:
        if 'items_schema' in schema:
            schema['items_schema'] = self.walk(schema['items_schema'], f)
        return schema

    def handle_set_schema(self, schema: cs.SetSchema, f: Walk) -> cs.CoreSchema:
        if 'items_schema' in schema:
            schema['items_schema'] = self.walk(schema['items_schema'], f)
        return schema

    def handle_frozenset_schema(self, schema: cs.FrozenSetSchema, f: Walk) -> cs.CoreSchema:
        if 'items_schema' in schema:
            schema['items_schema'] = self.walk(schema['items_schema'], f)
        return schema

    def handle_generator_schema(self, schema: cs.GeneratorSchema, f: Walk) -> cs.CoreSchema:
        if 'items_schema' in schema:
            schema['items_schema'] = self.walk(schema['items_schema'], f)
        return schema

    def handle_tuple_variable_schema(
        self, schema: cs.TupleVariableSchema | cs.TuplePositionalSchema, f: Walk
    ) -> cs.CoreSchema:
        schema = cast(cs.TupleVariableSchema, schema)
        if 'items_schema' in schema:
            schema['items_schema'] = self.walk(schema['items_schema'], f)
        return schema

    def handle_tuple_positional_schema(
        self, schema: cs.TupleVariableSchema | cs.TuplePositionalSchema, f: Walk
    ) -> cs.CoreSchema:
        schema = cast(cs.TuplePositionalSchema, schema)
        schema['items_schema'] = [self.walk(v, f) for v in schema['items_schema']]
        if 'extra_schema' in schema:
            schema['extra_schema'] = self.walk(schema['extra_schema'], f)
        return schema

    def handle_dict_schema(self, schema: cs.DictSchema, f: Walk) -> cs.CoreSchema:
        if 'keys_schema' in schema:
            schema['keys_schema'] = self.walk(schema['keys_schema'], f)
        if 'values_schema' in schema:
            schema['values_schema'] = self.walk(schema['values_schema'], f)
        return schema

    def handle_function_schema(self, schema: AnyFunctionSchema, f: Walk) -> cs.CoreSchema:
        if not is_function_with_inner_schema(schema):
            return schema
        schema['schema'] = self.walk(schema['schema'], f)
        return schema

    def handle_union_schema(self, schema: cs.UnionSchema, f: Walk) -> cs.CoreSchema:
        schema['choices'] = [self.walk(v, f) for v in schema['choices']]
        return schema

    def handle_tagged_union_schema(self, schema: cs.TaggedUnionSchema, f: Walk) -> cs.CoreSchema:
        new_choices: dict[str | int, str | int | cs.CoreSchema] = {}
        for k, v in schema['choices'].items():
            new_choices[k] = v if isinstance(v, (str, int)) else self.walk(v, f)
        schema['choices'] = new_choices
        return schema

    def handle_chain_schema(self, schema: cs.ChainSchema, f: Walk) -> cs.CoreSchema:
        schema['steps'] = [self.walk(v, f) for v in schema['steps']]
        return schema

    def handle_lax_or_strict_schema(self, schema: cs.LaxOrStrictSchema, f: Walk) -> cs.CoreSchema:
        schema['lax_schema'] = self.walk(schema['lax_schema'], f)
        schema['strict_schema'] = self.walk(schema['strict_schema'], f)
        return schema

    def handle_typed_dict_schema(self, schema: cs.TypedDictSchema, f: Walk) -> cs.CoreSchema:
        if 'extra_validator' in schema:
            schema['extra_validator'] = self.walk(schema['extra_validator'], f)
        replaced_fields: dict[str, cs.TypedDictField] = {}
        for k, v in schema['fields'].items():
            replaced_field = v.copy()
            replaced_field['schema'] = self.walk(v['schema'], f)
            replaced_fields[k] = replaced_field
        schema['fields'] = replaced_fields
        return schema

    def handle_dataclass_args_schema(self, schema: cs.DataclassArgsSchema, f: Walk) -> cs.CoreSchema:
        replaced_fields: list[cs.DataclassField] = []
        for field in schema['fields']:
            replaced_field = field.copy()
            replaced_field['schema'] = self.walk(field['schema'], f)
            replaced_fields.append(replaced_field)
        schema['fields'] = replaced_fields
        return schema

    def handle_arguments_schema(self, schema: cs.ArgumentsSchema, f: Walk) -> cs.CoreSchema:
        replaced_arguments_schema: list[cs.ArgumentsParameter] = []
        for param in schema['arguments_schema']:
            replaced_param = param.copy()
            replaced_param['schema'] = self.walk(param['schema'], f)
            replaced_arguments_schema.append(replaced_param)
        schema['arguments_schema'] = replaced_arguments_schema
        if 'var_args_schema' in schema:
            schema['var_args_schema'] = self.walk(schema['var_args_schema'], f)
        if 'var_kwargs_schema' in schema:
            schema['var_kwargs_schema'] = self.walk(schema['var_kwargs_schema'], f)
        return schema

    def handle_call_schema(self, schema: cs.CallSchema, f: Walk) -> cs.CoreSchema:
        schema['arguments_schema'] = self.walk(schema['arguments_schema'], f)
        if 'return_schema' in schema:
            schema['return_schema'] = self.walk(schema['return_schema'], f)
        return schema


_dispatch = _WalkCoreSchema().walk


def walk_core_schema(schema: cs.CoreSchema, f: Walk) -> cs.CoreSchema:
    """Recursively traverse a CoreSchema.

    Args:
        schema (cs.CoreSchema): The CoreSchema to process, it will not be modified.
        f (Walk): A function to apply. This function takes two arguments:
          1. The current CoreSchema that is being processed
             (not the same one you passed into this function, one level down).
          2. The "next" `f` to call. This lets you for example use `f=functools.partial(some_method, some_context)`
             to pass data down the recursive calls without using globals or other mutable state.

    Returns:
        cs.CoreSchema: A processed CoreSchema.
    """
    return f(schema.copy(), _dispatch)


def simplify_schema_references(schema: cs.CoreSchema) -> cs.CoreSchema:  # noqa: C901
    """
    Simplify schema references by:
      1. Grouping all definitions into a single top-level `definitions` schema, similar to a JSON schema's `#/$defs`.
      2. Inlining any definitions that are only referenced in one place and are not involved in a cycle.
      3. Removing any unused `ref` references from schemas.
    """
    all_defs: dict[str, cs.CoreSchema] = {}

    def get_ref(s: cs.CoreSchema) -> None | str:
        return s.get('ref', None)

    def collect_refs(s: cs.CoreSchema, next: Recurse) -> cs.CoreSchema:
        if s['type'] == 'definitions':
            for definition in s['definitions']:
                ref = get_ref(definition)
                assert ref is not None
                all_defs[ref] = next(definition, collect_refs).copy()
            return next(s['schema'], collect_refs)
        ref = get_ref(s)
        if ref is not None:
            all_defs[ref] = s
        return next(s, collect_refs)

    schema = walk_core_schema(schema, collect_refs)

    def flatten_refs(s: cs.CoreSchema, next: Recurse) -> cs.CoreSchema:
        if is_definitions_schema(s):
            # iterate ourselves, we don't want to flatten the actual defs!
            s['schema'] = next(s['schema'], flatten_refs)
            for definition in s['definitions']:
                next(definition, flatten_refs)  # don't re-assign here!
            return s
        s = next(s, flatten_refs)
        ref = get_ref(s)
        if ref and ref in all_defs and ref:
            all_defs[ref] = s
            return cs.definition_reference_schema(schema_ref=ref)
        return s

    schema = walk_core_schema(schema, flatten_refs)

    ref_counts: dict[str, int] = defaultdict(int)
    involved_in_recursion: dict[str, bool] = {}
    current_recursion_ref_count: dict[str, int] = defaultdict(int)

    def count_refs(s: cs.CoreSchema, next: Recurse) -> cs.CoreSchema:
        if not is_definition_ref_schema(s):
            return next(s, count_refs)
        ref = s['schema_ref']
        ref_counts[ref] += 1
        if current_recursion_ref_count[ref] != 0:
            involved_in_recursion[ref] = True
            return s

        current_recursion_ref_count[ref] += 1
        next(all_defs[ref], count_refs)
        current_recursion_ref_count[ref] -= 1
        return s

    schema = walk_core_schema(schema, count_refs)

    assert all(c == 0 for c in current_recursion_ref_count.values()), 'this is a bug! please report it'

    def inline_refs(s: cs.CoreSchema, next: Recurse) -> cs.CoreSchema:
        if s['type'] == 'definition-ref':
            ref = s['schema_ref']
            # Check if the reference is only used once and not involved in recursion
            if ref_counts[ref] <= 1 and not involved_in_recursion.get(ref, False):
                # Inline the reference by replacing the reference with the actual schema
                new = all_defs.pop(ref)
                ref_counts[ref] -= 1  # because we just replaced it!
                new.pop('ref')  # type: ignore
                s = next(new, inline_refs)
                return s
            else:
                return next(s, inline_refs)
        else:
            return next(s, inline_refs)

    schema = walk_core_schema(schema, inline_refs)

    definitions = [d for d in all_defs.values() if ref_counts[d['ref']] > 0]  # type: ignore

    if definitions:
        return cs.definitions_schema(schema=schema, definitions=definitions)
    return schema
