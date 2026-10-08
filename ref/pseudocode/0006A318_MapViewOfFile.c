/* ea:   0x0006A318 */
/* name: MapViewOfFile */
/* size: 8 */
// attributes: thunk
LPVOID __stdcall MapViewOfFile(
        HANDLE hFileMappingObject,
        DWORD dwDesiredAccess,
        DWORD dwFileOffsetHigh,
        DWORD dwFileOffsetLow,
        DWORD dwNumberOfBytesToMap)
{
  return __imp_MapViewOfFile(
           hFileMappingObject,
           dwDesiredAccess,
           dwFileOffsetHigh,
           dwFileOffsetLow,
           dwNumberOfBytesToMap);
}

