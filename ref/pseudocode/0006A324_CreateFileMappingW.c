/* ea:   0x0006A324 */
/* name: CreateFileMappingW */
/* size: 8 */
// attributes: thunk
HANDLE __stdcall CreateFileMappingW(
        HANDLE hFile,
        LPSECURITY_ATTRIBUTES lpFileMappingAttributes,
        DWORD flProtect,
        DWORD dwMaximumSizeHigh,
        DWORD dwMaximumSizeLow,
        LPCWSTR lpName)
{
  return __imp_CreateFileMappingW(
           hFile,
           lpFileMappingAttributes,
           flProtect,
           dwMaximumSizeHigh,
           dwMaximumSizeLow,
           lpName);
}

