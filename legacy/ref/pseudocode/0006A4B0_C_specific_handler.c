/* ea:   0x0006A4B0 */
/* name: _C_specific_handler */
/* size: 8 */
// attributes: thunk
EXCEPTION_DISPOSITION C_specific_handler(
        struct _EXCEPTION_RECORD *ExceptionRecord,
        void *EstablisherFrame,
        struct _CONTEXT *ContextRecord,
        struct _DISPATCHER_CONTEXT *DispatcherContext)
{
  return _C_specific_handler(ExceptionRecord, EstablisherFrame, ContextRecord, DispatcherContext);
}

