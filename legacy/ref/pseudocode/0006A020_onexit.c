/* ea:   0x0006A020 */
/* name: _onexit */
/* size: 164 */
int __fastcall onexit(int a1)
{
  UINT v2; // r0
  char *v3; // r4
  char *v4; // r0
  UINT v5; // r0

  v2 = LocalSize((HLOCAL)dword_8123C);
  v3 = (char *)dword_81238;
  if ( v2 < dword_81238 - dword_8123C + 4 )
  {
    if ( dword_8123C != 0 )
    {
      v5 = LocalSize((HLOCAL)dword_8123C);
      v4 = (char *)LocalReAlloc((HLOCAL)dword_8123C, v5 + 16, 2u);
    }
    else
    {
      v4 = (char *)LocalAlloc(0, 0x10u);
    }
    if ( v4 == nullptr )
      return 0;
    v3 = &v4[4 * ((dword_81238 - dword_8123C) >> 2)];
    dword_8123C = (int)v4;
  }
  *(_DWORD *)v3 = a1;
  dword_81238 = (int)(v3 + 4);
  return a1;
}

