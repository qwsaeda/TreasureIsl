/* ea:   0x000585F8 */
/* name: GetFileSize */
/* size: 8 */
// attributes: thunk
DWORD __stdcall GetFileSize(HANDLE hFile, LPDWORD lpFileSizeHigh)
{
  return __imp_GetFileSize(hFile, lpFileSizeHigh);
}

