import sys

if sys.implementation.name == 'cpython':
    from inline_snapshot import snapshot
else:
    # inline-snapshot has currently no pypy support.
    # This fallback allows snapshots to be used during pypy tests.
    def snapshot(value):
        return value
