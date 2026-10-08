/* ea:   0x0006A4C8 */
/* name: LocalAlloc */
/* size: 8 */
// attributes: thunk
HLOCAL __stdcall LocalAlloc(UINT fuFlags, UINT cbBytes)
{
  return __imp_LocalAlloc(fuFlags, cbBytes);
}

