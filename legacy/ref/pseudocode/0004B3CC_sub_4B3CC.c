/* ea:   0x0004B3CC */
/* name: sub_4B3CC */
/* size: 76 */
_DWORD *__fastcall sub_4B3CC(_DWORD *a1, int a2, int a3, int a4)
{
  _DWORD *result; // r0

  a1[256] = a2;
  a1[257] = a3;
  a1[258] = a4;
  if ( a3 == 0 )
    sub_445F4((int)a1, 2)[5] = 0;
  result = (_DWORD *)a1[256];
  if ( result == nullptr )
  {
    result = sub_445F4((int)a1, 1);
    result[5] = 0;
  }
  return result;
}

