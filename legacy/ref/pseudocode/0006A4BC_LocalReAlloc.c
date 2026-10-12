/* ea:   0x0006A4BC */
/* name: LocalReAlloc */
/* size: 8 */
// attributes: thunk
HLOCAL __stdcall LocalReAlloc(HLOCAL hMem, UINT cbBytes, UINT fuFlags)
{
  return __imp_LocalReAlloc(hMem, cbBytes, fuFlags);
}

