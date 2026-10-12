/* ea:   0x0006A2F4 */
/* name: SuspendThread */
/* size: 8 */
// attributes: thunk
DWORD __stdcall SuspendThread(HANDLE hThread)
{
  return __imp_SuspendThread(hThread);
}

