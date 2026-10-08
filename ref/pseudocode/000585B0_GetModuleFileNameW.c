/* ea:   0x000585B0 */
/* name: GetModuleFileNameW */
/* size: 8 */
// attributes: thunk
DWORD __stdcall GetModuleFileNameW(HMODULE hModule, LPWSTR lpFilename, DWORD nSize)
{
  return __imp_GetModuleFileNameW(hModule, lpFilename, nSize);
}

