/* ea:   0x0006A288 */
/* name: CreateThread */
/* size: 8 */
// attributes: thunk
HANDLE __stdcall CreateThread(
        LPSECURITY_ATTRIBUTES lpsa,
        DWORD cbStack,
        LPTHREAD_START_ROUTINE lpStartAddr,
        LPVOID lpvThreadParam,
        DWORD fdwCreate,
        LPDWORD lpIDThread)
{
  return __imp_CreateThread(lpsa, cbStack, lpStartAddr, lpvThreadParam, fdwCreate, lpIDThread);
}

