/* ea:   0x0006A2B8 */
/* name: WaitForSingleObject */
/* size: 8 */
// attributes: thunk
DWORD __stdcall WaitForSingleObject(HANDLE hHandle, DWORD dwMilliseconds)
{
  return __imp_WaitForSingleObject(hHandle, dwMilliseconds);
}

