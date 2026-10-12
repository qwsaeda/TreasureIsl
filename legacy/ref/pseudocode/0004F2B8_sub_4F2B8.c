/* ea:   0x0004F2B8 */
/* name: sub_4F2B8 */
/* size: 156 */
_DWORD *__fastcall sub_4F2B8(_DWORD *a1, const void *a2)
{
  void *v4; // r4
  int v5; // r0
  _DWORD *v6; // r5
  int v7; // r4
  int v8; // r1
  bool v9; // zf
  _DWORD *result; // r0

  v4 = a1 + 256;
  memset(a1 + 256, 0, 0x48u);
  memcpy(v4, a2, 0x48u);
  a1[275] = a1[256];
  a1[277] = 6;
  v5 = 0;
  v6 = a1;
  do
  {
    v7 = v5 + 1;
    sub_445F4((int)a1, v5 + 1)[5] = 0;
    v8 = v6[268];
    ++v6;
    v9 = v8 == 0;
    if ( v8 == 0 )
      v8 = a1[277] - 1;
    v5 = v7;
    if ( v9 )
      a1[277] = v8;
  }
  while ( v7 < 6 );
  result = sub_445F4((int)a1, 1);
  result[5] = 1;
  return result;
}

