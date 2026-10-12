/* ea:   0x0006A3CC */
/* name: FlushInstructionCache */
/* size: 8 */
// attributes: thunk
BOOL __stdcall FlushInstructionCache(HANDLE hProcess, LPCVOID lpBaseAddress, DWORD dwSize)
{
  return __imp_FlushInstructionCache(hProcess, lpBaseAddress, dwSize);
}

