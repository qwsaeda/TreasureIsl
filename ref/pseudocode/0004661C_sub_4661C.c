/* ea:   0x0004661C */
/* name: sub_4661C */
/* size: 180 */
_DWORD *__fastcall sub_4661C(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  _WORD *v7; // r4
  __int16 v8; // r2
  _DWORD *result; // r0

  v7 = a1 + 260;
  do
  {
    v8 = *(_WORD *)((char *)v7 + a5 - (_DWORD)(a1 + 260));
    *v7++ = v8;
  }
  while ( v8 != 0 );
  a1[255] = a2;
  a1[256] = a3;
  a1[259] = a4;
  if ( a3 != 0 )
  {
    a1[257] = 2;
    result = sub_445F4((int)a1, 22);
    a1[258] = result;
  }
  else
  {
    sub_445F4((int)a1, 11)[5] = 0;
    sub_445F4((int)a1, 12)[5] = 0;
    sub_445F4((int)a1, 21)[5] = 0;
    result = sub_445F4((int)a1, 22);
  }
  result[5] = 0;
  return result;
}

