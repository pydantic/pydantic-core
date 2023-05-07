from dataclasses import dataclass

import pytest

from pydantic_core._pydantic_core import BaseExceptionGroup


@dataclass(frozen=True)
class EqExc(Exception):
    x: int = 1


class EqError(BaseException):
    def __eq__(self, __value: object) -> bool:
        return self is __value


class EqTypeError(TypeError, EqError):
    pass


class ValueErrorTypeError(ValueError, EqError):
    pass


def test_construct() -> None:
    eg = BaseExceptionGroup('msg', [BaseException()])
    assert eg.message == 'msg'
    assert eg.__traceback__ is None
    assert eg.__cause__ is None
    assert eg.__context__ is None


def test_construct_empty() -> None:
    with pytest.raises(ValueError, match='must be a non-empty sequence'):
        BaseExceptionGroup('msg', [])


def test_construct_nest() -> None:
    eg = BaseExceptionGroup('msg', [BaseException()])
    eg = BaseExceptionGroup('outer_msg', [TypeError(), eg])


def test_eq() -> None:
    assert BaseExceptionGroup('msg', [EqExc()]) == BaseExceptionGroup('msg', [EqExc()])
    assert BaseExceptionGroup('msg', [EqExc()]) != BaseExceptionGroup('other msg', [EqExc()])
    assert BaseExceptionGroup('msg', [EqExc()]) != BaseExceptionGroup('msg', [TypeError()])
    assert BaseExceptionGroup('msg', [EqExc()]) != BaseExceptionGroup('msg', [EqExc(x=2)])


def test_exceptions_attribute() -> None:
    eg = BaseExceptionGroup('outer_msg', [EqExc(), BaseExceptionGroup('msg', [EqExc()])])
    assert eg.exceptions == [EqExc(), BaseExceptionGroup('msg', [EqExc()])]


def test_subgroup() -> None:
    te = TypeError()
    eg = BaseExceptionGroup('foo', [te, EqExc(x=2), BaseExceptionGroup('msg', [EqExc(x=1), EqExc(x=3), ValueError()])])

    new = eg.subgroup(lambda e: isinstance(e, TypeError) or isinstance(e, EqExc) and e.x > 1)

    assert new is not None
    new_excs = new.exceptions
    assert new_excs[0] is te
    assert new_excs[1:] == [EqExc(x=2), BaseExceptionGroup('msg', [EqExc(x=3)])]

    new = eg.subgroup(lambda _: False)
    assert new is None

    new = eg.subgroup(lambda e: isinstance(e, EqExc) and e.x == 2)

    assert new is not None
    assert new.exceptions == [EqExc(x=2)]


def test_split() -> None:
    te = EqTypeError()
    ve = ValueErrorTypeError()
    eg = BaseExceptionGroup('foo', [te, EqExc(x=2), BaseExceptionGroup('msg', [EqExc(x=1), EqExc(x=3), ve])])

    keep, discard = eg.split(lambda e: isinstance(e, TypeError) or isinstance(e, EqExc) and e.x > 1)

    assert keep == BaseExceptionGroup('foo', [te, EqExc(x=2), BaseExceptionGroup('msg', [EqExc(x=3)])])
    assert discard == BaseExceptionGroup('foo', [BaseExceptionGroup('msg', [EqExc(x=1), ve])])


def test_raise() -> None:
    with pytest.raises(BaseExceptionGroup):
        raise BaseExceptionGroup('foo', [EqTypeError(), EqExc(x=2), BaseExceptionGroup('msg', [EqExc(x=1)])])
