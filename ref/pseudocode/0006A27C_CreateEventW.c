/* ea:   0x0006A27C */
/* name: CreateEventW */
/* size: 8 */
// attributes: thunk
HANDLE __stdcall CreateEventW(
        LPSECURITY_ATTRIBUTES lpEventAttributes,
        BOOL bManualReset,
        BOOL bInitialState,
        LPCWSTR lpName)
{
  return __imp_CreateEventW(lpEventAttributes, bManualReset, bInitialState, lpName);
}

