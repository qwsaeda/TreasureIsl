/* ea:   0x0006A234 */
/* name: SetThreadPriority */
/* size: 8 */
// attributes: thunk
BOOL __stdcall SetThreadPriority(HANDLE hThread, int nPriority)
{
  return __imp_SetThreadPriority(hThread, nPriority);
}

