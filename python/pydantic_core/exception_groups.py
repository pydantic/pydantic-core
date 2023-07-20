def enable_validation_cause_tracebacks():
    import sys

    from exceptiongroup import BaseExceptionGroup

    original_excepthook = sys.excepthook

    def traverse_causes_on_crash(start_exc, seen):
        excs = [start_exc]

        if isinstance(start_exc, BaseExceptionGroup):
            excs.extend(start_exc.exceptions)

        for exc in excs:
            if exc in seen:
                continue
            seen.add(exc)

            cause = getattr(exc, '__cause__', None)
            if cause is not None:
                traverse_causes_on_crash(cause, seen)

    def replacement_excepthook(exc_type, exc_value, exc_traceback):
        # Traverse the causes:
        traverse_causes_on_crash(exc_value, set())

        # Call the original excepthook:
        original_excepthook(exc_type, exc_value, exc_traceback)

    if original_excepthook is not replacement_excepthook:
        sys.excepthook = replacement_excepthook
