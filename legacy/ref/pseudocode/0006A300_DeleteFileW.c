/* ea:   0x0006A300 */
/* name: DeleteFileW */
/* size: 8 */
// attributes: thunk
BOOL __stdcall DeleteFileW(LPCWSTR lpFileName)
{
  return __imp_DeleteFileW(lpFileName);
}

