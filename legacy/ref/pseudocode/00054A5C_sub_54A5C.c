/* ea:   0x00054A5C */
/* name: sub_54A5C */
/* size: 96 */
int __fastcall sub_54A5C(_DWORD *a1, int a2, int a3, int a4)
{
  __int64 v7; // r0
  int v8; // r2
  _DWORD *v9; // r0
  int v10; // r1
  __int64 v11; // r0

  v7 = 1717986919LL * a2;
  if ( (SHIDWORD(v7) >> 2) + (HIDWORD(v7) >> 31) != 0 )
  {
    v11 = 1717986919LL * ((SHIDWORD(v7) >> 2) + (HIDWORD(v7) >> 31));
    v8 = a3 + 5;
    if ( (SHIDWORD(v11) >> 2) + (HIDWORD(v11) >> 31) != 0 )
      v8 = a3;
    v9 = a1;
    v10 = a2;
  }
  else
  {
    v8 = 10;
    v9 = a1;
    v10 = a2;
  }
  return sub_44828(v9, v10, v8, a4);
}

